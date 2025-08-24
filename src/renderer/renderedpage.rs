use std::collections::HashMap;

use crate::{
    page::types::{Element, MetaDefinition, Page, Widget},
    store::traits::Store,
};

use minijinja::{context, Environment};
use tracing::info;

#[derive(Debug)]
pub struct RenderedPage {
    pub body: String,
    pub headers: HashMap<String, String>,
    pub response_code: u16,
    pub meta: Vec<MetaDefinition>,
    pub css_variables: HashMap<String, String>,
    pub errors: Vec<anyhow::Error>,
}

impl RenderedPage {
    pub fn new() -> Self {
        Self {
            body: String::new(),
            headers: HashMap::new(),
            response_code: 200,
            meta: Vec::new(),
            css_variables: HashMap::new(),
            errors: Vec::new(),
        }
    }
}

pub struct RenderedingPageData<'a> {
    page: &'a Page,
    store: &'a dyn Store,
    env: &'a Environment<'a>,
    pub rendered_page: RenderedPage,
}

impl<'a> RenderedingPageData<'a> {
    pub fn new(page: &'a Page, store: &'a dyn Store, env: &'a Environment) -> Self {
        Self {
            page: page,
            store: store,
            env: env,
            rendered_page: RenderedPage::new(),
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        for element in &self.page.children {
            self.render_element(element)?;
        }

        Ok(())
    }

    pub fn render_element(&mut self, element: &Element) -> anyhow::Result<()> {
        let widget = self.store.load_widget_definition(&element.widget)?;
        let widget = match widget {
            Some(widget) => widget,
            None => return Err(anyhow::anyhow!("Widget not found: {}", element.widget)),
        };

        let rendered_element = self.render_widget(&widget, element)?;

        self.rendered_page.body.push_str(&rendered_element);
        Ok(())
    }

    pub fn render_widget(&mut self, widget: &Widget, element: &Element) -> anyhow::Result<String> {
        let template = self.env.template_from_str(&widget.html)?;

        info!(
            "Rendering widget={} with data={:?}",
            widget.name, element.data
        );
        let rendered_element = template.render(context!(data => element.data.clone()))?;

        Ok(rendered_element)
    }
}

#[cfg(test)]
mod tests {
    use crate::store::factory::StoreFactory;

    use super::*;

    #[test]
    fn test_rendered_page() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string());
        let mut store = StoreFactory::new();
        let rendered_page = RenderedingPageData::new(&page, &store);

        println!("Rendered page: {:?}", rendered_page.rendered_page);
    }
}
