use std::collections::HashMap;

use crate::{
    page::types::{Page, Widget},
    store::types::{PageInfo, PageInfoResults},
    ResultI,
};
use async_trait::async_trait;

#[async_trait]
pub trait Store: Send + Sync {
    async fn load_widget_definition(&self, _path: &str) -> anyhow::Result<Option<Widget>> {
        Ok(None)
    }
    async fn load_page_definition(&self, _path: &str) -> anyhow::Result<Option<Page>> {
        Ok(None)
    }
    async fn save_page_definition(&self, _path: &str, _page: &Page) -> anyhow::Result<()> {
        Ok(())
    }
    async fn delete_page_definition(&self, _path: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn get_page_list(
        &self,
        _offset: usize,
        _limit: usize,
        _filter: &HashMap<String, String>,
    ) -> anyhow::Result<PageInfoResults> {
        Ok(PageInfoResults {
            count: 0,
            results: vec![],
        })
    }
}
