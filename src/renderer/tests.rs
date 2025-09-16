// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use std::collections::HashMap;

use minijinja::Environment;
use tracing::{debug, info};

use crate::{
    page::types::{CssClass, Element, MetaDefinition, Page, Widget},
    store::{factory::StoreFactory, traits::Store},
    utils::setup_logging,
    PageRenderer,
};
use ctor::ctor;

use super::renderedpage::{RenderedPage, RenderedingPageData};

#[ctor]
fn setup_logging_() {
    setup_logging(true);
}

// Helper function to parse YAML into Page object
fn parse_page_from_yaml(yaml: &str) -> anyhow::Result<Page> {
    // For now, implement basic YAML parsing or use serde_yaml
    // This is a placeholder implementation
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml)?;

    let mut page = Page::new();

    if let Some(title) = parsed.get("title").and_then(|v| v.as_str()) {
        page = page.with_title(title.to_string());
    }

    if let Some(path) = parsed.get("path").and_then(|v| v.as_str()) {
        page = page.with_path(path.to_string());
    }

    if let Some(children) = parsed.get("children").and_then(|v| v.as_sequence()) {
        let mut elements = Vec::new();
        for child in children {
            if let Ok(element) = parse_element_from_yaml_value(child) {
                elements.push(element);
            }
        }
        page = page.with_children(elements);
    }

    if let Some(meta) = parsed.get("meta").and_then(|v| v.as_sequence()) {
        let mut meta_defs = Vec::new();
        for meta_item in meta {
            if let Some(name) = meta_item.get("name").and_then(|v| v.as_str()) {
                if let Some(content) = meta_item.get("content").and_then(|v| v.as_str()) {
                    meta_defs.push(MetaDefinition {
                        name: name.to_string(),
                        content: content.to_string(),
                    });
                }
            }
        }
        page = page.with_meta(meta_defs);
    }

    Ok(page)
}

fn parse_element_from_yaml_value(yaml: &serde_yaml::Value) -> anyhow::Result<Element> {
    let widget = yaml
        .get("widget")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Widget field required"))?;

    let id = yaml
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut data = HashMap::new();
    if let Some(data_obj) = yaml.get("data").and_then(|v| v.as_mapping()) {
        for (k, v) in data_obj {
            if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                data.insert(key.to_string(), value.to_string());
            }
        }
    }

    let mut element = Element::new(widget.to_string(), data, id);

    if let Some(classes) = yaml.get("classes").and_then(|v| v.as_sequence()) {
        let class_list: Vec<String> = classes
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        element = element.with_classes(class_list);
    }

    if let Some(style) = yaml.get("style").and_then(|v| v.as_mapping()) {
        let mut style_map = HashMap::new();
        for (k, v) in style {
            if let (Some(key), Some(value)) = (k.as_str(), v.as_str()) {
                style_map.insert(key.to_string(), value.to_string());
            }
        }
        element = element.with_style(style_map);
    }

    if let Some(children) = yaml.get("children").and_then(|v| v.as_sequence()) {
        let mut child_elements = Vec::new();
        for child in children {
            if let Ok(child_element) = parse_element_from_yaml_value(child) {
                child_elements.push(child_element);
            }
        }
        element = element.with_children(child_elements);
    }

    Ok(element)
}

fn create_test_widget(name: &str, html: &str, css: &str) -> Widget {
    Widget {
        name: name.to_string(),
        html: html.to_string(),
        css: css.to_string(),
        editor: vec![],
        description: format!("Test widget: {}", name),
        icon: "".to_string(),
    }
}

// Enhanced test store with configurable widgets and CSS classes
struct TestStore {
    widgets: HashMap<String, Widget>,
    css_classes: HashMap<String, LocalCssClass>,
    should_error: bool,
}

#[derive(Clone)]
struct LocalCssClass {
    name: String,
    css: String,
}

impl TestStore {
    fn new() -> Self {
        let mut store = Self {
            widgets: HashMap::new(),
            css_classes: HashMap::new(),
            should_error: false,
        };

        // Add default test widgets
        store.add_widget(
            "text",
            "<a class=\"test-link\" id=\"{{data.id}}\">Hello, {{data.text}}!</a>",
            ".test-link { background: red; }",
        );
        store.add_widget("columns", "<div class=\"columns column-{{data.id}}\" id=\"{{data.id}}\">{{context.children|join('')}}</div>", ".columns { display: flex; }");
        store.add_widget(
            "section",
            "<section id=\"{{data.id}}\">{{context.children|join('')}}</section>",
            "section { padding: 1rem; }",
        );
        store.add_widget("invalid_template", "<div>{{invalid.syntax.here}}</div>", "");

        // Add default CSS classes
        store.add_css_class("primary", "color: blue; font-weight: bold;");
        store.add_css_class("secondary", "color: gray; font-size: 0.9em;");

        store
    }

