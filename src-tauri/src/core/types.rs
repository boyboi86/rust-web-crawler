use serde::{Deserialize, Serialize};

/// Request structure matching the frontend form
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrawlRequest {
    pub session_id: String,
    pub base_url: String,
    pub max_total_urls: u32,
    pub max_crawl_depth: u32,
    pub enable_discovery_crawling: bool,
    pub enable_keyword_filtering: bool,
    pub target_words: Vec<String>,
    pub enable_content_filtering: bool,
    pub avoid_url_extensions: Vec<String>,
    pub enable_language_filtering: bool,
    pub latin_word_filter: bool,
    pub match_strategy: String, // "any" or "all"
}

/// Status structure for frontend display
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrawlStatus {
    pub session_id: String,
    pub status: String, // "idle", "initialized", "running", "completed", "error"
    pub total_urls_processed: usize,
    pub successful_crawls: usize,
    pub failed_crawls: usize,
    pub current_url: Option<String>,
    pub errors: Vec<String>,
    pub results: Vec<CrawlResultSummary>,
}

/// Summary of a single crawl result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CrawlResultSummary {
    pub url: String,
    pub title: Option<String>,
    pub word_count: usize,
    pub target_words_found: Vec<String>,
    pub language: Option<String>,
    pub status_code: Option<u16>,
}

impl CrawlStatus {
    /// Create initial status for a new session
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            status: "initialized".to_string(),
            total_urls_processed: 0,
            successful_crawls: 0,
            failed_crawls: 0,
            current_url: None,
            errors: vec![],
            results: vec![],
        }
    }
}
