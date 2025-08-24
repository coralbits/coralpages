use anyhow::Result;
use clap::Parser;
use page_viewer::page::{Page, RenderedPage};
use std::fs;

#[derive(Parser)]
#[command(name = "page-viewer")]
#[command(about = "Page Viewer - Rust Edition")]
struct Args {
    /// Render a YAML page file and output the rendered body to stdout
    #[arg(long, value_name = "FILENAME")]
    render: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(filename) = args.render {
        // Read and render the YAML file
        render_page_file(&filename)?;
    } else {
        // Default behavior - show info about the tool
        println!("Page Viewer - Rust Edition");
        println!("Use --render <filename> to render a YAML page file");
    }

    Ok(())
}

fn render_page_file(filename: &str) -> Result<()> {
    // Read the YAML file
    let yaml_content = fs::read_to_string(filename)?;

    // Deserialize the YAML into a Page
    let page: Page = serde_yaml::from_str(&yaml_content)?;

    // Create a RenderedPage and render it
    let mut rendered_page = RenderedPage::new(page);
    rendered_page.render()?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body());

    Ok(())
}
