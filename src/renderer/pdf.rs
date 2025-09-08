// (C) Coralbits SL 2025
// This file is part of Coralpages and is licensed under the
// GNU Affero General Public License v3.0.
// A commercial license on request is also available;
// contact info@coralbits.com for details.

use crate::renderer::renderedpage::RenderedPage;
use anyhow::Result;
use tracing::info;

// Use a headless chromium to render the given html to pdf
pub async fn render_pdf(page: &RenderedPage) -> Result<Vec<u8>> {
    let html = page.render_full_html_page();
    // let pdf = chromium::HTML(html.as_str()).to_pdf();
    let pdf_config = {
        // inside the {} to release the lock ASAP
        let config = crate::config::get_config().await;

        config
            .pdf
            .clone()
            .ok_or(anyhow::anyhow!("PDF generation not enabled"))?
    };
    // Create a temporary directory
    let temp_dir = pdf_config.temp_dir;
    std::fs::create_dir_all(&temp_dir)?;
    info!("Created temp directory: {}", &temp_dir);
    // get cwd
    let cwd = std::env::current_dir()?;
    info!("CWD: {}", cwd.display());

    // move to the temp directory
    std::env::set_current_dir(&temp_dir)?;

    // It writes the html data to a temp file
    let temp_file = format!("{}/page.html", &temp_dir);
    std::fs::write(&temp_file, html.as_bytes())?;
    info!("Wrote html to temp file: {}", &temp_file);

    // Runs the external process to render the pdf
    let pdf = tokio::process::Command::new(pdf_config.chromium_path)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--print-to-pdf")
        .arg("--no-pdf-header-footer")
        .arg(&temp_file)
        .output()
        .await?;

    let pdfpath = format!("{}/output.pdf", &temp_dir);
    let pdfdata = std::fs::read(&pdfpath)?;
    info!("Rendered pdf to stdout, length: {}", pdfdata.len());

    // removes the temp file
    std::fs::remove_file(&temp_file)?;
    std::fs::remove_file(&pdfpath)?;
    std::fs::remove_dir(&temp_dir)?;

    info!("Output PDF length: {}", pdfdata.len());
    info!("Stderr PDF: {:?}", String::from_utf8_lossy(&pdf.stderr));

    // move to the cwd
    std::env::set_current_dir(&cwd)?;
    info!("Moved to cwd: {}", &cwd.display());

    Ok(pdfdata)
}
