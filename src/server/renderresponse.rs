use poem_openapi::payload::{Json, PlainText};
use std::collections::HashMap;

use poem_openapi::{ApiResponse, Object};

use crate::renderedpage::RenderedPage;

#[derive(Object)]
pub struct PageRenderHttp {
    headers: HashMap<String, String>,
    response_code: u16,
}

#[derive(Object)]
pub struct PageRenderHead {
    css: String,
    js: String,
    meta: Vec<PageRenderMeta>,
}

#[derive(Object)]
pub struct PageRenderMeta {
    name: String,
    content: String,
}

#[derive(Object)]
pub struct PageRenderResponseJson {
    title: String,
    body: String,
    store: String,
    path: String,
    head: PageRenderHead,
    http: PageRenderHttp,
}

impl PageRenderResponseJson {
    pub fn from_page_rendered(rendered: &RenderedPage) -> Self {
        let meta = rendered
            .meta
            .iter()
            .map(|m| PageRenderMeta {
                name: m.name.clone(),
                content: m.content.clone(),
            })
            .collect();

        Self {
            body: rendered.body.clone(),
            head: PageRenderHead {
                css: rendered.get_css(),
                js: "/** TODO **/".to_string(),
                meta: meta,
            },
            http: PageRenderHttp {
                headers: HashMap::new(),
                response_code: 200,
            },
            path: rendered.path.clone(),
            store: rendered.store.clone(),
            title: rendered.title.clone(),
        }
    }
}

#[derive(ApiResponse)]
pub enum PageRenderResponse {
    #[oai(status = 200, content_type = "application/json; charset=utf-8")]
    Json(Json<PageRenderResponseJson>),
    #[oai(status = 200, content_type = "text/html; charset=utf-8")]
    Html(PlainText<String>),
    #[oai(status = 200, content_type = "text/css; charset=utf-8")]
    Css(PlainText<String>),
}

#[derive(Object)]
pub struct Details {
    pub details: String,
}

impl Details {
    pub fn new(details: String) -> Self {
        Self { details }
    }
}
