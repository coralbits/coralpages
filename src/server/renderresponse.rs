// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use poem_openapi::payload::{Binary, Json, PlainText};
use serde::Serialize;
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
    link: Vec<PageRenderLink>,
}

#[derive(Object)]
pub struct PageRenderMeta {
    name: String,
    content: String,
}

#[derive(Object)]
pub struct PageRenderLink {
    href: String,
    rel: String,
}

#[derive(Object)]
pub struct PageRenderResponseJson {
    title: String,
    body: String,
    store: String,
    path: String,
    head: PageRenderHead,
    http: PageRenderHttp,
    elapsed: f32,
}

impl PageRenderResponseJson {
    pub fn from_page_rendered(rendered: &RenderedPage) -> Self {
        // let meta = rendered
        //     .meta
        //     .iter()
        //     .map(|m| PageRenderMeta {
        //         name: m.name.clone(),
        //         content: m.content.clone(),
        //     })
        //     .collect();
        let head = rendered.head.clone();

        Self {
            body: rendered.body.clone(),
            head: PageRenderHead {
                css: rendered.get_css(),
                js: "/** TODO **/".to_string(),
                meta: head
                    .meta
                    .unwrap_or_default()
                    .iter()
                    .map(|m| PageRenderMeta {
                        name: m.name.clone(),
                        content: m.content.clone(),
                    })
                    .collect(),
                link: head
                    .link
                    .unwrap_or_default()
                    .iter()
                    .map(|l| PageRenderLink {
                        href: l.href.clone(),
                        rel: l.rel.clone(),
                    })
                    .collect(),
            },
            http: PageRenderHttp {
                headers: HashMap::new(),
                response_code: 200,
            },
            path: rendered.path.clone(),
            store: rendered.store.clone(),
            title: rendered.title.clone(),
            elapsed: rendered.elapsed.elapsed().as_micros() as f32 / 1000.0,
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
    #[oai(status = 200, content_type = "application/pdf")]
    Pdf(Binary<Vec<u8>>),
    #[oai(status = 500, content_type = "application/json; charset=utf-8")]
    Error(Json<Details>),
}

#[derive(Object, Serialize, Debug)]
pub struct Details {
    pub details: String,
}

impl Details {
    pub fn new(details: String) -> Self {
        Self { details }
    }
}
