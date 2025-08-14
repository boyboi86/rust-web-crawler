#![allow(async_fn_in_trait)]

use crate::core::types::RetryConfig;
use anyhow::Error;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;
use url::Url;

/// Building block trait for categorizable types - promotes composition
pub trait Categorizable<T> {
    fn categorize(&self) -> T;
}

/// Building block trait for retryable operations - promotes reusability
pub trait Retryable {
    fn is_retryable(&self) -> bool;
    fn calculate_delay(&self, attempt: u32, config: &RetryConfig) -> Duration;
}

/// Building block trait for timestamped entities - promotes timing composition
pub trait TimestampedTask {
    type Timing;
    fn timing(&self) -> &Self::Timing;
    fn timing_mut(&mut self) -> &mut Self::Timing;

    /// Default implementation using building blocks
    fn mark_started(&mut self) {
        // Implementation will depend on the concrete Timing type
    }
}

/// Building block trait for validatable types - promotes validation composition
pub trait Validatable {
    type ValidationError;
    fn validate(&self) -> Result<(), Self::ValidationError>;
}

/// Building block trait for normalizable data - promotes consistency
pub trait Normalizable {
    fn normalize(&mut self) -> &mut Self;
    fn normalized(mut self) -> Self
    where
        Self: Sized,
    {
        self.normalize();
        self
    }
}

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
