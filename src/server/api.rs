// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use anyhow::Result;
use minijinja::context;
use poem::middleware::Cors;
use poem::web::Redirect;
use poem::{get, handler};
use poem_openapi::payload::Binary;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::page::types::ResultPageList;
use crate::server::PageRenderResponse;
use crate::traits::Store;
use crate::{
    renderedpage::RenderedPage,
    renderer::pdf::render_pdf,
    renderresponse::{Details, PageRenderResponseJson},
    ErrorResponse, StoreError,
};
use crate::{
    CssClass, CssClassResults, IdName, Page, PageRenderer, StoreListResults, WidgetResults,
};
use poem::{
    listener::TcpListener,
    middleware::{NormalizePath, Tracing, TrailingSlash},
    EndpointExt, Error as PoemError, Request, Route, Server,
};
use poem_openapi::param::Query;
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    OpenApi, OpenApiService,
};

pub struct Api {
    renderer: Arc<PageRenderer>,
}

#[OpenApi]
impl Api {
    pub fn new(renderer: PageRenderer) -> Result<Self> {
        Ok(Self {
            renderer: Arc::new(renderer),
        })
    }

    fn store_error_to_poem_error(&self, store_error: &StoreError) -> PoemError {
        let error_response = ErrorResponse::from_store_error(store_error);
        let status_code = poem::http::StatusCode::from_u16(error_response.status)
            .unwrap_or(poem::http::StatusCode::INTERNAL_SERVER_ERROR);

        PoemError::from_string(
            serde_json::to_string(&error_response)
                .unwrap_or_else(|_| error_response.details.clone()),
            status_code,
        )
    }

    #[oai(path = "/render/:store/:path1/:path2", method = "get")]
    async fn render_with_path(
        &self,
        request: &Request,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
        Query(template): Query<Option<String>>,
        Query(debug): Query<Option<bool>>,
    ) -> Result<PageRenderResponse, PoemError> {
        let realpath = format!("{}/{}", path1, path2);
        self.render(
            request,
            Path(store),
            Path(realpath),
            Query(template),
            Query(debug),
        )
        .await
    }

    #[oai(path = "/render/:store/:path", method = "get")]
    async fn render(
        &self,
        request: &Request,
        Path(store): Path<String>,
        Path(path): Path<String>,
        Query(format): Query<Option<String>>,
        Query(debug): Query<Option<bool>>,
        // Query(template): Query<Option<String>>,
    ) -> Result<PageRenderResponse, PoemError> {
        let mut extension = path.split(".").last();
        if extension == Some(&path) {
            extension = None;
        }
        let realpath = if let Some(ext) = extension {
            &path[..path.len() - (ext.len() + 1)]
        } else {
            path.as_str()
        };

        let pagename = format!("{}/{}", store, realpath);
        info!("Loading page definition from path={}", pagename);

        let page = self
            .renderer
            .store
            .load_page_definition(&pagename)
            .await
            .map_err(|e| {
                // Check if it's a StoreError and return structured JSON error
                if let Some(store_error) = e.downcast_ref::<StoreError>() {
                    self.store_error_to_poem_error(store_error)
                } else {
                    // For other errors, create a generic internal error response
                    let error_response = ErrorResponse {
                        details: e.to_string(),
                        code: "INTERNAL_ERROR".to_string(),
                        status: 500,
                        path: None,
                        store: None,
                    };
                    PoemError::from_string(
                        serde_json::to_string(&error_response).unwrap_or_else(|_| e.to_string()),
                        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                }
            })?
            .ok_or_else(|| {
                let error_response = ErrorResponse {
                    details: format!("Page '{}' not found", pagename),
                    code: "PAGE_NOT_FOUND".to_string(),
                    status: 404,
                    path: Some(pagename.clone()),
                    store: None,
                };
                PoemError::from_string(
                    serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| format!("Page '{}' not found", pagename)),
                    poem::http::StatusCode::NOT_FOUND,
                )
            })?;

        let page = page.fix();

        let ctx = context! {};

        let mut rendered = self
            .renderer
            .render_page(&page, &ctx, debug.unwrap_or(false))
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        rendered.store = page.store.clone();
        rendered.path = page.path.clone();

        let accept_type = self.accept_type(request, format, extension);
        return self.response(rendered, accept_type).await;
    }

