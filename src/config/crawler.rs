use crate::core::{DomainRateLimit, LangType, RetryConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Latin word filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatinWordFilter {
    pub exclude_numeric: bool,
    pub excluded_words: Vec<String>,
    pub min_word_length: usize,
}

impl Default for LatinWordFilter {
    fn default() -> Self {
        Self {
            exclude_numeric: true,
            excluded_words: vec![
                "the".to_string(),
                "and".to_string(),
                "or".to_string(),
                "but".to_string(),
                "in".to_string(),
                "on".to_string(),
                "at".to_string(),
                "to".to_string(),
                "for".to_string(),
                "of".to_string(),
                "with".to_string(),
                "by".to_string(),
            ],
            min_word_length: 3,
        }
    }
}

/// Simple logging configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String, // "trace", "debug", "info", "warn", "error"
    pub file_path: Option<PathBuf>,
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: Some(PathBuf::from("logs/crawler.log")),
            json_format: false,
        }
    }
}

/// Enhanced crawler configuration with better type safety
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebCrawlerConfig {
    pub base_url: Vec<String>,
    pub avoid_url_extensions: Vec<String>,
    pub target_words: Vec<String>,
    pub min_word_length: usize,
    pub proxy_pool: Vec<String>,
    pub user_agent: String,
    pub accepted_languages: Vec<LangType>,
    pub default_rate_limit: Option<DomainRateLimit>,
    pub domain_rate_limits: Option<HashMap<String, DomainRateLimit>>,
    pub retry_config: Option<RetryConfig>,
    pub logging_config: Option<LoggingConfig>,

    // Feature 1: Extension crawling option (follow links)
    pub enable_extension_crawling: bool,
    pub max_crawl_depth: usize,
    pub max_total_urls: usize,

    // Feature 2: Keyword filtering option
    pub enable_keyword_filtering: bool,

    // Feature 3: Latin word filtering (enhanced)
    pub latin_word_filter: LatinWordFilter,
}

impl Default for WebCrawlerConfig {
    fn default() -> Self {
        Self {
            base_url: vec!["https://example.com".to_string()],
            avoid_url_extensions: vec![".pdf".to_string(), ".jpg".to_string(), ".png".to_string()],
            target_words: vec![
                "news".to_string(),
                "article".to_string(),
                "content".to_string(),
                "information".to_string(),
            ],
            min_word_length: 10,  // Lowered from 50 to 10 for better success rate
            proxy_pool: vec![],
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".to_string(),
            accepted_languages: vec![LangType::Eng],
            default_rate_limit: Some(DomainRateLimit::default()),
            domain_rate_limits: None,
            retry_config: Some(RetryConfig::default()),
            logging_config: Some(LoggingConfig::default()),

            // Feature 1: Extension crawling - DEFAULT OFF
            enable_extension_crawling: false,
            max_crawl_depth: 3,
            max_total_urls: 1000,

            // Feature 2: Keyword filtering - DEFAULT OFF
            enable_keyword_filtering: false,

            // Feature 3: Latin word filtering
            latin_word_filter: LatinWordFilter::default(),
        }
    }
}

/// HTTP client factory with common configuration
pub struct HttpClientFactory;

impl HttpClientFactory {
    /// Create a standard HTTP client with default settings
    pub fn create_default_client(user_agent: &str) -> Result<reqwest::Client, reqwest::Error> {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(defaults::MAX_REDIRECTS))
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(
                defaults::REQUEST_TIMEOUT_SECS,
            ))
            .build()
    }

    /// Create an HTTP client with proxy support
    pub fn create_proxy_client(
        proxy_url: &str,
        user_agent: &str,
    ) -> Result<reqwest::Client, reqwest::Error> {
        let proxy = if proxy_url.starts_with("socks5://") {
            reqwest::Proxy::all(proxy_url)?
        } else {
            reqwest::Proxy::http(proxy_url)?
        };

        reqwest::Client::builder()
            .proxy(proxy)
            .redirect(reqwest::redirect::Policy::limited(defaults::MAX_REDIRECTS))
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(
                defaults::REQUEST_TIMEOUT_SECS,
            ))
            .pool_max_idle_per_host(defaults::CONNECTION_POOL_SIZE)
            .pool_idle_timeout(std::time::Duration::from_secs(
                defaults::CONNECTION_IDLE_TIMEOUT_SECS,
            ))
            .build()
    }
}

