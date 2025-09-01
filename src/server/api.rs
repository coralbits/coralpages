use anyhow::Result;
use minijinja::context;
use poem::middleware::Cors;
use poem::web::Redirect;
use poem::{get, handler};
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};

use crate::page::types::ResultPageList;
use crate::server::PageRenderResponse;
use crate::traits::Store;
use crate::{
    renderedpage::RenderedPage,
    renderresponse::{Details, PageRenderResponseJson},
};
use crate::{IdName, Page, PageRenderer, StoreListResults, WidgetResults};
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

    #[oai(path = "/render/:store/:path1/:path2", method = "get")]
    async fn render_with_path(
        &self,
        request: &Request,
        Path(store): Path<String>,
        Path(path1): Path<String>,
        Path(path2): Path<String>,
    ) -> Result<PageRenderResponse, PoemError> {
        let realpath = format!("{}/{}", path1, path2);
        self.render(
            request,
            Path(store),
            Path(realpath),
            Query(None),
            // Query(None),
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
        // Query(template): Query<Option<String>>,
    ) -> Result<PageRenderResponse, PoemError> {
        let realpath = if path.ends_with(".json") {
            path.trim_end_matches(".json")
        } else {
            &path
        };

        // FIXME. Remove any part of the path, keep just the last part
        let realpath = realpath.split("/").last().unwrap();

        let pagename = format!("{}/{}", store, realpath);

        let page = self
            .renderer
            .store
            .load_page_definition(&pagename)
            .await
            .map_err(|e| {
                PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                PoemError::from_string(
                    format!("Page '{}' not found", pagename),
                    poem::http::StatusCode::NOT_FOUND,
                )
            })?;

        let page = page.fix();

        let ctx = context! {};

        let mut rendered = self.renderer.render_page(&page, &ctx).await.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        rendered.store = store.clone();
        rendered.path = path.clone();

        return self.response(request, format, rendered);
    }

    #[oai(path = "/render/", method = "post")]
    async fn render_post(
        &self,
        request: &Request,
        Json(page): Json<Page>,
        Query(format): Query<Option<String>>,
    ) -> Result<PageRenderResponse, PoemError> {
        let page = page.fix();

        let ctx = context! {};

        let rendered = self.renderer.render_page(&page, &ctx).await.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        });

        let rendered = match rendered {
            Ok(rendered) => rendered,
            Err(e) => {
                error!("Error rendering page path={}: {:?}", page.path, e);
                return self.error_response(e);
            }
        };

        return self.response(request, format, rendered);
    }

    fn error_response(&self, _error: PoemError) -> Result<PageRenderResponse, PoemError> {
        let error_details = Details::new("Error rendering page".to_string());
        let response = PageRenderResponse::Error(Json(error_details));
        Ok(response)
    }

    fn response(
        &self,
        request: &Request,
        format: Option<String>,
        rendered: RenderedPage,
    ) -> Result<PageRenderResponse, PoemError> {
        let accept_type_ = self.response_type(request, format);

        info!("Accept: {}", accept_type_);

        let response = match accept_type_.as_str() {
            "text/html" => PageRenderResponse::Html(PlainText(rendered.body)),
            "text/css" => PageRenderResponse::Css(PlainText(rendered.get_css())),
            _ => PageRenderResponse::Json(Json(PageRenderResponseJson::from_page_rendered(
                &rendered,
            ))),
        };
        Ok(response)
    }

    fn response_type(&self, request: &Request, format: Option<String>) -> String {
        // Get from format query parameter
        if let Some(format) = format {
            let format = match format.as_str() {
                "application/json" => "application/json",
                "text/json" => "application/json",
                "text/css" => "text/css",
                "html" => "text/html",
                "css" => "text/css",
                _ => "application/json",
            };
            return format.to_string();
        }

        // Get from Accept header
        let ret = if let Some(accept) = request.headers().get("Accept") {
            accept
                .to_str()
                .unwrap()
                .split(";")
                .next()
                .unwrap()
                .trim()
                .to_string()
        } else {
            "application/json".to_string()
        };

        return ret;
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
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        let page = page.ok_or_else(|| {
            PoemError::from_string(
                format!("Page '{}' not found", path),
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
}

#[handler]
async fn root_redirect() -> Redirect {
    return Redirect::moved_permanent("/api/v1/render/default/index?format=html");
}

pub async fn start(listen: &str, renderer: PageRenderer) -> Result<()> {
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
    Server::new(listener).run(app).await?;

    Ok(())
}