    fn add_widget(&mut self, name: &str, html: &str, css: &str) {
        self.widgets
            .insert(name.to_string(), create_test_widget(name, html, css));
    }

    fn add_css_class(&mut self, name: &str, css: &str) {
        self.css_classes.insert(
            name.to_string(),
            LocalCssClass {
                name: name.to_string(),
                css: css.to_string(),
            },
        );
    }
}

#[async_trait::async_trait]
impl Store for TestStore {
    fn name(&self) -> &str {
        "test"
    }

    async fn load_widget_definition(&self, path: &str) -> anyhow::Result<Option<Widget>> {
        debug!("Loading widget definition from path: {}", path);

        if self.should_error && path == "error_widget" {
            return Err(anyhow::anyhow!("Simulated widget loading error"));
        }

        // Handle path prefixes
        let widget_name = if path.contains('/') {
            path.split('/').last().unwrap_or(path)
        } else {
            path
        };

        Ok(self.widgets.get(widget_name).cloned())
    }

    async fn load_css_class_definition(&self, name: &str) -> anyhow::Result<Option<CssClass>> {
        Ok(self.css_classes.get(name).map(|class| CssClass {
            name: class.name.clone(),
            description: format!("Test CSS class: {}", class.name),
            css: class.css.clone(),
            tags: vec![],
        }))
    }
}

// Assertion helpers

fn assert_html_structure(html: &str, expected_substring: &str) {
    assert!(
        html.contains(expected_substring),
        "HTML does not contain expected structure: {}\nActual HTML: {}",
        expected_substring,
        html
    );
}

fn assert_meta_tags(rendered: &RenderedPage, expected: &[MetaDefinition]) {
    assert_eq!(
        rendered.meta.len(),
        expected.len(),
        "Meta tag count mismatch"
    );
    for (i, expected_meta) in expected.iter().enumerate() {
        let actual_meta = &rendered.meta[i];
        assert_eq!(actual_meta.name, expected_meta.name);
        assert_eq!(actual_meta.content, expected_meta.content);
    }
}

// T001 - Very basic page render (✅ Existing test moved)
#[tokio::test]
async fn test_basic_page_render() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, world!".to_string())]),
            "test-link".to_string(),
        )
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, child!".to_string())]),
            "test-link-child".to_string(),
        )])]);

    let mut store = StoreFactory::new();
    store.add_store(Box::new(TestStore::new()));

    let env = Environment::new();
    let mut rendered_page = RenderedingPageData::new(&page, &store, &env);

    let ctx = minijinja::context! {};
    rendered_page.render(&ctx).await.unwrap();

    assert!(!rendered_page.rendered_page.body.is_empty());
    assert_eq!(rendered_page.rendered_page.title, "Test Page");
    assert_eq!(rendered_page.rendered_page.path, "/test");

    info!("T001 - Basic page render: PASSED");
}

// T002 - Render page with section with children (✅ Existing test moved)
#[tokio::test]
async fn test_page_with_nested_children() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/columns".to_string(),
            HashMap::from([
                ("wrap".to_string(), "true".to_string()),
                ("gap".to_string(), "12".to_string()),
            ]),
            "test-columns".to_string(),
        )
        .with_children(vec![
            Element::new(
                "test/text".to_string(),
                HashMap::from([("text".to_string(), "Column 1".to_string())]),
                "test-link-1".to_string(),
            ),
            Element::new(
                "test/text".to_string(),
                HashMap::from([("text".to_string(), "Column 2".to_string())]),
                "test-link-2".to_string(),
            ),
        ])]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let expected_html = "<div class=\"columns column-test-columns\" id=\"test-columns\"><a class=\"test-link\" id=\"test-link-1\">Hello, Column 1!</a><a class=\"test-link\" id=\"test-link-2\">Hello, Column 2!</a></div>";
    assert_eq!(rendered_page.body, expected_html);

    info!("T002 - Nested children render: PASSED");
}

// T003 - Render with styles (✅ Existing test moved)
#[tokio::test]
async fn test_page_with_custom_styles() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, world!".to_string())]),
            "test-link-id".to_string(),
        )
        .with_style(HashMap::from([(
            "background".to_string(),
            "red".to_string(),
        )]))]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let css = rendered_page.get_css();
    let from_widget_css = ".test-link { background: red; }";
    let from_element_style = "#test-link-id {\n background: red;\n }";

    assert!(css.contains(from_widget_css));
    assert!(css.contains(from_element_style));

    info!("T003 - Custom styles render: PASSED");
}

