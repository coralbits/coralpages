use anyhow::Result;
use minijinja::context;
use std::sync::Arc;
use tracing::info;

use crate::file::FileStore;
use crate::traits::Store;
use crate::PageRenderer;
use poem::{
    listener::TcpListener,
    middleware::{NormalizePath, Tracing, TrailingSlash},
    EndpointExt, Error as PoemError, Request, Route, Server,
};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    ApiResponse, Object, OpenApi, OpenApiService,
};

pub struct Api {
    renderer: Arc<PageRenderer>,
}

#[derive(Object)]
struct PageRenderResponseJson {
    body: String,
    css: Option<String>,
    js: Option<String>,
}

#[derive(ApiResponse)]
enum PageRenderResponse {
    #[oai(status = 200, content_type = "application/json; charset=utf-8")]
    Json(Json<PageRenderResponseJson>),
    #[oai(status = 200, content_type = "text/html; charset=utf-8")]
    Html(PlainText<String>),
    #[oai(status = 200, content_type = "text/css; charset=utf-8")]
    Css(PlainText<String>),
}

#[derive(Object)]
struct PageInfoResults {
    count: u64,
    results: Vec<PageInfo>,
}

#[derive(Object)]
struct PageInfo {
    name: String,
    path: String,
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

    #[oai(path = "/render/:store/:path<.*>", method = "get")]
    async fn render(
        &self,
        request: &Request,
        Path(store): Path<String>,
        Path(path): Path<String>,
    ) -> Result<PageRenderResponse, PoemError> {
        let pagename = format!("{}/{}", store, path);
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

        let rendered = self.renderer.render_page(&page, &ctx).await.map_err(|e| {
            PoemError::from_string(e.to_string(), poem::http::StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        let accept_type_ = if let Some(accept) = request.headers().get("Accept") {
            accept.to_str().unwrap().split(";").next().unwrap().trim()
        } else {
            "application/json"
        };

        info!("Accept: {}", accept_type_);

        let response = match accept_type_ {
            "text/html" => PageRenderResponse::Html(PlainText(rendered.body)),
            "text/css" => PageRenderResponse::Css(PlainText("/** not yet **/".to_string())),
            _ => PageRenderResponse::Json(Json(PageRenderResponseJson {
                body: rendered.body,
                css: None,
                js: None,
            })),
        };
        Ok(response)
    }

    #[oai(path = "/page", method = "get")]
    async fn page(&self) -> Result<Json<PageInfoResults>, PoemError> {
        return Ok(Json(PageInfoResults {
            count: 0,
            results: vec![],
        }));
    }
    #[oai(path = "/page/", method = "get")]
    async fn page2(&self) -> Result<Json<PageInfoResults>, PoemError> {
        return Ok(Json(PageInfoResults {
            count: 0,
            results: vec![],
        }));
    }
}

pub async fn start(listen: &str) -> Result<()> {
    let api = Api::new()?;
    let api_service = OpenApiService::new(api, "Page Viewer", "0.1.0").server("/api/v1");

    let docs = api_service.swagger_ui();
    let app = Route::new()
        .nest("api/v1", api_service)
        .nest("/docs", docs)
        .with(Tracing)
        .with(NormalizePath::new(TrailingSlash::Trim));

    let listener = TcpListener::bind(listen);
    Server::new(listener).run(app).await?;

    Ok(())
}
