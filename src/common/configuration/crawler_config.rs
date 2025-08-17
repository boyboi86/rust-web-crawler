use crate::common::configuration::{
    ConfigBuilder, ContentFilterConfig, HttpClientConfig, ProxyConfig, RateLimitConfig, RetryConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    retry_config: RetryConfig,
    rate_limit_config: RateLimitConfig,
    http_client_config: HttpClientConfig,
    proxy_config: ProxyConfig,
    content_filter_config: ContentFilterConfig,
}

impl CrawlerConfig {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }
    pub fn rate_limit_config(&self) -> &RateLimitConfig {
        &self.rate_limit_config
    }
    pub fn http_client_config(&self) -> &HttpClientConfig {
        &self.http_client_config
    }
    pub fn proxy_config(&self) -> &ProxyConfig {
        &self.proxy_config
    }
    pub fn content_filter_config(&self) -> &ContentFilterConfig {
        &self.content_filter_config
    }
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        ConfigBuilder::default().build()
    }
}
