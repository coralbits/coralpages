use std::collections::HashMap;

use crate::cache::cache;
use async_trait::async_trait;
use minijinja::{context, Value};
use tracing::debug;

use crate::{traits::Store, Element, Widget, WidgetEditor, WidgetResults};

pub struct CodeStore {
    name: String,
}

impl CodeStore {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub async fn get_nested_widget_context(
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<minijinja::Value> {
        debug!("Getting nested widget context: widget={}", element.widget);
        match element.widget.split("/").nth(1).unwrap_or("??") {
            "static_context" => CodeStore::static_context(element, ctx).await,
            "url_context" => CodeStore::url_context(element, ctx).await,
            name => Err(anyhow::anyhow!("Widget not found: {}", name)),
        }
    }
    async fn static_context(
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<minijinja::Value> {
        let null = "null".to_string();
        let value_str = element.data.get("value").unwrap_or(&null);
        let value_json: Value = serde_json::from_str(value_str)?;
        let mut hashmap: HashMap<String, Value> = HashMap::new();
        let key = element
            .data
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("Key not found"))?;

        debug!(
            "Nested widget context: Value key={key} string={value_str}",
            key = key,
            value_str = value_str
        );
        hashmap.insert(key.to_string(), value_json);
        let ctx = context! {
            ..hashmap,
            ..ctx.clone(),
        };
        Ok(ctx)
    }

    async fn url_context(
        element: &Element,
        ctx: &minijinja::Value,
    ) -> anyhow::Result<minijinja::Value> {
        let url = element
            .data
            .get("url")
            .ok_or_else(|| anyhow::anyhow!("URL not found"))?;
        let key = element
            .data
            .get("key")
            .ok_or_else(|| anyhow::anyhow!("Key not found"))?;

        // first all read
        let value = if let Some(value_str) = cache::cache().get(url).await {
            let value: Value = serde_json::from_str(&value_str)?;
            debug!("Cache hit for URL: {}, value length={:?}", url, value.len());
            Some(value.clone())
        } else {
            debug!("Cache miss for URL: {}", url);
            None
        };

        // If fail then write
        let value = match value {
            Some(value) => value,
            None => {
                // get the url contents, ask for application/json
                let client = reqwest::Client::new();
                let url_contents = client
                    .get(url)
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .header(reqwest::header::USER_AGENT, "page-viewer")
                    .send()
                    .await?;
                let body = url_contents.bytes().await?;
                cache::cache()
                    .set(url, &String::from_utf8(body.to_vec())?)
                    .await;
                debug!("Body length: {:?}", body.len());
                let body: Value = serde_json::from_slice(&body)?;
                body
            }
        };

        // debug!(
        //     "URL context: URL={url} body={body} key={key}",
        //     url = url,
        //     body = value,
        //     key = key
        // );
        let mut hashmap: HashMap<String, Value> = HashMap::new();
        hashmap.insert(key.to_string(), value);

        let ctx = context! {
            ..hashmap,
            ..ctx.clone()
        };
        Ok(ctx)
    }
}

#[async_trait]
impl Store for CodeStore {
    fn name(&self) -> &str {
        &self.name
    }

    async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        match path {
    "static_context" => {
        Ok(Some(Widget {
            name: "static_context".to_string(),
            description: "Static context".to_string(),
            html: "{% for child in context.children %}{{child}}{% endfor %}".to_string(),
            css: "".to_string(),
            editor: vec![
                WidgetEditor::new()
                .with_editor_type("description".to_string())
                .with_label("Description".to_string())
                .with_placeholder("This variable will be added to the context for the children of this widget, and can be accessed in the template code".to_string())
                ,
                WidgetEditor::new()
                .with_editor_type("text".to_string())
                .with_label("Variable name".to_string())
                .with_name("key".to_string())
                .with_placeholder("Enter variable name".to_string()) 
                ,
                WidgetEditor::new()
                .with_editor_type("textarea".to_string())
                .with_label("Static JSON value".to_string())
                .with_name("value".to_string())
                .with_placeholder("Enter static JSON code".to_string()) 
                ,
            ],
            icon: "static_context".to_string(),
        }))
    }
    "url_context" => {
        Ok(Some(Widget {
            name: "url_context".to_string(),
            description: "URL context".to_string(),
            html: "{% for child in context.children %}{{child}}{% endfor %}".to_string(),
            css: "".to_string(),
            editor: vec![
                WidgetEditor::new()
                .with_editor_type("text".to_string())
                .with_label("Variable name".to_string())
                .with_name("key".to_string())
                .with_placeholder("Enter variable name".to_string()) 
                ,
                WidgetEditor::new()
                .with_editor_type("text".to_string())
                .with_label("URL".to_string())
                .with_name("url".to_string())
                .with_placeholder("Enter URL".to_string())
                ,
            ],
            icon: "url_context".to_string(),
        }))
    },
    _ => {
        Ok(None)
    }
        }
    }

    async fn get_widget_list(&self) -> anyhow::Result<WidgetResults> {
        Ok(WidgetResults {
            count: 1,
            results: vec![
                self.load_widget_definition("static_context")
                    .await?
                    .unwrap(),
                self.load_widget_definition("url_context").await?.unwrap(),
            ],
        })
    }
}
