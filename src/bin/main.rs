/// Simplified main.rs using the new session management system
///
/// This demonstrates how to use the refactored architecture with minimal boilerplate
use anyhow::Error;
use rust_web_crawler::{
    config::presets::create_production_session_config, logging::init_logging, session::CrawlSession,
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
    info!("=== Crawl Session Summary ===");
    info!(
        "Total URLs processed: {}",
        session_result.total_urls_processed
    );
    info!("Successful crawls: {}", session_result.successful_crawls);
    info!("Failed crawls: {}", session_result.failed_crawls);
    if session_result.total_urls_processed > 0 {
        let success_rate = (session_result.successful_crawls as f64
            / session_result.total_urls_processed as f64)
            * 100.0;
        info!("Success rate: {:.1}%", success_rate);
    }
    info!(
        "Session duration: {:.2}s",
        session_result.total_duration.as_secs_f64()
    );

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
