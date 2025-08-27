use poem_openapi::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Widget {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub html: String,
    #[serde(default)]
    pub css: String,
    #[serde(default)]
    pub editor: Vec<WidgetEditor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct WidgetEditor {
    #[serde(rename = "type")]
    pub editor_type: String,
    pub label: String,
    pub name: String,
    #[serde(default)]
    pub placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct WidgetResults {
    pub count: usize,
    pub results: Vec<Widget>,
}

/// A meta definition for page metadata
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MetaDefinition {
    pub name: String,
    pub content: String,
}

/// Each widget use in a page, with content, and maybe more children
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Element {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type")]
    pub widget: String,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub children: Vec<Element>,
    #[serde(default)]
    pub style: std::collections::HashMap<String, String>,
}

impl Element {
    pub fn new(widget: String, data: serde_json::Value, id: String) -> Self {
        Self {
            id,
            widget,
            data,
            children: Vec::new(),
            style: std::collections::HashMap::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    pub fn with_style(mut self, style: std::collections::HashMap<String, String>) -> Self {
        self.style.extend(style);
        self
    }
}

/// The page definition, with a title, and a list of blocks
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Page {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub path: String,
    pub url: Option<String>,
    pub template: Option<String>,
    #[serde(default)]
    pub children: Vec<Element>,
    #[serde(default)]
    pub cache: Vec<String>,
    pub last_modified: Option<String>,
    #[serde(default)]
    pub meta: Vec<MetaDefinition>,
    #[serde(default)]
    pub css_variables: std::collections::HashMap<String, String>,
}

impl Page {
    pub fn new() -> Self {
        Self {
            title: "".to_string(),
            path: "".to_string(),
            url: None,
            template: None,
            children: Vec::new(),
            cache: Vec::new(),
            last_modified: None,
            meta: Vec::new(),
            css_variables: std::collections::HashMap::new(),
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = path;
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_template(mut self, template: String) -> Self {
        self.template = Some(template);
        self
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    pub fn with_meta(mut self, meta: Vec<MetaDefinition>) -> Self {
        self.meta = meta;
        self
    }
}

/// A page info, with a title, and a url
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PageInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub store: String,
}

/// A page list result, with a list of pages and a total count
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct ResultPageList {
    pub count: usize,
    pub results: Vec<PageInfo>,
}

#[cfg(test)]
mod tests {
    use crate::page::types::{Element, MetaDefinition, Page};
    use std::collections::HashMap;

    #[test]
    fn test_page_creation() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string());

        assert_eq!(page.title, "Test Page");
        assert_eq!(page.path, "/test");
        assert!(page.url.is_none());
        assert!(page.template.is_none());
        assert!(page.children.is_empty());
        assert!(page.cache.is_empty());
        assert!(page.last_modified.is_none());
        assert!(page.meta.is_empty());
        assert!(page.css_variables.is_empty());
    }

    #[test]
    fn test_page_builder_pattern() {
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_url("https://example.com/test".to_string())
            .with_template("default".to_string())
            .with_meta(vec![MetaDefinition {
                name: "description".to_string(),
                content: "A test page".to_string(),
            }]);

        assert_eq!(page.url, Some("https://example.com/test".to_string()));
        assert_eq!(page.template, Some("default".to_string()));
        assert_eq!(page.meta.len(), 1);
        assert_eq!(page.meta[0].name, "description");
    }

    #[test]
    fn test_element_creation() {
        let element = Element::new(
            "div".to_string(),
            serde_json::json!({"text": "Hello"}),
            "test-div".to_string(),
        );

        assert_eq!(element.widget, "div");
        assert_eq!(element.data["text"], "Hello");
        assert_eq!(element.id, "test-div");
        assert!(element.children.is_empty());
        assert!(element.style.is_empty());
    }

    #[test]
    fn test_element_builder_pattern() {
        let mut style = HashMap::new();
        style.insert("color".to_string(), "red".to_string());

        let element = Element::new(
            "div".to_string(),
            serde_json::json!({"text": "Hello"}),
            "test-div".to_string(),
        )
        .with_style(style);

        assert_eq!(element.id, "test-div");
        assert_eq!(element.style["color"], "red");
    }

    #[test]
    fn test_page_with_elements() {
        let element = Element::new(
            "div".to_string(),
            serde_json::json!({"text": "Hello"}),
            "test-div".to_string(),
        );
        let page = Page::new()
            .with_title("Test Page".to_string())
            .with_path("/test".to_string())
            .with_children(vec![element]);

        assert_eq!(page.children.len(), 1);
        assert_eq!(page.children[0].widget, "div");
    }

    #[test]
    fn test_meta_definition() {
        let meta = MetaDefinition {
            name: "keywords".to_string(),
            content: "rust, page, viewer".to_string(),
        };

        assert_eq!(meta.name, "keywords");
        assert_eq!(meta.content, "rust, page, viewer");
    }

    #[test]
    fn test_read_page_definition_json() {
        let page_json = r#"
        {
            "title": "Test Page",
            "path": "/test",
            "url": "https://example.com/test",
            "template": "default",
            "meta": [{"name": "description", "content": "A test page"}]
        }
        "#;
        let page: Page = serde_json::from_str(page_json).unwrap();
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.path, "/test");
        assert_eq!(page.url, Some("https://example.com/test".to_string()));
        assert_eq!(page.template, Some("default".to_string()));
        assert_eq!(page.meta.len(), 1);
        assert_eq!(page.meta[0].name, "description");

        println!("Page: {:?}", page);
        println!("Page JSON: {}", serde_yaml::to_string(&page).unwrap());
    }
}
