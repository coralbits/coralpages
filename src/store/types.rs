use poem_openapi::Object;

use crate::Page;

#[derive(Object)]
pub struct PageInfo {
    pub id: String,
    pub store: String,
    pub title: String,
    pub url: String,
}

#[derive(Object)]
pub struct PageInfoResults {
    pub count: usize,
    pub results: Vec<PageInfo>,
}
