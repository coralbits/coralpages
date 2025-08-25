use anyhow::Result;
use clap::Parser;
use minijinja::context;
use page_viewer::file::FileStore;
use page_viewer::traits::Store;
use page_viewer::{utils, Page, PageRenderer};
use std::fs;
use std::time::Instant;
use tracing::info;

use page_viewer::server::start;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Render a single YAML page file
    #[arg(long, value_name = "FILENAME")]
    render_file: Option<String>,
    /// Render all pages in the given directory
    #[arg(long, value_name = "FILENAME")]
    render_from_store: Option<String>,
    #[arg(short, long, value_name = "LISTEN", default_value = "0.0.0.0:8006")]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::setup_logging();

    let args = Args::parse();

    if let Some(filename) = args.render_file {
        // Read and render the YAML file
        info!("Rendering page file: {}", filename);
        let start = Instant::now();
        render_page_file(&filename).await?;
        let duration = start.elapsed();
        info!("Rendered page file in {:?}", duration);
    } else if let Some(pagename) = args.render_from_store {
        // Read and render the YAML file
        info!("Rendering page from store: {}", pagename);
        let start = Instant::now();
        render_from_store(&pagename).await?;
        let duration = start.elapsed();
        info!("Rendered page file in {:?}", duration);
    } else if let Some(listen) = args.listen {
        // Start the server
        info!("Starting server on {}", listen);
        start_server(&listen).await?;
    } else {
        // Default behavior - show info about the tool
        println!("Page Viewer - Rust Edition");
        println!("Use --render <filename> to render a YAML page file");
    }

    Ok(())
}

async fn render_page_file(filename: &str) -> Result<()> {
    // Read the YAML file
    let yaml_content = fs::read_to_string(filename)?;

    // Deserialize the YAML into a Page
    let page: Page = serde_yaml::from_str(&yaml_content)?;

    let mut renderer = PageRenderer::new();

    renderer
        .store
        .add_store("builtin", Box::new(FileStore::new("builtin/widgets")?));

    // Create a RenderedPage and render it
    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx).await?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}

async fn render_from_store(pagename: &str) -> Result<()> {
    let mut renderer = PageRenderer::new();

    renderer
        .store
        .add_store("builtin", Box::new(FileStore::new("builtin/widgets")?));
    renderer
        .store
        .add_store("pages", Box::new(FileStore::new("builtin/pages")?));

    let page = renderer
        .store
        .load_page_definition(pagename)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Page '{}' not found", pagename))?;

    info!("Loaded page: {:?}", page.path);

    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx).await?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}

async fn start_server(listen: &str) -> Result<()> {
    info!("Starting server on http://{}", listen);
    start(listen).await?;
    Ok(())
}
