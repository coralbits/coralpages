use std::collections::HashMap;

use crate::{
    page::types::{Element, MetaDefinition, Page, Widget},
    store::traits::Store,
};

use minijinja::{context, Environment};
use tracing::{debug, info, instrument};

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

    pub fn render(&mut self, ctx: &minijinja::Value) -> anyhow::Result<()> {
        for element in &self.page.children {
            self.render_element(element, ctx)?;
        }

        // add the meta
        self.rendered_page.meta.extend(self.page.meta.clone());

        Ok(())
    }

    pub fn render_element(
        &mut self,
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<()> {
        let widget = self.store.load_widget_definition(&element.widget)?;
        let widget = match widget {
            Some(widget) => widget,
            None => return Err(anyhow::anyhow!("Widget not found: {}", element.widget)),
        };

        // Render recursively all the children, and add to context.children as a list
        let mut children = Vec::new();
        for child in &element.children {
            let rendered_child = self.render_element(child, &ctx)?;
            children.push(rendered_child);
        }
        let new_ctx = context! { ..ctx.clone(), ..context!{children => children} };

        let rendered_element = self.render_widget(&widget, element, new_ctx)?;

        self.rendered_page.body.push_str(&rendered_element);
        Ok(())
    }

    pub fn render_widget(
        &mut self,
        widget: &Widget,
        element: &Element,
        ctx: minijinja::Value,
    ) -> anyhow::Result<String> {
        let template = self.env.template_from_str(&widget.html)?;

        let ctx = context! { ..ctx, ..context!{data => element.data.clone()} };

        let rendered_element = template.render(ctx)?;

        // Add the CSS to the rendered page
        self.rendered_page
            .css_variables
            .insert(widget.name.clone(), widget.css.clone());

        Ok(rendered_element)
    }
}

#[cfg(test)]
mod tests {
    use minijinja::Environment;

    use crate::store::factory::StoreFactory;

    use super::*;

    #[test]
    fn test_rendered_page() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![Element::new(
                "text".to_string(),
                serde_json::json!({ "text": "Hello, world!" }),
            )
            .with_children(vec![Element::new(
                "text".to_string(),
                serde_json::json!({ "text": "Hello, child!" }),
            )])]);
        let mut store = StoreFactory::new();
        let env = Environment::new();
        let rendered_page = RenderedingPageData::new(&page, &store, &env);

        info!(
            "Rendered page: {:?}",
            rendered_page.rendered_page.body.len()
        );
    }
}
