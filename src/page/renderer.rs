use std::collections::HashMap;

use crate::page::types::{MetaDefinition, Page};

#[derive(Debug)]
pub struct RenderedPage {
    page: Page,
    body: String,
    headers: HashMap<String, String>,
    response_code: u16,
    meta: Vec<MetaDefinition>,
    css_variables: HashMap<String, String>,
    errors: Vec<anyhow::Error>,
}

impl RenderedPage {
    pub fn new(page: Page) -> Self {
        Self {
            page,
            body: String::new(),
            headers: HashMap::new(),
            response_code: 200,
            meta: Vec::new(),
            css_variables: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.body = "WIP".to_string();
        Ok(())
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rendered_page() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string());
        let mut rendered_page = RenderedPage::new(page);

        println!("Rendered page: {:?}", rendered_page);
    }
}