/// Crawler default constants
pub mod defaults {
    // Network settings
    pub const MAX_REDIRECTS: usize = 10;
    pub const REQUEST_TIMEOUT_SECS: u64 = 30;
    pub const CONNECTION_POOL_SIZE: usize = 10;
    pub const CONNECTION_IDLE_TIMEOUT_SECS: u64 = 60;

    // Rate limiting and politeness
    pub const DEFAULT_POLITENESS_DELAY_MS: u64 = 1000;
    pub const POLITENESS_DELAY_DIVISOR: u64 = 2;

    // DNS caching
    pub const DNS_CACHE_TTL_SECS: u64 = 300; // 5 minutes

    // Robots.txt caching
    pub const ROBOTS_CACHE_TTL_HOURS: u64 = 24; // 24 hours

    // Content processing
    pub const MIN_CONTENT_LENGTH_BYTES: usize = 100;
    pub const MIN_EXTRACTED_TEXT_LENGTH: usize = 50;
    pub const MIN_WORD_COUNT_THRESHOLD: usize = 10;
    pub const MIN_WORD_LENGTH_LATIN: usize = 3;

    // Language detection
    pub const LANG_DETECTION_SAMPLE_SIZE: usize = 1000;
    pub const CJK_WORD_COUNT_SAMPLE_SIZE: usize = 500;

    // Bloom filter settings
    pub const BLOOM_FALSE_POSITIVE_RATE: f32 = 0.01; // 1% false positive rate
    pub const BLOOM_CAPACITY: u32 = 1_000_000; // 1M URLs

    // HTTP headers
    pub const ACCEPT_HEADER: &str =
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8";
    pub const ACCEPT_ENCODING_HEADER: &str = "gzip, deflate";
    pub const CONNECTION_HEADER: &str = "keep-alive";
    pub const UPGRADE_INSECURE_REQUESTS: &str = "1";

    // User agents for rotation - Updated August 2025
    // memory efficient, stored in binary
    // no runtime allocation needed
    pub const USER_AGENTS: &[&str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36 Edg/127.0.0.0",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:129.0) Gecko/20100101 Firefox/129.0",
    ];

    // Language negotiation
    pub const FALLBACK_ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";
    pub const MAX_QUALITY: f64 = 1.0;
    pub const QUALITY_DECREMENT: f64 = 0.1;
    pub const QUALITY_STEP_DIVISOR: f64 = 1.0;
    pub const WILDCARD_QUALITY: &str = "*;q=0.1";

    // Time conversion
    pub const SECONDS_TO_MS_MULTIPLIER: f64 = 1000.0;

    // Cache and timeout durations
    pub const ZOMBIE_TASK_TIMEOUT_SECS: u64 = 300; // 5 minutes
    pub const CLEANUP_INTERVAL_SECS: u64 = 30;
    pub const MONITORING_UPDATE_INTERVAL_MS: u64 = 500;
    pub const QUEUE_POLLING_INTERVAL_MS: u64 = 100;

    // Retry and backoff
    pub const DEFAULT_BASE_RETRY_DELAY_MS: u64 = 1000;
    pub const DEFAULT_MAX_RETRY_DELAY_MS: u64 = 30000;
    pub const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;

    // Crawling defaults
    pub const DEFAULT_MAX_DEPTH: usize = 3;

    // Application defaults
    pub const DEFAULT_APP_USER_AGENT: &str = "RustCrawler/1.0";
    pub const DEFAULT_WEBCRAWLER_USER_AGENT: &str = "WebCrawler/1.0";

    // HTTP client defaults
    pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;
    pub const DEFAULT_TCP_KEEPALIVE_SECS: u64 = 30;

    // Rate limiting thresholds
    pub const RATE_LIMIT_LOG_THRESHOLD_MS: u64 = 100;

    // Timeout defaults
    pub const DEFAULT_QUEUE_PROCESSING_TIMEOUT_SECS: u64 = 300; // 5 minutes

    // Task queue defaults
    pub const DEFAULT_TASK_QUEUE_RETRIES: usize = 3;
}