    #[oai(path = "/render/", method = "post")]
    async fn render_post(
        &self,
        request: &Request,
        Json(page): Json<Page>,
        Query(format): Query<Option<String>>,
        Query(debug): Query<Option<bool>>,
    ) -> Result<PageRenderResponse, PoemError> {
        let page = page.fix();

        let ctx = context! {};

        let debug = debug.unwrap_or(false);
        let rendered = self
            .renderer
            .render_page(&page, &ctx, debug)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            });

        let rendered = match rendered {
            Ok(rendered) => rendered,
            Err(e) => {
                error!("Error rendering page path={}: {:?}", page.path, e);
                return self.error_response(e);
            }
        };

        let accept_type = self.accept_type(request, format, None);
        return self.response(rendered, accept_type).await;
    }

    fn accept_type(
        &self,
        request: &Request,
        format: Option<String>,
        extension: Option<&str>,
    ) -> String {
        let accept_type = request.headers().get("Accept");

        if let Some(extension) = extension {
            match extension {
                "json" => return "application/json".to_string(),
                "html" => return "text/html".to_string(),
                "css" => return "text/css".to_string(),
                "pdf" => return "application/pdf".to_string(),
                _ => return "application/json".to_string(),
            }
        }

        if let Some(format) = format {
            match format.as_str() {
                "application/json" => return "application/json".to_string(),
                "text/json" => return "application/json".to_string(),
                "text/css" => return "text/css".to_string(),
                "application/pdf" => return "application/pdf".to_string(),
                "html" => return "text/html".to_string(),
                "css" => return "text/css".to_string(),
                "pdf" => return "application/pdf".to_string(),
                _ => return "application/json".to_string(),
            }
        }

        if let Some(accept) = accept_type {
            let accept_type = accept
                .to_str()
                .unwrap()
                .split(";")
                .next()
                .unwrap()
                .trim()
                .to_string();
            return accept_type;
        }

        return "application/json".to_string();
    }

    fn error_response(&self, _error: PoemError) -> Result<PageRenderResponse, PoemError> {
        let error_details = Details::new("Error rendering page".to_string());
        let response = PageRenderResponse::Error(Json(error_details));
        Ok(response)
    }

    async fn response(
        &self,
        rendered: RenderedPage,
        accept_type: String,
    ) -> Result<PageRenderResponse, PoemError> {
        let response = match accept_type.as_str() {
            "text/html" => PageRenderResponse::Html(PlainText(rendered.render_full_html_page())),
            "text/css" => PageRenderResponse::Css(PlainText(rendered.get_css())),
            "application/pdf" => {
                PageRenderResponse::Pdf(Binary(render_pdf(&rendered).await.map_err(|e| {
                    error!("Error rendering PDF: {:?}", e);
                    PoemError::from_string(
                        e.to_string(),
                        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                    )
                })?))
            }
            _ => PageRenderResponse::Json(Json(PageRenderResponseJson::from_page_rendered(
                &rendered,
            ))),
        };
        Ok(response)
    }

    #[oai(path = "/page", method = "get")]
    async fn page(
        &self,
        Query(offset): Query<Option<usize>>,
        Query(limit): Query<Option<usize>>,
        Query(r#type): Query<Option<String>>,
        Query(store): Query<Option<String>>,
    ) -> Result<Json<ResultPageList>, PoemError> {
        let mut filter = HashMap::new();

        if let Some(r#type) = r#type {
            filter.insert("type".to_string(), r#type);
        }
        if let Some(store) = store {
            filter.insert("store".to_string(), store);
        }

        let page_list = self
            .renderer
            .store
            .get_page_list(offset.unwrap_or(0), limit.unwrap_or(10), &filter)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        return Ok(Json(page_list));
    }

    // I dont know how to make poem openapi (mayeb some bug?) accept a path as last param.. so I do it manually
    #[oai(path = "/page/:store/:path1/:path2/:path3", method = "get")]
    async fn get_page_definition_with_path_3(
        &self,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
        Path(path3): Path<String>,
    ) -> Result<Json<Page>, PoemError> {
        let realpath = format!("{}/{}/{}", path1, path2, path3);
        self.get_page_definition(Path(store), Path(realpath)).await
    }

    #[oai(path = "/page/:store/:path1/:path2", method = "get")]
    async fn get_page_definition_with_path(
        &self,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
    ) -> Result<Json<Page>, PoemError> {
        let realpath = format!("{}/{}", path1, path2);
        self.get_page_definition(Path(store), Path(realpath)).await
    }

    #[oai(path = "/page/:store/:path", method = "get")]
    async fn get_page_definition(
        &self,
        Path(store): Path<String>,
        Path(path): Path<String>,
    ) -> Result<Json<Page>, PoemError> {
        let store = match self.renderer.store.get_store(&store) {
            Some(store) => store,
            None => {
                return Err(PoemError::from_string(
                    format!("Store '{}' not found", store),
                    poem::http::StatusCode::NOT_FOUND,
                ));
            }
        };

        let page = store.load_page_definition(&path).await.map_err(|e| {
            // Check if it's a StoreError and return structured JSON error
            if let Some(store_error) = e.downcast_ref::<StoreError>() {
                self.store_error_to_poem_error(store_error)
            } else {
                // For other errors, create a generic internal error response
                let error_response = ErrorResponse {
                    details: e.to_string(),
                    code: "INTERNAL_ERROR".to_string(),
                    status: 500,
                    path: None,
                    store: None,
                };
                PoemError::from_string(
                    serde_json::to_string(&error_response).unwrap_or_else(|_| e.to_string()),
                    poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        })?;
        let page = page.ok_or_else(|| {
            let error_response = ErrorResponse {
                details: format!("Page '{}' not found", path),
                code: "PAGE_NOT_FOUND".to_string(),
                status: 404,
                path: Some(path.clone()),
                store: None,
            };
            PoemError::from_string(
                serde_json::to_string(&error_response)
                    .unwrap_or_else(|_| format!("Page '{}' not found", path)),
                poem::http::StatusCode::NOT_FOUND,
            )
        })?;
        let page = page.fix();
        Ok(Json(page))
    }

    #[oai(path = "/page/:store/:path1/:path2/:path3", method = "post")]
    async fn post_page_definition_with_path_3(
        &self,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
        Path(path3): Path<String>,
        Json(page): Json<Page>,
    ) -> Result<Json<Details>, PoemError> {
        let realpath = format!("{}/{}/{}", path1, path2, path3);
        self.post_page_definition(Path(store), Path(realpath), Json(page))
            .await
    }

    #[oai(path = "/page/:store/:path1/:path2", method = "post")]
    async fn post_page_definition_with_path(
        &self,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
        Json(page): Json<Page>,
    ) -> Result<Json<Details>, PoemError> {
        let realpath = format!("{}/{}", path1, path2);
        self.post_page_definition(Path(store), Path(realpath), Json(page))
            .await
    }

    #[oai(path = "/page/:store/:path", method = "post")]
    async fn post_page_definition(
        &self,
        Path(store): Path<String>,
        Path(path): Path<String>,
        Json(page): Json<Page>,
    ) -> Result<Json<Details>, PoemError> {
        let page = page.fix();

        // check it is a valid page
        let store = match self.renderer.store.get_store(&store) {
            Some(store) => store,
            None => {
                return Err(PoemError::from_string(
                    format!("Store '{}' not found", store),
                    poem::http::StatusCode::NOT_FOUND,
                ));
            }
        };

        store
            .save_page_definition(&path, &page)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Json(Details::new("Page definition saved".to_string())))
    }

    #[oai(path = "/page/:store/:path1/:path2", method = "put")]
    async fn put_page_definition_with_path(
        &self,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
    ) -> Result<Json<Details>, PoemError> {
        let realpath = format!("{}/{}", path1, path2);
        self.put_page_definition(Path(store), Path(realpath)).await
    }

    #[oai(path = "/page/:store/:path", method = "put")]
    async fn put_page_definition(
        &self,
        Path(store): Path<String>,
        Path(path): Path<String>,
    ) -> Result<Json<Details>, PoemError> {
        let page = Page::new().with_title(path.clone());

        let page = page.fix();

        let store = match self.renderer.store.get_store(&store) {
            Some(store) => store,
            None => {
                return Err(PoemError::from_string(
                    format!("Store '{}' not found", store),
                    poem::http::StatusCode::NOT_FOUND,
                ));
            }
        };

        store
            .save_page_definition(&path, &page)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Json(Details::new("Page definition saved".to_string())))
    }

    #[oai(path = "/widget/", method = "get")]
    async fn widget(
        &self,
        Query(store): Query<Option<String>>,
    ) -> Result<Json<WidgetResults>, PoemError> {
        let results = if let Some(store) = store {
            if let Some(store) = self.renderer.store.get_store(&store) {
                store.get_widget_list().await
            } else {
                return Err(PoemError::from_string(
                    format!("Store '{}' not found", store),
                    poem::http::StatusCode::NOT_FOUND,
                ));
            }
        } else {
            self.renderer.store.get_widget_list().await
        };

        let results = results.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        Ok(Json(results))
    }

    #[oai(path = "/store", method = "get")]
    async fn get_store_list(&self) -> Result<Json<StoreListResults>, PoemError> {
        let stores = self.renderer.store.get_store_list().await.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let results = stores
            .iter()
            .map(|s| IdName {
                id: s.clone(),
                name: s.clone(),
            })
            .collect();

        Ok(Json(StoreListResults {
            count: stores.len(),
            results,
        }))
    }

    #[oai(path = "/classes/", method = "get")]
    async fn classes(
        &self,
        Query(store): Query<Option<String>>,
    ) -> Result<Json<CssClassResults>, PoemError> {
        let results = if let Some(store) = store {
            if let Some(store) = self.renderer.store.get_store(&store) {
                store.load_css_classes().await
            } else {
                return Err(PoemError::from_string(
                    format!("Store '{}' not found", store),
                    poem::http::StatusCode::NOT_FOUND,
                ));
            }
        } else {
            self.renderer.store.load_css_classes().await
        };
        let results = results.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        Ok(Json(results))
    }

    #[oai(path = "/classes/:store/:name", method = "get")]
    async fn get_css_class_definition(
        &self,
        Path(store): Path<String>,
        Path(name): Path<String>,
    ) -> Result<Json<CssClass>, PoemError> {
        let full_name = format!("{}/{}", store, name);
        let results = self
            .renderer
            .store
            .load_css_class_definition(&full_name)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        let results = results.ok_or_else(|| {
            PoemError::from_string(
                format!("CSS class '{}' not found", full_name),
                poem::http::StatusCode::NOT_FOUND,
            )
        })?;
        Ok(Json(results))
    }
}

#[handler]
async fn root_redirect() -> Redirect {
    return Redirect::moved_permanent("/api/v1/render/default/index?format=html");
}

pub async fn start(listen: &str, renderer: PageRenderer) -> Result<()> {
    let (_, shutdown_rx) = broadcast::channel(1);
    start_with_shutdown(listen, renderer, shutdown_rx).await
}

pub async fn start_with_shutdown(
    listen: &str,
    renderer: PageRenderer,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let api = Api::new(renderer)?;
    let api_service = OpenApiService::new(api, "Page Viewer", "0.1.0").server("/api/v1");

    let cors = Cors::new()
        // .allow_origin("*")
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["content-type"]);
    let docs = api_service.swagger_ui();
    let app = Route::new()
        .nest("api/v1", api_service)
        .nest("/docs", docs)
        .at("/", get(root_redirect))
        .with(Tracing)
        .with(NormalizePath::new(TrailingSlash::Trim))
        .with(cors);

    let listener = TcpListener::bind(listen);
    let server = Server::new(listener);

    // Run the server until shutdown signal is received
    tokio::select! {
        result = server.run(app) => {
            result?;
        }
        _ = shutdown_rx.recv() => {
            info!("Shutdown signal received, stopping server...");
        }
    }

    Ok(())
}
