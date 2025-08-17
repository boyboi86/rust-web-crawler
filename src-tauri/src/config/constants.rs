// =============================================================================
// CONFIGURATION CONSTANTS - Single Source of Truth
// =============================================================================
// All hardcoded values that appear throughout the application should be
// defined here as the single source of truth.

use std::time::Duration;

// =============================================================================
// NETWORK CONFIGURATION
// =============================================================================

/// Default user agents for web crawling
pub const DEFAULT_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36",
];

/// Default user agent (primary choice)
pub const DEFAULT_USER_AGENT: &str = DEFAULT_USER_AGENTS[0];

/// Tauri-specific user agent (for debugging/identification)
pub const TAURI_USER_AGENT: &str = "Tauri WebCrawler/1.0";

/// Default request timeout in seconds
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Minimum allowed request timeout in seconds (validation limit)
pub const MIN_REQUEST_TIMEOUT_SECS: u64 = 1;

/// Maximum allowed request timeout in seconds (validation limit)
pub const MAX_REQUEST_TIMEOUT_SECS: u64 = 300;

/// Default max redirects to follow
pub const DEFAULT_MAX_REDIRECTS: usize = 10;

/// HTTP success status code
pub const HTTP_SUCCESS_STATUS: u16 = 200;

/// Default politeness delay between requests (milliseconds)
pub const DEFAULT_POLITENESS_DELAY_MS: u64 = 1000;

// =============================================================================
// CRAWLING CONFIGURATION
// =============================================================================

/// Default maximum URLs to crawl
pub const DEFAULT_MAX_TOTAL_URLS: u32 = 100;

/// Default maximum crawl depth
pub const DEFAULT_MAX_CRAWL_DEPTH: u32 = 3;

/// Default minimum word length for content filtering
pub const DEFAULT_MIN_WORD_LENGTH: u32 = 3;

/// Default maximum concurrent requests
pub const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 5;

/// Maximum cap for concurrent requests (safety limit)
pub const MAX_CONCURRENT_REQUESTS_CAP: usize = 10;

/// Default file extensions to avoid during crawling
pub const DEFAULT_AVOID_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "zip", "rar", "7z", "jpg", "jpeg", "png", "gif", "bmp", "svg", "mp3",
    "mp4", "avi", "mov", "wmv", "flv", "exe", "msi", "dmg", "pkg", "deb", "rpm",
];

/// Default target words for content filtering
pub const DEFAULT_TARGET_WORDS: &[&str] = &["news", "article", "content", "information"];

/// Default excluded words for Latin content filtering
pub const DEFAULT_EXCLUDED_WORDS: &[&str] = &[
    "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
];

// =============================================================================
// RATE LIMITING CONFIGURATION
// =============================================================================

/// Default rate limit: requests per second
pub const DEFAULT_RATE_LIMIT_RPS: u32 = 2;

/// Default rate limit window size in milliseconds
pub const DEFAULT_RATE_LIMIT_WINDOW_MS: u64 = 1000;

// =============================================================================
// RETRY CONFIGURATION
// =============================================================================

/// Default maximum number of retries
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default base delay for exponential backoff (milliseconds)
pub const DEFAULT_BASE_DELAY_MS: u64 = 1000;

/// Default maximum delay for exponential backoff (milliseconds)
pub const DEFAULT_MAX_DELAY_MS: u64 = 30000;

/// Default backoff multiplier
pub const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;

/// Default jitter factor (0.0 to 1.0)
pub const DEFAULT_JITTER_FACTOR: f64 = 0.1;

// =============================================================================
// LOGGING CONFIGURATION
// =============================================================================

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Default log file path
pub const DEFAULT_LOG_FILE_PATH: &str = "logs/crawler.log";

/// Default log format (JSON vs plain text)
pub const DEFAULT_LOG_JSON_FORMAT: bool = false;

// =============================================================================
// PROXY CONFIGURATION
// =============================================================================

/// Default proxy pool (empty by default)
pub const DEFAULT_PROXY_POOL: &[&str] = &[];

/// Default proxy connection timeout (seconds)
pub const DEFAULT_PROXY_TIMEOUT_SECS: u64 = 10;

// =============================================================================
// CONTENT PROCESSING
// =============================================================================

/// Default minimum content length to process
pub const DEFAULT_MIN_CONTENT_LENGTH: usize = 100;

/// Default language content percentage threshold
pub const DEFAULT_LANGUAGE_CONTENT_PERCENTAGE: f64 = 70.0;

/// Default accepted languages
pub const DEFAULT_ACCEPTED_LANGUAGES: &[&str] = &["eng"];

// =============================================================================
// FRONTEND DEFAULTS
// =============================================================================

/// Default poll interval for status updates (milliseconds)
pub const DEFAULT_STATUS_POLL_INTERVAL_MS: u64 = 2000;

/// Default form validation timeout (milliseconds)
pub const DEFAULT_FORM_VALIDATION_TIMEOUT_MS: u64 = 5000;

// =============================================================================
// DEVELOPMENT/DEBUG CONFIGURATION
// =============================================================================

/// Whether to enable debug mode by default
pub const DEFAULT_DEBUG_MODE: bool = false;

/// Whether to enable metrics collection by default
pub const DEFAULT_ENABLE_METRICS: bool = false;

/// Default metrics port
pub const DEFAULT_METRICS_PORT: u16 = 9090;

/// Default health check interval (seconds)
pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Get default user agent for the current platform
pub fn get_default_user_agent() -> &'static str {
    #[cfg(target_os = "windows")]
    return DEFAULT_USER_AGENTS[0]; // Windows Chrome

    #[cfg(target_os = "macos")]
    return DEFAULT_USER_AGENTS[1]; // macOS Chrome

    #[cfg(target_os = "linux")]
    return DEFAULT_USER_AGENTS[2]; // Linux Chrome

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return DEFAULT_USER_AGENTS[0]; // Fallback to Windows Chrome
}

/// Get default request timeout as Duration
pub fn get_default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS)
}

/// Get default politeness delay as Duration
pub fn get_default_delay() -> Duration {
    Duration::from_millis(DEFAULT_POLITENESS_DELAY_MS)
}

// =============================================================================
// VALIDATION HELPERS
// =============================================================================

/// Validate that user agent is not a bot identifier
pub fn is_valid_user_agent(user_agent: &str) -> bool {
    let bot_indicators = ["bot", "crawler", "spider", "scraper"];
    let ua_lower = user_agent.to_lowercase();

    !bot_indicators
        .iter()
        .any(|indicator| ua_lower.contains(indicator))
}

/// Validate rate limit values
pub fn is_valid_rate_limit(rps: u32, window_ms: u64) -> bool {
    rps > 0 && rps <= 100 && window_ms >= 100 && window_ms <= 60000
}

/// Validate retry configuration
pub fn is_valid_retry_config(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> bool {
    max_retries <= 10
        && base_delay_ms >= 100
        && max_delay_ms >= base_delay_ms
        && max_delay_ms <= 300000 // 5 minutes max
}