// T004 - Render with custom classes
#[tokio::test]
async fn test_page_with_custom_classes() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, world!".to_string())]),
            "test-link".to_string(),
        )
        .with_classes(vec![
            "test/primary".to_string(),
            "test/secondary".to_string(),
        ])]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let css = rendered_page.get_css();
    assert!(css.contains("color: blue; font-weight: bold;"));
    assert!(css.contains("color: gray; font-size: 0.9em;"));

    info!("T004 - Custom classes render: PASSED");
}

// T005 - Error handling - Missing widget
#[tokio::test]
async fn test_missing_widget_error() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/nonexistent_widget".to_string(),
            HashMap::new(),
            "test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let result = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Widget not found"));

    info!("T005 - Missing widget error handling: PASSED");
}

// T006 - Error handling - Template rendering errors
#[tokio::test]
async fn test_template_rendering_errors() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/invalid_template".to_string(),
            HashMap::new(),
            "test".to_string(),
        )]);

    // Test debug mode - should show error in HTML
    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, true)
        .await
        .unwrap();

    assert!(rendered_page.body.contains("<pre style=\"color:red;\">"));
    assert!(!rendered_page.errors.is_empty());

    // Test non-debug mode - should return error
    let result = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await;

    assert!(result.is_err());

    info!("T006 - Template rendering error handling: PASSED");
}

// T007 - Meta data handling
#[tokio::test]
async fn test_meta_data_handling() {
    let meta_defs = vec![
        MetaDefinition {
            name: "description".to_string(),
            content: "Test page description".to_string(),
        },
        MetaDefinition {
            name: "keywords".to_string(),
            content: "test, page".to_string(),
        },
    ];

    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_meta(meta_defs.clone())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello".to_string())]),
            "test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    assert_meta_tags(&rendered_page, &meta_defs);

    info!("T007 - Meta data handling: PASSED");
}

// T008 - Data context templating
#[tokio::test]
async fn test_data_context_templating() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, {{context.name}}!".to_string())]),
            "test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let ctx = minijinja::context! { name => "World" };
    let rendered_page = renderer.render_page(&page, &ctx, false).await.unwrap();

    assert_html_structure(&rendered_page.body, "Hello, World!");

    info!("T008 - Data context templating: PASSED");
}

// T010 - Full HTML page generation
#[tokio::test]
async fn test_full_html_page_generation() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello, world!".to_string())]),
            "test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let full_html = rendered_page.render_full_html_page();

    assert!(full_html.contains("<!DOCTYPE html>"));
    assert!(full_html.contains("<html>"));
    assert!(full_html.contains("<head>"));
    assert!(full_html.contains("<meta name=\"viewport\""));
    assert!(full_html.contains("<style>"));
    assert!(full_html.contains("<body>"));
    assert!(full_html.contains("Hello, world!"));

    info!("T010 - Full HTML page generation: PASSED");
}

// T011 - CSS variable generation
#[tokio::test]
async fn test_css_variable_generation() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![
            Element::new(
                "test/text".to_string(),
                HashMap::from([("text".to_string(), "Text 1".to_string())]),
                "test1".to_string(),
            )
            .with_style(HashMap::from([("color".to_string(), "red".to_string())])),
            Element::new(
                "test/section".to_string(),
                HashMap::new(),
                "test2".to_string(),
            )
            .with_style(HashMap::from([("padding".to_string(), "2rem".to_string())])),
        ]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let css = rendered_page.get_css();

    // Check that CSS is sorted and properly formatted
    assert!(css.contains("#test1 {\n color: red;\n }"));
    assert!(css.contains("#test2 {\n padding: 2rem;\n }"));
    assert!(css.contains(".test-link { background: red; }"));
    assert!(css.contains("section { padding: 1rem; }"));

    info!("T011 - CSS variable generation: PASSED");
}

