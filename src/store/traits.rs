use std::collections::HashMap;

use crate::page::types::{Page, Widget};
use crate::page::types::{ResultPageList, WidgetResults};
use async_trait::async_trait;

#[async_trait]
pub trait Store: Send + Sync {
    fn name(&self) -> &str;
    async fn load_widget_definition(&self, _path: &str) -> anyhow::Result<Option<Widget>> {
        Ok(None)
    }
    async fn load_page_definition(&self, _path: &str) -> anyhow::Result<Option<Page>> {
        Ok(None)
    }
    async fn save_page_definition(&self, _path: &str, _page: &Page) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Not implemented"))
    }
    async fn delete_page_definition(&self, _path: &str) -> anyhow::Result<bool> {
        Err(anyhow::anyhow!("Not implemented"))
    }
    async fn get_page_list(
        &self,
        _offset: usize,
        _limit: usize,
        _filter: &HashMap<String, String>,
    ) -> anyhow::Result<ResultPageList> {
        Ok(ResultPageList {
            count: 0,
            results: vec![],
        })
    }
    async fn get_widget_list(&self) -> anyhow::Result<WidgetResults> {
        Ok(WidgetResults {
            count: 0,
            results: vec![],
        })
    }
}
