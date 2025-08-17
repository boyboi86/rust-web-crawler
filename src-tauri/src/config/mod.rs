// =============================================================================
// CONFIGURATION MODULE - Single Source of Truth
// =============================================================================
// This module provides unified configuration management for the entire
// application, ensuring all defaults and environment variables are loaded
// from a single location.

pub mod constants;
pub mod defaults;
pub mod environment;

use crate::config::constants::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// UNIFIED APPLICATION CONFIGURATION
// =============================================================================

/// Complete application configuration structure
/// This represents the single source of truth for all application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub crawling: CrawlingConfig,
    pub rate_limiting: RateLimitingConfig,
    pub retry: RetryConfig,
    pub logging: LoggingConfig,
    pub proxy: ProxyConfig,
    pub content: ContentConfig,
    pub frontend: FrontendConfig,
    pub development: DevelopmentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub user_agent: String,
    pub request_timeout_secs: u64,
    pub max_redirects: usize,
    pub politeness_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlingConfig {
    pub max_total_urls: u32,
    pub max_crawl_depth: u32,
    pub min_word_length: u32,
    pub max_concurrent_requests: usize,
    pub avoid_extensions: Vec<String>,
    pub target_words: Vec<String>,
    pub excluded_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_second: u32,
    pub window_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub json_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_pool: Vec<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    pub min_content_length: usize,
    pub language_content_percentage: f64,
    pub accepted_languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub status_poll_interval_ms: u64,
    pub form_validation_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    pub debug_mode: bool,
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval_secs: u64,
}

// =============================================================================
// CONFIGURATION BUILDER
// =============================================================================

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            crawling: CrawlingConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            retry: RetryConfig::default(),
            logging: LoggingConfig::default(),
            proxy: ProxyConfig::default(),
            content: ContentConfig::default(),
            frontend: FrontendConfig::default(),
            development: DevelopmentConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            user_agent: get_default_user_agent().to_string(),
            request_timeout_secs: DEFAULT_REQUEST_TIMEOUT_SECS,
            max_redirects: DEFAULT_MAX_REDIRECTS,
            politeness_delay_ms: DEFAULT_POLITENESS_DELAY_MS,
        }
    }
}

impl Default for CrawlingConfig {
    fn default() -> Self {
        Self {
            max_total_urls: DEFAULT_MAX_TOTAL_URLS,
            max_crawl_depth: DEFAULT_MAX_CRAWL_DEPTH,
            min_word_length: DEFAULT_MIN_WORD_LENGTH,
            max_concurrent_requests: DEFAULT_MAX_CONCURRENT_REQUESTS,
            avoid_extensions: DEFAULT_AVOID_EXTENSIONS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            target_words: DEFAULT_TARGET_WORDS.iter().map(|s| s.to_string()).collect(),
            excluded_words: DEFAULT_EXCLUDED_WORDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            requests_per_second: DEFAULT_RATE_LIMIT_RPS,
            window_ms: DEFAULT_RATE_LIMIT_WINDOW_MS,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
            max_delay_ms: DEFAULT_MAX_DELAY_MS,
            backoff_multiplier: DEFAULT_BACKOFF_MULTIPLIER,
            jitter_factor: DEFAULT_JITTER_FACTOR,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: DEFAULT_LOG_LEVEL.to_string(),
            file_path: DEFAULT_LOG_FILE_PATH.to_string(),
            json_format: DEFAULT_LOG_JSON_FORMAT,
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            proxy_pool: DEFAULT_PROXY_POOL.iter().map(|s| s.to_string()).collect(),
            timeout_secs: DEFAULT_PROXY_TIMEOUT_SECS,
        }
    }
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            min_content_length: DEFAULT_MIN_CONTENT_LENGTH,
            language_content_percentage: DEFAULT_LANGUAGE_CONTENT_PERCENTAGE,
            accepted_languages: DEFAULT_ACCEPTED_LANGUAGES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            status_poll_interval_ms: DEFAULT_STATUS_POLL_INTERVAL_MS,
            form_validation_timeout_ms: DEFAULT_FORM_VALIDATION_TIMEOUT_MS,
        }
    }
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            debug_mode: DEFAULT_DEBUG_MODE,
            enable_metrics: DEFAULT_ENABLE_METRICS,
            metrics_port: DEFAULT_METRICS_PORT,
            health_check_interval_secs: DEFAULT_HEALTH_CHECK_INTERVAL_SECS,
        }
    }
}

// =============================================================================
// CONFIGURATION LOADING & VALIDATION
// =============================================================================

impl AppConfig {
    /// Load configuration from environment variables and defaults
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // Load from environment variables if available
        if let Ok(env_config) = environment::load_from_environment() {
            config = config.merge_with_environment(env_config);
        }

        // Validate the final configuration
        config.validate()?;

        Ok(config)
    }

    /// Merge current config with environment overrides
    fn merge_with_environment(mut self, env: environment::EnvironmentConfig) -> Self {
        // Network overrides
        if let Some(ua) = env.user_agent {
            self.network.user_agent = ua;
        }
        if let Some(timeout) = env.request_timeout_secs {
            self.network.request_timeout_secs = timeout;
        }

        // Crawling overrides
        if let Some(max_urls) = env.max_total_urls {
            self.crawling.max_total_urls = max_urls;
        }
        if let Some(max_depth) = env.max_crawl_depth {
            self.crawling.max_crawl_depth = max_depth;
        }

        // Rate limiting overrides
        if let Some(rps) = env.rate_limit_rps {
            self.rate_limiting.requests_per_second = rps;
        }

        // Proxy overrides
        if let Some(proxies) = env.proxy_pool {
            self.proxy.proxy_pool = proxies;
        }

        // Development overrides
        if let Some(debug) = env.debug_mode {
            self.development.debug_mode = debug;
        }

        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user agent
        if !is_valid_user_agent(&self.network.user_agent) {
            return Err("Invalid user agent: contains bot indicators".into());
        }

        // Validate rate limiting
        if !is_valid_rate_limit(
            self.rate_limiting.requests_per_second,
            self.rate_limiting.window_ms,
        ) {
            return Err("Invalid rate limiting configuration".into());
        }

        // Validate retry configuration
        if !is_valid_retry_config(
            self.retry.max_retries,
            self.retry.base_delay_ms,
            self.retry.max_delay_ms,
        ) {
            return Err("Invalid retry configuration".into());
        }

        // Validate timeouts are reasonable
        if self.network.request_timeout_secs < crate::config::constants::MIN_REQUEST_TIMEOUT_SECS
            || self.network.request_timeout_secs
                > crate::config::constants::MAX_REQUEST_TIMEOUT_SECS
        {
            return Err(format!(
                "Request timeout must be between {} and {} seconds",
                crate::config::constants::MIN_REQUEST_TIMEOUT_SECS,
                crate::config::constants::MAX_REQUEST_TIMEOUT_SECS
            )
            .into());
        }

        Ok(())
    }

    /// Get a user-friendly summary of the current configuration
    pub fn summary(&self) -> String {
        format!(
            "App Configuration:\n\
             - User Agent: {}\n\
             - Max URLs: {}\n\
             - Max Depth: {}\n\
             - Request Timeout: {}s\n\
             - Rate Limit: {} req/s\n\
             - Proxy Pool: {} proxies\n\
             - Debug Mode: {}",
            self.network.user_agent,
            self.crawling.max_total_urls,
            self.crawling.max_crawl_depth,
            self.network.request_timeout_secs,
            self.rate_limiting.requests_per_second,
            self.proxy.proxy_pool.len(),
            self.development.debug_mode
        )
    }
}
