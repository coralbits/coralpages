// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

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
pub struct WidgetEditorOption {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct WidgetEditor {
    #[serde(rename = "type")]
    #[oai(rename = "type")]
    pub editor_type: String,
    pub label: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub options: Vec<WidgetEditorOption>,
}

impl WidgetEditor {
    pub fn new() -> Self {
        WidgetEditor {
            editor_type: "".to_string(),
            label: "".to_string(),
            name: "".to_string(),
            placeholder: "".to_string(),
            options: Vec::new(),
        }
    }

    pub fn with_editor_type(mut self, editor_type: String) -> Self {
        self.editor_type = editor_type;
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = label;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct WidgetResults {
    pub count: usize,
    pub results: Vec<Widget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct IdName {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct StoreListResults {
    pub count: usize,
    pub results: Vec<IdName>,
}

/// A meta definition for page metadata
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct MetaDefinition {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct LinkDefinition {
    pub href: String,
    pub rel: String,
}
/// Each widget use in a page, with content, and maybe more children
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct Element {
    #[serde(default)]
    #[oai(default)]
    pub id: String,
    #[oai(default)]
    pub widget: String,
    #[serde(default)]
    #[oai(default)]
    pub data: std::collections::HashMap<String, String>,
    #[serde(default)]
    #[oai(default)]
    pub children: Vec<Element>,
    #[serde(default)]
    #[oai(default)]
    pub style: std::collections::HashMap<String, String>,
    #[serde(default)]
    #[oai(default)]
    pub classes: Vec<String>,
}

impl Element {
    pub fn new(
        widget: String,
        data: std::collections::HashMap<String, String>,
        id: String,
    ) -> Self {
        Self {
            id,
            widget,
            data,
            children: Vec::new(),
            style: std::collections::HashMap::new(),
            classes: Vec::new(),
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

    pub fn with_classes(mut self, classes: Vec<String>) -> Self {
        self.classes = classes;
        self
    }

    // Post read fix element, and recursively fix children.
    pub fn fix(mut self) -> Self {
        // check if the id is valid
        if self.id.is_empty() {
            self.id = uuid::Uuid::new_v4().to_string();
            // id can not start with a number
            if self.id.chars().next().unwrap().is_digit(10) {
                self.id = "id_".to_string() + &self.id;
            }
        }
        self.children = self.children.into_iter().map(|child| child.fix()).collect();

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
    #[serde(default)]
    pub store: String,
    pub url: Option<String>,
    pub template: Option<String>,
    #[serde(default)]
    pub children: Vec<Element>,
    #[serde(default)]
    pub cache: Vec<String>,
    pub last_modified: Option<String>,
    #[serde(default)]
    pub head: Option<PageHead>,
    #[serde(default)]
    pub css_variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct PageHead {
    pub meta: Option<Vec<MetaDefinition>>,
    pub link: Option<Vec<LinkDefinition>>,
}

impl PageHead {
    pub fn new() -> Self {
        Self {
            meta: None,
            link: None,
        }
    }
}

impl Page {
    pub fn new() -> Self {
        Self {
            title: "".to_string(),
            path: "".to_string(),
            store: "".to_string(),
            url: None,
            template: None,
            children: Vec::new(),
            cache: Vec::new(),
            last_modified: None,
            head: None,
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

    pub fn with_head(mut self, head: PageHead) -> Self {
        self.head = Some(head);
        self
    }

    pub fn fix(mut self) -> Self {
        self.children = self.children.into_iter().map(|child| child.fix()).collect();
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

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CssClass {
    pub name: String,
    pub description: String,
    pub css: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CssClassResult {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
pub struct CssClassResults {
    pub count: usize,
    pub results: Vec<CssClassResult>,
}
