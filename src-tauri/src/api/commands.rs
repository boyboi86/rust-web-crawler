use rust_web_crawler::config::WebCrawlerConfig;

use crate::actors::CrawlerBridge;
use crate::core::{CrawlRequest, CrawlStatus};
use crate::utils::validate_crawl_request;

/// Get default crawler configuration
#[tauri::command]
pub async fn get_default_config() -> Result<WebCrawlerConfig, String> {
    println!("🔧 get_default_config called");
    Ok(WebCrawlerConfig::default())
}

/// Validate crawler request (legacy command)
#[tauri::command]
pub async fn validate_crawl_request_api(request: CrawlRequest) -> Result<String, String> {
    println!(
        "🔍 validate_crawl_request_api called for session: {}",
        request.session_id
    );
    println!("📊 Received CrawlRequest details:");
    println!("  - base_url: {}", request.base_url);
    println!("  - user_agent: {:?}", request.user_agent);
    println!("  - proxy_pool: {:?}", request.proxy_pool);
    println!("  - retry_config: {:?}", request.retry_config);
    println!("  - accepted_languages: {:?}", request.accepted_languages);
    println!("  - min_word_length: {:?}", request.min_word_length);
    println!("📋 Full request: {:#?}", request);

    match validate_crawl_request(&request) {
        Ok(()) => {
            println!("✅ Configuration validation passed");
            Ok("Configuration is valid".to_string())
        }
        Err(e) => {
            println!("❌ Validation failed: {}", e);
            Err(e.to_string())
        }
    }
}

/// Initialize a new crawl session
#[tauri::command]
pub async fn start_crawl(
    request: CrawlRequest,
    bridge: tauri::State<'_, CrawlerBridge>,
) -> Result<String, String> {
    println!("🚀 start_crawl called for session: {}", request.session_id);
    println!("🔗 Base URL received: {}", request.base_url);
    println!("📊 Complete request data received:");
    println!("  - user_agent: {:?}", request.user_agent);
    println!("  - proxy_pool: {:?}", request.proxy_pool);
    println!("  - accepted_languages: {:?}", request.accepted_languages);
    println!("  - min_word_length: {:?}", request.min_word_length);
    println!("  - default_rate_limit: {:?}", request.default_rate_limit);
    println!("  - retry_config: {:?}", request.retry_config);
    println!("  - logging_config: {:?}", request.logging_config);
    println!("📋 Full request structure: {:#?}", request);

    // Validate request first
    if let Err(e) = validate_crawl_request(&request) {
        println!("❌ Validation failed: {}", e);
        return Err(e.to_string());
    }
    println!("✅ Config validation passed");

    // Use the bridge to start crawling
    bridge.start_crawl(request).await
}

/// Get current status of a crawl session
#[tauri::command]
pub async fn get_crawl_status(
    session_id: String,
    bridge: tauri::State<'_, CrawlerBridge>,
) -> Result<CrawlStatus, String> {
    println!("📊 get_crawl_status called for session: {}", session_id);

    match bridge.get_status(session_id.clone()).await {
        Ok(Some(status)) => {
            println!("✅ Status found for session: {}", session_id);
            Ok(status)
        }
        Ok(None) => {
            println!("❌ Session {} not found", session_id);
            Err(format!("Session {} not found", session_id))
        }
        Err(e) => {
            println!("❌ Error getting status for session {}: {}", session_id, e);
            Err(e)
        }
    }
}

/// Stop and clean up a crawl session
#[tauri::command]
pub async fn stop_crawl(
    session_id: String,
    bridge: tauri::State<'_, CrawlerBridge>,
) -> Result<String, String> {
    println!("🛑 stop_crawl called for session: {}", session_id);

    bridge.stop_crawl(session_id).await
}
