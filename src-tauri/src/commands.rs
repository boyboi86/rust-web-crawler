use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Import our web crawler library
use rust_web_crawler::config::WebCrawlerConfig;
use rust_web_crawler::crawler::WebCrawler;

// Application state for managing crawler instances
#[derive(Default)]
pub struct AppState {
    crawlers: Arc<Mutex<HashMap<String, WebCrawler>>>,
}

// Data structures for frontend communication
#[derive(Serialize, Deserialize)]
pub struct CrawlRequest {
    pub id: String,
    pub url: String,
    pub config: WebCrawlerConfig,
}

#[derive(Serialize, Deserialize)]
pub struct CrawlStatus {
    pub id: String,
    pub status: String,
    pub pages_crawled: u32,
    pub total_pages: u32,
    pub current_url: Option<String>,
    pub errors: Vec<String>,
}

// Tauri commands (API endpoints for frontend)
#[tauri::command]
pub async fn get_default_config() -> Result<WebCrawlerConfig, String> {
    Ok(WebCrawlerConfig::default())
}

#[tauri::command]
pub async fn validate_config(config: WebCrawlerConfig) -> Result<String, String> {
    // Basic validation
    if config.base_url.is_empty() {
        return Err("Base URL is required".to_string());
    }

    if config.max_total_urls == 0 {
        return Err("Max total URLs must be greater than 0".to_string());
    }

    if config.max_crawl_depth == 0 {
        return Err("Max crawl depth must be greater than 0".to_string());
    }

    Ok("Configuration is valid".to_string())
}

#[tauri::command]
pub async fn start_crawl(
    request: CrawlRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut crawlers = state.crawlers.lock().await;

    // Create new crawler instance with proper parameters
    let crawler = WebCrawler::new(
        request.config,
        10, // max_concurrent_requests
        5,  // max_depth
    )
    .map_err(|e| format!("Failed to create crawler: {}", e))?;

    crawlers.insert(request.id.clone(), crawler);

    Ok(format!("Crawler {} initialized successfully", request.id))
}

#[tauri::command]
pub async fn get_crawl_status(
    crawler_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<CrawlStatus, String> {
    let crawlers = state.crawlers.lock().await;

    if crawlers.contains_key(&crawler_id) {
        // In a real implementation, you'd get actual status from the crawler
        Ok(CrawlStatus {
            id: crawler_id,
            status: "running".to_string(),
            pages_crawled: 0,
            total_pages: 0,
            current_url: None,
            errors: vec![],
        })
    } else {
        Err(format!("Crawler {} not found", crawler_id))
    }
}

#[tauri::command]
pub async fn stop_crawl(
    crawler_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut crawlers = state.crawlers.lock().await;

    if crawlers.remove(&crawler_id).is_some() {
        Ok(format!("Crawler {} stopped successfully", crawler_id))
    } else {
        Err(format!("Crawler {} not found", crawler_id))
    }
}
