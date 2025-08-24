use std::collections::HashMap;

use crate::{
    page::types::{Page, Widget},
    store::types::PageInfo,
    ResultI,
};

pub trait Store {
    fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>>;
    fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>>;
    fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()>;
    fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool>;
    fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: &Option<HashMap<String, String>>,
    ) -> anyhow::Result<ResultI<PageInfo>>;
}
