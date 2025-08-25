use anyhow::Result;
use minijinja::context;
use std::sync::Arc;
use tracing::info;

use crate::file::FileStore;
use crate::traits::Store;
use crate::PageRenderer;
use poem::{
    listener::TcpListener, middleware::Tracing, EndpointExt, Error as PoemError, Route, Server,
};
use poem_openapi::{param::Path, payload::PlainText, OpenApi, OpenApiService};

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

    #[oai(path = "/render/:store/:path", method = "get")]
    async fn render(
        &self,
        Path(store): Path<String>,
        Path(path): Path<String>,
    ) -> Result<PlainText<String>, PoemError> {
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
        Ok(PlainText(rendered.body))
    }
}

pub async fn start(listen: &str) -> Result<()> {
    let api = Api::new()?;
    let api_service = OpenApiService::new(api, "Page Viewer", "0.1.0").server("/api/v1");

    let docs = api_service.swagger_ui();
    let app = Route::new()
        .nest("api/v1", api_service)
        .nest("/docs", docs)
        .with(Tracing);

    let listener = TcpListener::bind(listen);
    Server::new(listener).run(app).await?;

    Ok(())
}
