// =============================================================================
// DEFAULT CONFIGURATION BUILDERS
// =============================================================================
// This module provides builders and helpers for creating default configurations
// for different use cases (development, production, testing).

use crate::config::AppConfig;
use crate::config::constants::*;

/// Configuration preset for different environments
#[derive(Debug, Clone)]
pub enum ConfigPreset {
    Development,
    Production,
    Testing,
    Debug,
}

impl AppConfig {
    /// Create configuration for a specific preset
    pub fn for_preset(preset: ConfigPreset) -> Self {
        match preset {
            ConfigPreset::Development => Self::development_config(),
            ConfigPreset::Production => Self::production_config(),
            ConfigPreset::Testing => Self::testing_config(),
            ConfigPreset::Debug => Self::debug_config(),
        }
    }

    /// Development configuration with relaxed settings
    pub fn development_config() -> Self {
        let mut config = Self::default();

        // Relaxed network settings for development
        config.network.request_timeout_secs = 10;
        config.network.politeness_delay_ms = 500; // Faster crawling

        // Smaller crawl limits for development
        config.crawling.max_total_urls = 20;
        config.crawling.max_crawl_depth = 2;
        config.crawling.max_concurrent_requests = 2;

        // More aggressive rate limiting for development
        config.rate_limiting.requests_per_second = 1;

        // Development logging
        config.logging.level = "debug".to_string();
        config.logging.json_format = false; // Human-readable logs

        // Development flags
        config.development.debug_mode = true;
        config.development.enable_metrics = true;

        config
    }

    /// Production configuration with conservative settings
    pub fn production_config() -> Self {
        let mut config = Self::default();

        // Conservative network settings for production
        config.network.request_timeout_secs = 30;
        config.network.politeness_delay_ms = 2000; // Respectful crawling

        // Production crawl limits
        config.crawling.max_total_urls = 1000;
        config.crawling.max_crawl_depth = 5;
        config.crawling.max_concurrent_requests = 3;

        // Conservative rate limiting
        config.rate_limiting.requests_per_second = 1;

        // Production logging
        config.logging.level = "info".to_string();
        config.logging.json_format = true; // Structured logs

        // Production flags
        config.development.debug_mode = false;
        config.development.enable_metrics = true;

        config
    }

    /// Testing configuration with minimal settings
    pub fn testing_config() -> Self {
        let mut config = Self::default();

        // Fast network settings for testing
        config.network.request_timeout_secs = 5;
        config.network.politeness_delay_ms = 100; // Very fast for tests

        // Minimal crawl limits for testing
        config.crawling.max_total_urls = 5;
        config.crawling.max_crawl_depth = 1;
        config.crawling.max_concurrent_requests = 1;

        // Fast rate limiting for testing
        config.rate_limiting.requests_per_second = 10;

        // Testing logging
        config.logging.level = "warn".to_string(); // Minimal logs
        config.logging.json_format = false;

        // Testing flags
        config.development.debug_mode = false;
        config.development.enable_metrics = false;

        config
    }

    /// Debug configuration with verbose settings
    pub fn debug_config() -> Self {
        let mut config = Self::default();

        // Debug network settings
        config.network.request_timeout_secs = 60; // Longer for debugging
        config.network.politeness_delay_ms = 3000; // Very slow for analysis

        // Debug crawl limits
        config.crawling.max_total_urls = 10;
        config.crawling.max_crawl_depth = 2;
        config.crawling.max_concurrent_requests = 1; // Sequential for debugging

        // Slow rate limiting for debugging
        config.rate_limiting.requests_per_second = 1;

        // Verbose logging
        config.logging.level = "trace".to_string();
        config.logging.json_format = false; // Human-readable for debugging

        // All debug flags enabled
        config.development.debug_mode = true;
        config.development.enable_metrics = true;

        config
    }
}

/// Builder pattern for creating custom configurations
pub struct ConfigBuilder {
    config: AppConfig,
}

