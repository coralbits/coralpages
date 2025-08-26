use std::collections::HashMap;

use crate::{
    page::types::{Element, MetaDefinition, Page, Widget},
    store::traits::Store,
};

use minijinja::{context, Environment};

#[derive(Debug)]
pub struct RenderedPage {
    pub path: String,
    pub store: String,
    pub title: String,
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
            path: String::new(),
            store: String::new(),
            title: String::new(),
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
        let mut rendered_page = RenderedPage::new();
        rendered_page.path = page.path.clone();
        rendered_page.title = page.title.clone();

        Self {
            page: page,
            store: store,
            env: env,
            rendered_page,
        }
    }

    pub async fn render(&mut self, ctx: &minijinja::Value) -> anyhow::Result<()> {
        for element in &self.page.children {
            self.render_element(element, ctx).await?;
        }

        // add the meta
        self.rendered_page.meta.extend(self.page.meta.clone());

        Ok(())
    }

    pub async fn render_element(
        &mut self,
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<()> {
        let widget = self.store.load_widget_definition(&element.widget).await?;
        let widget = match widget {
            Some(widget) => widget,
            None => return Err(anyhow::anyhow!("Widget not found: {}", element.widget)),
        };

        // Render recursively all the children, and add to context.children as a list
        let mut children = Vec::new();
        for child in &element.children {
            let rendered_child = Box::pin(self.render_element(child, &ctx)).await?;
            children.push(rendered_child);
        }
        let new_ctx = context! { ..ctx.clone(), ..context!{children => children} };

        let rendered_element = self.render_widget(&widget, element, new_ctx).await?;

        self.rendered_page.body.push_str(&rendered_element);
        Ok(())
    }

    pub async fn render_widget(
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
    use tracing::info;

    use crate::{store::factory::StoreFactory, utils};

    use super::*;

    struct TestStore {}

    #[async_trait::async_trait]
    impl Store for TestStore {
        async fn load_widget_definition(&self, _path: &str) -> anyhow::Result<Option<Widget>> {
            Ok(Some(Widget {
                name: "test".to_string(),
                html: "Hello, {{data.text}}!".to_string(),
                css: "".to_string(),
                editor: vec![],
                description: "Test widget".to_string(),
                icon: "".to_string(),
            }))
        }
    }

    #[tokio::test]
    async fn test_rendered_page() {
        utils::setup_logging();

        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![Element::new(
                "test/text".to_string(),
                serde_json::json!({ "text": "Hello, world!" }),
            )
            .with_children(vec![Element::new(
                "test/text".to_string(),
                serde_json::json!({ "text": "Hello, child!" }),
            )])]);
        let mut store = StoreFactory::new();
        store.add_store("test", Box::new(TestStore {}));

        let env = Environment::new();
        let mut rendered_page = RenderedingPageData::new(&page, &store, &env);

        // Test the render method
        let ctx = minijinja::context! {};
        rendered_page.render(&ctx).await.unwrap();

        info!(
            "Rendered page: {:?}",
            rendered_page.rendered_page.body.len()
        );
    }
}
