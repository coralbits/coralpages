use std::collections::HashMap;

use async_trait::async_trait;
use crate::{
    page::types::{Page, Widget},
    store::types::PageInfo,
    ResultI,
};

#[async_trait]
pub trait Store: Send + Sync {
    async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>>;
    async fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>>;
    async fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()>;
    async fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool>;
    async fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &Option<HashMap<String, String>>,
    ) -> anyhow::Result<ResultI<PageInfo>>;
}
