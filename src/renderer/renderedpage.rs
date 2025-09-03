use std::collections::HashMap;

use crate::{
    config,
    page::types::{Element, MetaDefinition, Page, Widget},
    store::traits::Store,
};

use minijinja::{context, Environment, HtmlEscape};
use tracing::{debug, error};

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
    pub elapsed: std::time::Instant,
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
            elapsed: std::time::Instant::now(),
        }
    }

    pub fn get_css(&self) -> String {
        let css_variables = self
            .css_variables
            .iter()
            .map(|(k, v)| {
                if k.starts_with("--") {
                    v.clone()
                } else {
                    format!("{} {{\n {}\n }}\n", k, v)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!("{}", css_variables)
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
        let mut rendered_body = String::new();
        for element in &self.page.children {
            debug!("Rendering element: {:?}", element.widget);
            rendered_body.push_str(&self.render_element(element, ctx).await?);
        }

        debug!(
            "Adding extra elements (meta, css, title...): {:?}",
            self.rendered_page.path
        );
        // add the meta
        self.rendered_page.meta.extend(self.page.meta.clone());

        self.rendered_page.body = rendered_body;

        Ok(())
    }

    pub async fn render_element(
        &mut self,
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<String> {
        let widget = self.store.load_widget_definition(&element.widget).await?;
        let widget = match widget {
            Some(widget) => widget,
            None => return Err(anyhow::anyhow!("Widget not found: {}", element.widget)),
        };

        // Render recursively all the children, and add to context.children as a list
        let mut children = Vec::new();
        for child in &element.children {
            debug!("Rendering child: {:?}", child.widget);
            let rendered_child = Box::pin(self.render_element(child, &ctx)).await?;
            children.push(rendered_child);
        }
        let new_ctx = context! { ..ctx.clone(), ..context!{children => children} };

        let rendered_element = self.render_widget(&widget, element, new_ctx).await;

        let rendered_text = match rendered_element {
            Ok(rendered_element) => rendered_element,
            Err(e) => {
                if config::get_debug().await {
                    let ret = format!(
                        "<pre style=\"color:red;\">{}</pre>",
                        HtmlEscape(&e.to_string()).to_string()
                    );
                    self.rendered_page.errors.push(e);
                    ret
                } else {
                    // on no debug, just return an error when rendering a failed widget
                    return Err(e);
                }
            }
        };

        Ok(rendered_text)
    }

    pub async fn render_widget(
        &mut self,
        widget: &Widget,
        element: &Element,
        ctx: minijinja::Value,
    ) -> anyhow::Result<String> {
        debug!("Rendering widget: {:?}", widget.name);

        let template = self.env.template_from_str(&widget.html)?;

        let ctx = context! {
            data => context!{
                ..minijinja::Value::from_serialize(&element.data),
                ..context! {
                    id => &element.id,
                }
            },
            context => ctx
        };

        debug!("Context: {:?}", ctx);
        let rendered_element = match template.render(ctx) {
            Ok(rendered_element) => rendered_element,
            Err(e) => {
                error!(
                    "Error rendering page={}, widget={}: {:?}. html={:?}",
                    self.rendered_page.path, widget.name, e, widget.html
                );
                return Err(anyhow::anyhow!(
                    "Error rendering page={}, widget={}: {:?}",
                    self.rendered_page.path,
                    widget.name,
                    e
                ));
            }
        };
        debug!("Rendered element: {:?}", rendered_element);

        // Add the CSS to the rendered page
        self.rendered_page
            .css_variables
            .insert(format!("--{}", widget.name), widget.css.clone());

        // If the element has an id, add the CSS to the rendered page
        if !element.id.is_empty() && !element.style.is_empty() {
            let css = element
                .style
                .iter()
                .map(|(k, v)| format!("{}: {};", k, v))
                .collect::<Vec<String>>()
                .join("\n");

            self.rendered_page
                .css_variables
                .insert(format!("#{}", element.id), css);
        }

        Ok(rendered_element)
    }
}

#[cfg(test)]
mod tests {
    use minijinja::Environment;
    use tracing::info;

    use crate::{store::factory::StoreFactory, utils::setup_logging, PageRenderer};
    use ctor::ctor;

    use super::*;

    struct TestStore {}

    #[ctor]
    fn setup_logging_() {
        setup_logging(true);
    }

    #[async_trait::async_trait]
    impl Store for TestStore {
        fn name(&self) -> &str {
            "test"
        }

        async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
            debug!("Loading widget definition from path: {}", path);
            let html = match path {
                "text" => "<a class=\"test-link\" id=\"{{data.id}}\">Hello, {{data.text}}!</a>",
                "columns" => "<div class=\"columns column-{{data.id}}\" id=\"{{data.id}}\">{{context.children|join('')}}</div>",
                _ => return Ok(None),
            };

            Ok(Some(Widget {
                name: path.to_string(),
                html: html.to_string(),
                css: ".test-link { background: red; }".to_string(),
                editor: vec![],
                description: "Test widget".to_string(),
                icon: "".to_string(),
            }))
        }
    }

    #[tokio::test]
    async fn test_rendered_page() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![Element::new(
                "test/text".to_string(),
                serde_json::json!({ "text": "Hello, world!" }),
                "test-link".to_string(),
            )
            .with_children(vec![Element::new(
                "test/text".to_string(),
                serde_json::json!({ "text": "Hello, child!" }),
                "test-link-child".to_string(),
            )])]);
        let mut store = StoreFactory::new();
        store.add_store(Box::new(TestStore {}));

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

    #[tokio::test]
    async fn test_rendered_page_css() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![Element::new(
                "test/text".to_string(),
                serde_json::json!({ "text": "Hello, world!" }),
                "test-link-id".to_string(),
            )
            .with_style(std::collections::HashMap::from([(
                "background".to_string(),
                "red".to_string(),
            )]))]);

        // Add the test store to the renderer
        let mut renderer = PageRenderer::new();
        renderer.store.add_store(Box::new(TestStore {}));

        // Render
        let rendered_page = renderer
            .render_page(&page, &minijinja::context! {})
            .await
            .unwrap();

        info!("Rendered page CSS: {:?}", rendered_page.get_css());

        let from_element_class = ".test-link { background: red; }";
        let from_element_id = "#test-link-id {\n background: red;\n }";

        let css = rendered_page.get_css();
        assert!(css.contains(from_element_class));
        assert!(css.contains(from_element_id));
    }

    #[tokio::test]
    async fn test_rendered_page_columns() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![Element::new(
                "test/columns".to_string(),
                serde_json::json!({ "wrap": true, "gap": 12 }),
                "test-columns".to_string(),
            )
            .with_children(vec![
                Element::new(
                    "test/text".to_string(),
                    serde_json::json!({ "text": "Column 1" }),
                    "test-link-1".to_string(),
                ),
                Element::new(
                    "test/text".to_string(),
                    serde_json::json!({ "text": "Column 2" }),
                    "test-link-2".to_string(),
                ),
            ])]);

        let mut renderer = PageRenderer::new();
        renderer.store.add_store(Box::new(TestStore {}));

        let rendered_page = renderer
            .render_page(&page, &minijinja::context! {})
            .await
            .unwrap();

        info!("Rendered page: {:?}", rendered_page.body);
        assert_eq!(rendered_page.body, "<div class=\"columns column-test-columns\" id=\"test-columns\"><a class=\"test-link\" id=\"test-link-1\">Hello, Column 1!</a><a class=\"test-link\" id=\"test-link-2\">Hello, Column 2!</a></div>");
    }
}
