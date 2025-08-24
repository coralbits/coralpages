use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A meta definition for page metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaDefinition {
    pub name: String,
    pub content: String,
}

/// Each block definition, with content, and maybe more children
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub element_type: String,
    pub data: serde_json::Value,
    pub id: Option<String>,
    pub children: Vec<Element>,
    pub style: std::collections::HashMap<String, String>,
}

impl Element {
    pub fn new(element_type: String, data: serde_json::Value) -> Self {
        Self {
            element_type,
            data,
            id: None,
            children: Vec::new(),
            style: std::collections::HashMap::new(),
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    pub fn with_style(mut self, style: std::collections::HashMap<String, String>) -> Self {
        self.style = style;
        self
    }
}

/// The page definition, with a title, and a list of blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub title: String,
    pub path: String,
    pub url: Option<String>,
    pub template: Option<String>,
    #[serde(default)]
    pub children: Vec<Element>,
    #[serde(default)]
    pub cache: Vec<String>,
    pub last_modified: Option<DateTime<Utc>>,
    #[serde(default)]
    pub meta: Vec<MetaDefinition>,
    #[serde(default)]
    pub css_variables: std::collections::HashMap<String, String>,
}

impl Page {
    pub fn new(title: String, path: String) -> Self {
        Self {
            title,
            path,
            url: None,
            template: None,
            children: Vec::new(),
            cache: Vec::new(),
            last_modified: None,
            meta: Vec::new(),
            css_variables: std::collections::HashMap::new(),
        }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub store: String,
}

/// A page list result, with a list of pages and a total count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageListResult {
    pub count: usize,
    pub results: Vec<PageInfo>,
}
