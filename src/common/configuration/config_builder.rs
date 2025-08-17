use crate::common::CrawlerConfig;
use crate::common::configuration::{
    ContentFilterConfig, HttpClientConfig, ProxyConfig, RateLimitConfig, RetryConfig,
};

#[derive(Debug)]
pub struct ConfigBuilder {
    retry_config: RetryConfig,
    rate_limit_config: RateLimitConfig,
    http_client_config: HttpClientConfig,
    proxy_config: ProxyConfig,
    content_filter_config: ContentFilterConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            retry_config: RetryConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            http_client_config: HttpClientConfig::default(),
            proxy_config: ProxyConfig::default(),
            content_filter_config: ContentFilterConfig::default(),
        }
    }
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    pub fn with_rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = config;
        self
    }
    pub fn with_http_client_config(mut self, config: HttpClientConfig) -> Self {
        self.http_client_config = config;
        self
    }
    pub fn with_proxy_config(mut self, config: ProxyConfig) -> Self {
        self.proxy_config = config;
        self
    }
    pub fn with_content_filter_config(mut self, config: ContentFilterConfig) -> Self {
        self.content_filter_config = config;
        self
    }
    pub fn build(self) -> CrawlerConfig {
        CrawlerConfig {
            retry_config: self.retry_config,
            rate_limit_config: self.rate_limit_config,
            http_client_config: self.http_client_config,
            proxy_config: self.proxy_config,
            content_filter_config: self.content_filter_config,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