impl ConfigBuilder {
    /// Start with default configuration
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    /// Start with a preset configuration
    pub fn from_preset(preset: ConfigPreset) -> Self {
        Self {
            config: AppConfig::for_preset(preset),
        }
    }

    /// Set user agent
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.config.network.user_agent = user_agent.into();
        self
    }

    /// Set request timeout
    pub fn request_timeout_secs(mut self, timeout: u64) -> Self {
        self.config.network.request_timeout_secs = timeout;
        self
    }

    /// Set maximum URLs to crawl
    pub fn max_total_urls(mut self, max_urls: u32) -> Self {
        self.config.crawling.max_total_urls = max_urls;
        self
    }

    /// Set maximum crawl depth
    pub fn max_crawl_depth(mut self, max_depth: u32) -> Self {
        self.config.crawling.max_crawl_depth = max_depth;
        self
    }

    /// Set rate limiting
    pub fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.config.rate_limiting.requests_per_second = requests_per_second;
        self
    }

    /// Set proxy pool
    pub fn proxy_pool(mut self, proxies: Vec<String>) -> Self {
        self.config.proxy.proxy_pool = proxies;
        self
    }

    /// Set log level
    pub fn log_level<S: Into<String>>(mut self, level: S) -> Self {
        self.config.logging.level = level.into();
        self
    }

    /// Enable or disable debug mode
    pub fn debug_mode(mut self, enabled: bool) -> Self {
        self.config.development.debug_mode = enabled;
        self
    }

    /// Set minimum word length for content filtering
    pub fn min_word_length(mut self, min_length: u32) -> Self {
        self.config.crawling.min_word_length = min_length;
        self
    }

    /// Build the final configuration
    pub fn build(self) -> AppConfig {
        self.config
    }

    /// Build and validate the configuration
    pub fn build_and_validate(self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        let config = self.config;
        config.validate()?;
        Ok(config)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// CONFIGURATION TEMPLATES
// =============================================================================

/// Get configuration for Yahoo.com crawling (production-ready)
pub fn yahoo_config() -> AppConfig {
    ConfigBuilder::from_preset(ConfigPreset::Production)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
        .max_total_urls(50)
        .max_crawl_depth(2)
        .rate_limit(1) // Very respectful for Yahoo
        .request_timeout_secs(30)
        .build()
}

/// Get configuration for testing with any URL
pub fn test_config() -> AppConfig {
    ConfigBuilder::from_preset(ConfigPreset::Testing)
        .user_agent(get_default_user_agent())
        .max_total_urls(5)
        .max_crawl_depth(1)
        .rate_limit(5) // Faster for testing
        .request_timeout_secs(10)
        .debug_mode(true)
        .build()
}

/// Get configuration for development work
pub fn dev_config() -> AppConfig {
    ConfigBuilder::from_preset(ConfigPreset::Development)
        .user_agent(get_default_user_agent())
        .max_total_urls(20)
        .max_crawl_depth(2)
        .rate_limit(2)
        .request_timeout_secs(15)
        .debug_mode(true)
        .log_level("debug")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .user_agent("Test Bot")
            .max_total_urls(100)
            .debug_mode(true)
            .build();

        assert_eq!(config.network.user_agent, "Test Bot");
        assert_eq!(config.crawling.max_total_urls, 100);
        assert_eq!(config.development.debug_mode, true);
    }

    #[test]
    fn test_presets() {
        let dev_config = AppConfig::for_preset(ConfigPreset::Development);
        let prod_config = AppConfig::for_preset(ConfigPreset::Production);

        // Development should be more permissive
        assert!(dev_config.crawling.max_total_urls < prod_config.crawling.max_total_urls);
        assert!(dev_config.development.debug_mode);
        assert!(!prod_config.development.debug_mode);
    }

    #[test]
    fn test_yahoo_config() {
        let config = yahoo_config();
        assert_eq!(config.rate_limiting.requests_per_second, 1);
        assert_eq!(config.crawling.max_total_urls, 50);
        assert!(config.network.user_agent.contains("Chrome"));
    }
}
