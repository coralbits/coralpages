// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use std::collections::HashMap;

use crate::{
    code::CodeStore,
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
        let mut css_variables = self
            .css_variables
            .iter()
            .map(|(k, v)| {
                if k.starts_with("--") {
                    v.clone()
                } else {
                    format!("{} {{\n {}\n }}\n", k, v)
                }
            })
            .collect::<Vec<String>>();
        css_variables.sort_by(|a, b| a.cmp(b));
        let css_variables = css_variables.join("\n");

        format!("{}", css_variables)
    }

    pub fn render_full_html_page(&self) -> String {
        let css = self.get_css();
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
{}
</style>
</head>
<body>
{}
</body>
</html>"#,
            css, self.body
        );
        html
    }
}

pub struct RenderedingPageData<'a> {
    page: &'a Page,
    store: &'a dyn Store,
    env: &'a Environment<'a>,
    pub rendered_page: RenderedPage,
    debug: bool,
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
            debug: false,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
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

        // TODO is forcing create a clone always, when in most cases is not needed. But have lifetime problems if not.
        // Also not best way to get the widget type
        let ctx = if widget.name == "static_context" || widget.name == "url_context" {
            debug!("Getting static context for element: {:?}", element.widget);
            let ctx = match CodeStore::get_nested_widget_context(element, &ctx).await {
                Ok(ctx) => ctx,
                Err(e) => {
                    error!(
                        "Error getting static context for element: {:?}: {}",
                        element.widget, e
                    );
                    if self.debug {
                        return Ok(format!(
                            "<pre style=\"color:red;\">{}</pre>",
                            HtmlEscape(&e.to_string()).to_string()
                        ));
                    }
                    return Err(e);
                }
            };
            ctx
        } else {
            ctx.clone()
        };

        // Render recursively all the children, and add to context.children as a list
        let mut children = Vec::new();
        for child in &element.children {
            debug!("Rendering child: {:?}", child.widget);
            let rendered_child = Box::pin(self.render_element(child, &ctx)).await?;
            children.push(rendered_child);
        }
        let render_ctx = context! { ..ctx, ..context!{children => children} };

        let rendered_element = self.render_widget(&widget, element, render_ctx).await;

        let rendered_text = match rendered_element {
            Ok(rendered_element) => rendered_element,
            Err(e) => {
                if self.debug {
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

        let ctx = if widget.name == "static_context" || widget.name == "url_context" {
            debug!("Getting static context for element: {:?}", element.widget);
            CodeStore::get_nested_widget_context(element, &ctx).await?
        } else {
            ctx
        };

        let ctx = if element.classes.is_empty() {
            ctx
        } else {
            let mut classes = vec![];

            for class in &element.classes {
                if let Some(classdef) = self.store.load_css_class_definition(class).await? {
                    self.rendered_page
                        .css_variables
                        .insert(format!("--{}", class), classdef.css.clone());
                    classes.push(classdef.name.clone());
                } else {
                    error!("CSS class not found: {}", class);
                    return Err(anyhow::anyhow!("CSS class not found: {}", class));
                };
            }

            context! {
                ..ctx,
                ..context! {
                    classes,
                }
            }
        };

        let non_templated_context = context! {
            data => context!{
                ..minijinja::Value::from_serialize(&element.data),
                ..context! {
                    id => &element.id,
                }
            },
            context => ctx
        };

        // this should be some if, not every element should have this
        let templated_context = Self::render_data_context(&element.data, non_templated_context)?;
        let render_ctx = context! {
            data => context!{
                ..minijinja::Value::from_serialize(templated_context),
                ..context! {
                    id => &element.id,
                }
            },
            context => ctx
        };

        // debug!("Render context: {:?}", render_ctx);
        let rendered_element = match template.render(render_ctx) {
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

    fn render_data_context(
        data: &HashMap<String, String>,
        ctx: minijinja::Value,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut result = HashMap::new();

        for (k, v) in data {
            let rendered_v = Self::render_data_context_str(v, ctx.clone())?;
            result.insert(k.clone(), rendered_v);
        }

        Ok(result)
    }

    fn render_data_context_str(data: &String, ctx: minijinja::Value) -> anyhow::Result<String> {
        // debug!("Rendering data context: {:?}", ctx);
        if data.contains("{{") || data.contains("{%") {
            let env = minijinja::Environment::new();
            let template = env.template_from_str(data)?;
            let rendered_data = template.render(ctx)?;
            debug!("Rendered data: {:?} -> {:?}", data, rendered_data);
            Ok(rendered_data)
        } else {
            Ok(data.clone())
        }
    }
}
