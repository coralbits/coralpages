use std::collections::HashMap;

use async_trait::async_trait;
use minijinja::{context, Value};
use tracing::{debug, info};

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
    _ => {
        Ok(None)
    }
        }
    }

    async fn get_widget_list(&self) -> anyhow::Result<WidgetResults> {
        Ok(WidgetResults {
            count: 1,
            results: vec![self
                .load_widget_definition("static_context")
                .await?
                .unwrap()],
        })
    }
}
