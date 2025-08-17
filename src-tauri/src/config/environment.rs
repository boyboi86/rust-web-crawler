// =============================================================================
// ENVIRONMENT VARIABLE LOADING
// =============================================================================
// This module handles loading configuration from environment variables,
// providing a clean way to override defaults without hardcoding values.

use std::env;

/// Configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    // Network settings
    pub user_agent: Option<String>,
    pub request_timeout_secs: Option<u64>,
    pub max_redirects: Option<usize>,
    pub politeness_delay_ms: Option<u64>,

    // Crawling settings
    pub max_total_urls: Option<u32>,
    pub max_crawl_depth: Option<u32>,
    pub min_word_length: Option<u32>,
    pub max_concurrent_requests: Option<usize>,

    // Rate limiting
    pub rate_limit_rps: Option<u32>,
    pub rate_limit_window_ms: Option<u64>,

    // Retry configuration
    pub max_retries: Option<u32>,
    pub base_delay_ms: Option<u64>,
    pub max_delay_ms: Option<u64>,

    // Proxy settings
    pub proxy_pool: Option<Vec<String>>,
    pub proxy_timeout_secs: Option<u64>,

    // Logging
    pub log_level: Option<String>,
    pub log_file_path: Option<String>,
    pub log_json_format: Option<bool>,

    // Development
    pub debug_mode: Option<bool>,
    pub enable_metrics: Option<bool>,
    pub metrics_port: Option<u16>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            user_agent: None,
            request_timeout_secs: None,
            max_redirects: None,
            politeness_delay_ms: None,
            max_total_urls: None,
            max_crawl_depth: None,
            min_word_length: None,
            max_concurrent_requests: None,
            rate_limit_rps: None,
            rate_limit_window_ms: None,
            max_retries: None,
            base_delay_ms: None,
            max_delay_ms: None,
            proxy_pool: None,
            proxy_timeout_secs: None,
            log_level: None,
            log_file_path: None,
            log_json_format: None,
            debug_mode: None,
            enable_metrics: None,
            metrics_port: None,
        }
    }
}

/// Load configuration from environment variables
pub fn load_from_environment() -> Result<EnvironmentConfig, Box<dyn std::error::Error>> {
    let mut config = EnvironmentConfig::default();

    // Network settings
    config.user_agent = env::var("CRAWLER_USER_AGENT").ok();
    config.request_timeout_secs = parse_env_u64("CRAWLER_REQUEST_TIMEOUT_SECS");
    config.max_redirects = parse_env_usize("CRAWLER_MAX_REDIRECTS");
    config.politeness_delay_ms = parse_env_u64("CRAWLER_POLITENESS_DELAY_MS");

    // Crawling settings
    config.max_total_urls = parse_env_u32("CRAWLER_MAX_TOTAL_URLS");
    config.max_crawl_depth = parse_env_u32("CRAWLER_MAX_CRAWL_DEPTH");
    config.min_word_length = parse_env_u32("CRAWLER_MIN_WORD_LENGTH");
    config.max_concurrent_requests = parse_env_usize("CRAWLER_MAX_CONCURRENT_REQUESTS");

    // Rate limiting
    config.rate_limit_rps = parse_env_u32("CRAWLER_RATE_LIMIT_RPS");
    config.rate_limit_window_ms = parse_env_u64("CRAWLER_RATE_LIMIT_WINDOW_MS");

    // Retry configuration
    config.max_retries = parse_env_u32("CRAWLER_MAX_RETRIES");
    config.base_delay_ms = parse_env_u64("CRAWLER_BASE_DELAY_MS");
    config.max_delay_ms = parse_env_u64("CRAWLER_MAX_DELAY_MS");

    // Proxy settings
    config.proxy_pool = parse_env_string_list("CRAWLER_PROXY_POOL");
    config.proxy_timeout_secs = parse_env_u64("CRAWLER_PROXY_TIMEOUT_SECS");

    // Logging
    config.log_level = env::var("CRAWLER_LOG_LEVEL").ok();
    config.log_file_path = env::var("CRAWLER_LOG_FILE_PATH").ok();
    config.log_json_format = parse_env_bool("CRAWLER_LOG_JSON_FORMAT");

    // Development
    config.debug_mode = parse_env_bool("CRAWLER_DEBUG_MODE");
    config.enable_metrics = parse_env_bool("CRAWLER_ENABLE_METRICS");
    config.metrics_port = parse_env_u16("CRAWLER_METRICS_PORT");

    Ok(config)
}

// =============================================================================
// ENVIRONMENT PARSING HELPERS
// =============================================================================

