/// Simplified main.rs using the new session management system
///
/// This demonstrates how to use the refactored architecture with minimal boilerplate
use anyhow::Error;
use rust_web_crawler::{
    config::presets::create_production_session_config,
    session::CrawlSession,
    utils::{init_logging, log_session_summary},
};
use tracing::info;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    init_logging()?;

    info!("🚀 Rust Web Crawler - Refactored Production Mode");
    info!("==================================================");

    // Create session configuration using preset
    let session_config = create_production_session_config();
    info!("⚙️ Loaded production session configuration");

    // Create crawl session
    let session = CrawlSession::new(session_config).await?;
    info!("📝 Session ID: {}", session.session_id());

    // Define target URLs
    let target_urls = get_target_urls()?;
    info!("🎯 Target URLs: {}", target_urls.len());

    // Execute the crawl session
    let session_result = session.execute_crawl(target_urls).await?;

    // Log final statistics
    log_session_summary(&session_result);

    info!("✅ Crawl session completed successfully!");
    info!("📊 Results stored in configured storage location");

    Ok(())
}

/// Get target URLs for crawling
fn get_target_urls() -> Result<Vec<Url>, Error> {
    Ok(vec![
        Url::parse("https://www.bbc.com/news")?,
        Url::parse("https://httpbin.org/html")?,
        Url::parse("https://example.com")?,
        Url::parse("https://httpbin.org/json")?,
    ])
}
