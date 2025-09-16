// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

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
            std::collections::HashMap::from([("text".to_string(), "Hello".to_string())]),
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
            std::collections::HashMap::from([("text".to_string(), "Hello".to_string())]),
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
            std::collections::HashMap::from([("text".to_string(), "Hello".to_string())]),
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
