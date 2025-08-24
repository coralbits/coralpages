use anyhow::Result;
use clap::Parser;
use minijinja::context;
use page_viewer::{
    page::Page,
    renderer::renderer::PageRenderer,
    store::{file::FileStore, traits::Store},
};
use std::{fs, time::Instant};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "page-viewer")]
#[command(about = "Page Viewer - Rust Edition")]
struct Args {
    /// Render a YAML page file and output the rendered body to stdout
    #[arg(long, value_name = "FILENAME")]
    render_file: Option<String>,
    /// Render all pages in the given directory
    #[arg(long, value_name = "FILENAME")]
    render_from_store: Option<String>,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // trace to stderr
        .with_writer(std::io::stderr)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    if let Some(filename) = args.render_file {
        // Read and render the YAML file
        info!("Rendering page file: {}", filename);
        let start = Instant::now();
        render_page_file(&filename)?;
        let duration = start.elapsed();
        info!("Rendered page file in {:?}", duration);
    } else if let Some(pagename) = args.render_from_store {
        // Read and render the YAML file
        info!("Rendering page from store: {}", pagename);
        let start = Instant::now();
        render_from_store(&pagename)?;
        let duration = start.elapsed();
        info!("Rendered page file in {:?}", duration);
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

    let mut renderer = PageRenderer::new();

    renderer
        .store
        .add_store("builtin", Box::new(FileStore::new("builtin/widgets")?));

    // Create a RenderedPage and render it
    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx)?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}

fn render_from_store(pagename: &str) -> Result<()> {
    let mut renderer = PageRenderer::new();

    renderer
        .store
        .add_store("builtin", Box::new(FileStore::new("builtin/widgets")?));
    renderer
        .store
        .add_store("pages", Box::new(FileStore::new("builtin/pages")?));

    let page = renderer
        .store
        .load_page_definition(pagename)?
        .ok_or_else(|| anyhow::anyhow!("Page '{}' not found", pagename))?;

    info!("Loaded page: {:?}", page);

    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx)?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}
