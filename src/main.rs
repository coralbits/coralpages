use anyhow::Result;
use clap::Parser;
use minijinja::context;
use page_viewer::traits::Store;
use page_viewer::{cache, config, utils, Page, PageRenderer, RestartManager};
use std::fs;
use std::time::Instant;
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

use page_viewer::config::{get_config, load_config, watch_config};
use page_viewer::server::start;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(long, value_name = "CONFIG_FILE", default_value = "config.yaml")]
    config: String,
    /// Render a single YAML page file
    #[arg(long, value_name = "FILENAME")]
    render_file: Option<String>,
    /// Render all pages in the given directory
    #[arg(long, value_name = "FILENAME")]
    render_from_store: Option<String>,
    #[arg(long, value_name = "LISTEN", default_value = "0.0.0.0:8006")]
    listen: Option<String>,
    #[arg(long, default_value = "false")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    info!("Loading configuration from: {}", args.config);
    load_config(&args.config).await?;

    // Start watching the config file for changes
    info!("Starting config file watcher for: {}", args.config);
    watch_config(&args.config).await?;

    let debug = if args.verbose {
        true
    } else {
        config::get_debug().await
    };
    utils::setup_logging(debug);

    {
        if let Some(cache) = get_config().await.cache.as_ref() {
            cache::set_cache(&cache.backend, &cache.url).await?;
        }
    }

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
        // Start the server with restart capability
        start_server_with_restart(&listen).await?;
    } else {
        let server = {
            let config = get_config().await;
            config.server.clone()
        };
        start_server_with_restart(&format!("{}:{}", server.host, server.port)).await?;
    }

    Ok(())
}

async fn render_page_file(filename: &str) -> Result<()> {
    // Read the YAML file
    let yaml_content = fs::read_to_string(filename)?;

    // Deserialize the YAML into a Page
    let page: Page = serde_yaml::from_str(&yaml_content)?;

    let config = get_config().await;
    let renderer = PageRenderer::new().with_stores(&config.stores).await?;

    // Create a RenderedPage and render it
    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx, false).await?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}

async fn render_from_store(pagename: &str) -> Result<()> {
    let config = get_config().await;
    let renderer = PageRenderer::new().with_stores(&config.stores).await?;

    let page = renderer
        .store
        .load_page_definition(pagename)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Page '{}' not found", pagename))?;

    info!("Loaded page: {:?}", page.path);

    let ctx = context! {};
    let rendered_page = renderer.render_page(&page, &ctx, false).await?;

    // Print the rendered body to stdout
    print!("{}", rendered_page.body);

    Ok(())
}

async fn start_server(listen: &str) -> Result<()> {
    let renderer = {
        let config = get_config().await;
        PageRenderer::new().with_stores(&config.stores).await?
    };

    info!("Starting server on http://{}", listen);
    info!("OpenAPI docs: http://{}/docs", listen);
    start(listen, renderer).await?;
    Ok(())
}

async fn start_server_with_restart(listen: &str) -> Result<()> {
    let restart_manager = RestartManager::new(listen.to_string());

    // Set up signal handlers
    restart_manager.enable_restart_with_signal(SignalKind::hangup())?;

    // Run the server with restart capability
    restart_manager
        .run_with_restart(move |listen_addr, shutdown_rx| async move {
            let renderer = {
                let config = get_config().await;
                PageRenderer::new().with_stores(&config.stores).await?
            };

            info!("Starting server on http://{}", listen_addr);
            info!("OpenAPI docs: http://{}/docs", listen_addr);

            // Use the new start_with_shutdown function
            page_viewer::server::start_with_shutdown(&listen_addr, renderer, shutdown_rx).await?;
            Ok(())
        })
        .await?;

    Ok(())
}
