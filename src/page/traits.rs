use crate::page::types::{Element, Page, PageListResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for page operations
#[async_trait]
pub trait PageOperations {
    /// Load a page definition from the store
    async fn load_page_definition(&self, path: &str) -> anyhow::Result<Option<Page>>;

    /// Save a page definition to the store
    async fn save_page_definition(&self, path: &str, page: &Page) -> anyhow::Result<()>;

    /// Delete a page definition from the store
    async fn delete_page_definition(&self, path: &str) -> anyhow::Result<bool>;

    /// Get a list of pages
    async fn get_page_list(
        &self,
        offset: usize,
        limit: usize,
        filter: Option<HashMap<String, String>>,
    ) -> anyhow::Result<PageListResult>;
}

/// Trait for page rendering
pub trait PageRenderer {
    /// Render a page to HTML
    fn render_page(&self, page: &Page) -> anyhow::Result<String>;

    /// Render a single element
    fn render_element(&self, element: &Element) -> anyhow::Result<String>;

    /// Get CSS for a page
    fn get_page_css(&self, page: &Page) -> anyhow::Result<String>;
}

/// Trait for page validation
pub trait PageValidator {
    /// Validate a page definition
    fn validate_page(&self, page: &Page) -> anyhow::Result<()>;

    /// Validate an element
    fn validate_element(&self, element: &Element) -> anyhow::Result<()>;
}