// T012 - Complex nested rendering
#[tokio::test]
async fn test_complex_nested_rendering() {
    let yaml_content = r#"
title: "Complex Page"
path: "/complex"
children:
  - widget: "test/section"
    id: "main-section"
    children:
      - widget: "test/columns"
        id: "main-columns"
        data:
          gap: "16"
        children:
          - widget: "test/text"
            id: "col1"
            data:
              text: "Column 1"
          - widget: "test/section"
            id: "col2-section"
            children:
              - widget: "test/text"
                id: "nested-text"
                data:
                  text: "Nested content"
"#;

    let page = parse_page_from_yaml(yaml_content).unwrap();

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    // Verify the complex nested structure is rendered correctly
    assert_html_structure(&rendered_page.body, "<section id=\"main-section\">");
    assert_html_structure(
        &rendered_page.body,
        "<div class=\"columns column-main-columns\" id=\"main-columns\">",
    );
    assert_html_structure(&rendered_page.body, "Column 1");
    assert_html_structure(&rendered_page.body, "<section id=\"col2-section\">");
    assert_html_structure(&rendered_page.body, "Nested content");

    info!("T012 - Complex nested rendering: PASSED");
}

// T013 - Widget CSS integration
#[tokio::test]
async fn test_widget_css_integration() {
    let mut test_store = TestStore::new();
    test_store.add_widget(
        "custom",
        "<div class=\"custom\">{{data.content}}</div>",
        ".custom { border: 1px solid blue; background: yellow; }",
    );

    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/custom".to_string(),
            HashMap::from([("content".to_string(), "Custom content".to_string())]),
            "custom-element".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(test_store));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let css = rendered_page.get_css();
    assert!(css.contains("border: 1px solid blue"));
    assert!(css.contains("background: yellow"));

    info!("T013 - Widget CSS integration: PASSED");
}

// T015 - Debug mode behavior
#[tokio::test]
async fn test_debug_mode_behavior() {
    let mut test_store = TestStore::new();
    test_store.add_widget("error_widget", "<div>{{nonexistent.variable}}</div>", "");

    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/error_widget".to_string(),
            HashMap::new(),
            "error-test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(test_store));

    // Test debug mode - should show error in red box
    let rendered_page_debug = renderer
        .render_page(&page, &minijinja::context! {}, true)
        .await
        .unwrap();

    assert!(rendered_page_debug
        .body
        .contains("<pre style=\"color:red;\">"));
    assert!(!rendered_page_debug.errors.is_empty());

    // Test non-debug mode - should return error
    let result_no_debug = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await;

    assert!(result_no_debug.is_err());

    info!("T015 - Debug mode behavior: PASSED");
}

// T018 - Response codes and headers
#[tokio::test]
async fn test_response_codes_and_headers() {
    let page = Page::new()
        .with_title("Test Page".to_string())
        .with_path("/test".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Hello".to_string())]),
            "test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    // Test default response code
    assert_eq!(rendered_page.response_code, 200);

    // Test that headers are initialized (empty by default)
    assert!(rendered_page.headers.is_empty());

    info!("T018 - Response codes and headers: PASSED");
}

// Helper test for YAML parsing functionality
#[tokio::test]
async fn test_yaml_page_parsing() {
    let yaml_content = r#"
title: "YAML Test Page"
path: "/yaml-test"
meta:
  - name: "description"
    content: "A test page parsed from YAML"
children:
  - widget: "test/text"
    id: "yaml-text"
    data:
      text: "Parsed from YAML"
    style:
      color: "blue"
    classes: ["test/primary"]
"#;

    let page = parse_page_from_yaml(yaml_content).unwrap();

    assert_eq!(page.title, "YAML Test Page");
    assert_eq!(page.path, "/yaml-test");
    assert_eq!(page.meta.len(), 1);
    assert_eq!(page.meta[0].name, "description");
    assert_eq!(page.children.len(), 1);

    let element = &page.children[0];
    assert_eq!(element.widget, "test/text");
    assert_eq!(element.id, "yaml-text");
    assert_eq!(element.data.get("text").unwrap(), "Parsed from YAML");
    assert!(element.classes.contains(&"test/primary".to_string()));
    assert_eq!(element.style.get("color").unwrap(), "blue");

    info!("YAML parsing helper: PASSED");
}

#[tokio::test]
async fn test_performance_timing() {
    let start = std::time::Instant::now();

    let page = Page::new()
        .with_title("Performance Test".to_string())
        .with_path("/perf".to_string())
        .with_children(vec![Element::new(
            "test/text".to_string(),
            HashMap::from([("text".to_string(), "Performance test".to_string())]),
            "perf-test".to_string(),
        )]);

    let mut renderer = PageRenderer::new();
    renderer.store.add_store(Box::new(TestStore::new()));

    let rendered_page = renderer
        .render_page(&page, &minijinja::context! {}, false)
        .await
        .unwrap();

    let elapsed = start.elapsed();

    // Verify that elapsed time tracking exists
    assert!(rendered_page.elapsed.elapsed() <= elapsed);

    info!("Performance timing test: PASSED (took {:?})", elapsed);
}
