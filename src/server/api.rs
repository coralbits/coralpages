use anyhow::Result;
use minijinja::context;
use poem::middleware::Cors;
use std::{collections::HashMap, sync::Arc};
use tracing::info;

use crate::page::types::{PageInfo, ResultPageList};
use crate::server::PageRenderResponse;
use crate::traits::Store;
use crate::{file::FileStore, renderedpage::RenderedPage, renderresponse::PageRenderResponseJson};
use crate::{Page, PageRenderer, WidgetResults};
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
    pub fn new() -> Result<Self> {
        let mut renderer = PageRenderer::new();
        renderer
            .store
            .add_store("builtin", Box::new(FileStore::new("builtin/widgets")?));
        renderer
            .store
            .add_store("pages", Box::new(FileStore::new("builtin/pages")?));
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
            Query(None),
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
        Query(template): Query<Option<String>>,
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

        let ctx = context! {};

        let mut rendered = self.renderer.render_page(&page, &ctx).await.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        rendered.store = store.clone();
        rendered.path = path.clone();

        return self.response(request, format, rendered);
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

    #[oai(path = "/page/:store/:path/", method = "get")]
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
        Ok(Json(page))
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
}

pub async fn start(listen: &str) -> Result<()> {
    let api = Api::new()?;
    let api_service = OpenApiService::new(api, "Page Viewer", "0.1.0").server("/api/v1");

    let cors = Cors::new()
        // .allow_origin("*")
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["*"]);
    let docs = api_service.swagger_ui();
    let app = Route::new()
        .nest("api/v1", api_service)
        .nest("/docs", docs)
        .with(Tracing)
        .with(NormalizePath::new(TrailingSlash::Trim))
        .with(cors);

    let listener = TcpListener::bind(listen);
    Server::new(listener).run(app).await?;

    Ok(())
}