fn parse_env_u32(key: &str) -> Option<u32> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_u64(key: &str) -> Option<u64> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_u16(key: &str) -> Option<u16> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_usize(key: &str) -> Option<usize> {
    env::var(key).ok()?.parse().ok()
}

fn parse_env_bool(key: &str) -> Option<bool> {
    match env::var(key).ok()?.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_env_string_list(key: &str) -> Option<Vec<String>> {
    let value = env::var(key).ok()?;
    if value.trim().is_empty() {
        return None;
    }

    Some(
        value
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    )
}

// =============================================================================
// ENVIRONMENT VARIABLE DOCUMENTATION
// =============================================================================

/// Print documentation for all supported environment variables
pub fn print_env_documentation() {
    println!(
        r#"
=============================================================================
CRAWLER ENVIRONMENT VARIABLES
=============================================================================

Network Configuration:
  CRAWLER_USER_AGENT                 - Custom user agent string
  CRAWLER_REQUEST_TIMEOUT_SECS       - Request timeout in seconds (1-300)
  CRAWLER_MAX_REDIRECTS              - Maximum redirects to follow (0-20)
  CRAWLER_POLITENESS_DELAY_MS        - Delay between requests in milliseconds

Crawling Configuration:
  CRAWLER_MAX_TOTAL_URLS             - Maximum URLs to crawl (1-10000)
  CRAWLER_MAX_CRAWL_DEPTH            - Maximum crawl depth (1-10)
  CRAWLER_MIN_WORD_LENGTH            - Minimum word length for filtering
  CRAWLER_MAX_CONCURRENT_REQUESTS    - Max concurrent requests (1-50)

Rate Limiting:
  CRAWLER_RATE_LIMIT_RPS             - Requests per second (1-100)
  CRAWLER_RATE_LIMIT_WINDOW_MS       - Rate limit window in milliseconds

Retry Configuration:
  CRAWLER_MAX_RETRIES                - Maximum retry attempts (0-10)
  CRAWLER_BASE_DELAY_MS              - Base delay for retries in milliseconds
  CRAWLER_MAX_DELAY_MS               - Maximum delay for retries in milliseconds

Proxy Configuration:
  CRAWLER_PROXY_POOL                 - Comma-separated list of proxy URLs
  CRAWLER_PROXY_TIMEOUT_SECS         - Proxy connection timeout in seconds

Logging Configuration:
  CRAWLER_LOG_LEVEL                  - Log level (trace, debug, info, warn, error)
  CRAWLER_LOG_FILE_PATH              - Path to log file
  CRAWLER_LOG_JSON_FORMAT            - Use JSON log format (true/false)

Development Configuration:
  CRAWLER_DEBUG_MODE                 - Enable debug mode (true/false)
  CRAWLER_ENABLE_METRICS             - Enable metrics collection (true/false)
  CRAWLER_METRICS_PORT               - Port for metrics server

Example Usage:
  export CRAWLER_USER_AGENT="Custom Bot 1.0"
  export CRAWLER_MAX_TOTAL_URLS=500
  export CRAWLER_DEBUG_MODE=true
  export CRAWLER_PROXY_POOL="http://proxy1:8080,http://proxy2:8080"

=============================================================================
"#
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_bool() {
        std::env::set_var("TEST_BOOL_TRUE", "true");
        std::env::set_var("TEST_BOOL_FALSE", "false");
        std::env::set_var("TEST_BOOL_1", "1");
        std::env::set_var("TEST_BOOL_0", "0");
        std::env::set_var("TEST_BOOL_INVALID", "maybe");

        assert_eq!(parse_env_bool("TEST_BOOL_TRUE"), Some(true));
        assert_eq!(parse_env_bool("TEST_BOOL_FALSE"), Some(false));
        assert_eq!(parse_env_bool("TEST_BOOL_1"), Some(true));
        assert_eq!(parse_env_bool("TEST_BOOL_0"), Some(false));
        assert_eq!(parse_env_bool("TEST_BOOL_INVALID"), None);
        assert_eq!(parse_env_bool("TEST_BOOL_NONEXISTENT"), None);
    }

    #[test]
    fn test_parse_env_string_list() {
        std::env::set_var("TEST_LIST", "a,b,c");
        std::env::set_var("TEST_LIST_SPACES", " a , b , c ");
        std::env::set_var("TEST_LIST_EMPTY", "");

        assert_eq!(
            parse_env_string_list("TEST_LIST"),
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        assert_eq!(
            parse_env_string_list("TEST_LIST_SPACES"),
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        assert_eq!(parse_env_string_list("TEST_LIST_EMPTY"), None);
        assert_eq!(parse_env_string_list("TEST_LIST_NONEXISTENT"), None);
    }
}
