#![allow(async_fn_in_trait)]

use crate::core::types::RetryConfig;
use anyhow::Error;
use std::collections::HashMap;
use std::net::IpAddr;
use url::Url;

/// Trait for DNS resolution with caching capabilities
pub trait DnsResolver {
    async fn resolve_domain(&self, domain: &str) -> Result<String, Error>;
    async fn resolve_hostname(&self, hostname: &str) -> Result<IpAddr, Error>;
    async fn cleanup_dns_cache(&self);
    async fn get_dns_cache_stats(&self) -> HashMap<String, String>;
}

/// Trait for robots.txt handling
pub trait RobotsChecker {
    async fn is_allowed_by_robots(&self, url: &Url) -> Result<bool, Error>;
    fn parse_robots_txt(&self, robots_content: &str, path: &str) -> (bool, Option<u64>);
}

/// Trait for content processing and validation
pub trait ContentProcessor {
    async fn extract_and_validate(&self, content: &str) -> Result<(String, usize), Error>;
    fn extract_text_from_cleaned_html(&self, html: &str) -> String;
    async fn extract_text_fallback(&self, content: &str) -> Result<(String, usize), Error>;
}

/// Trait for rate limiting behavior
pub trait RateLimiter {
    async fn check_and_wait(&self, domain: &str) -> Result<(), Error>;
    async fn get_current_request_count(&self, domain: &str) -> usize;
}

/// Trait for HTTP client management
pub trait HttpClientManager {
    async fn create_client_with_proxy(&self) -> Result<reqwest::Client, Error>;
    fn get_random_user_agent(&self) -> &'static str;
    fn get_accept_language_header(&self) -> String;
}

/// Trait for error categorization and retry logic
pub trait ErrorHandler {
    fn categorize_error(error: &Error) -> crate::core::error::CrawlError;
    fn should_retry(&self, error: &crate::core::error::CrawlError, config: &RetryConfig) -> bool;
    fn calculate_retry_delay(&self, config: &RetryConfig) -> std::time::Duration;
}
