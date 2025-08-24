use page_viewer::page::{Element, Page};

fn main() {
    println!("Page Viewer - Rust Edition");

    // Create a simple page
    let element = Element::new(
        "div".to_string(),
        serde_json::json!({
            "text": "Hello from Rust!"
        }),
    );

    let page =
        Page::new("Welcome".to_string(), "/welcome".to_string()).with_children(vec![element]);

    println!("Created page: {}", page.title);
    println!("Page path: {}", page.path);
    println!("Number of elements: {}", page.children.len());
}
