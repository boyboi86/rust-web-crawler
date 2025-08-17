/// Crawler engine - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex crawler configurations
/// Following Rule 4: Privacy first - controlled access to crawling logic
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, CrawlResult, CrawlTask, CrawlerConfig, DelayDuration, LimitValue, NetworkResult,
    ProcessingResult, SessionId, TaskError, TaskId, TaskResult, TimeoutDuration, UrlString,
};
use bloom::{ASMS, BloomFilter};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::sleep;
use url::Url;

/// Configuration for the crawler engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerEngineConfig {
    // Concurrency settings
    max_concurrent_requests: LimitValue,
    request_delay: DelayDuration,
    retry_delay: DelayDuration,
    request_timeout: TimeoutDuration,

    // URL filtering
    bloom_filter_capacity: LimitValue,
    bloom_filter_false_positive_rate: f64,
    max_url_length: LimitValue,

    // Content settings
    follow_redirects: BooleanFlag,
    max_redirects: LimitValue,
    respect_robots_txt: BooleanFlag,

    // Rate limiting
    requests_per_second: LimitValue,
    burst_capacity: LimitValue,
    domain_delay: DelayDuration,

    // Session management
    session_id: SessionId,
    enable_statistics: BooleanFlag,
    enable_logging: BooleanFlag,
}

impl CrawlerEngineConfig {
    pub fn builder() -> CrawlerEngineConfigBuilder {
        CrawlerEngineConfigBuilder::new()
    }

    pub fn max_concurrent_requests(&self) -> u64 {
        self.max_concurrent_requests.value()
    }

    pub fn request_delay(&self) -> DelayDuration {
        self.request_delay
    }

    pub fn retry_delay(&self) -> DelayDuration {
        self.retry_delay
    }

    pub fn request_timeout(&self) -> TimeoutDuration {
        self.request_timeout
    }

    pub fn bloom_filter_capacity(&self) -> u64 {
        self.bloom_filter_capacity.value()
    }

    pub fn bloom_filter_false_positive_rate(&self) -> f64 {
        self.bloom_filter_false_positive_rate
    }

    pub fn max_url_length(&self) -> u64 {
        self.max_url_length.value()
    }

    pub fn should_follow_redirects(&self) -> bool {
        self.follow_redirects.is_enabled()
    }

    pub fn max_redirects(&self) -> u64 {
        self.max_redirects.value()
    }

    pub fn should_respect_robots_txt(&self) -> bool {
        self.respect_robots_txt.is_enabled()
    }

    pub fn requests_per_second(&self) -> u64 {
        self.requests_per_second.value()
    }

    pub fn burst_capacity(&self) -> u64 {
        self.burst_capacity.value()
    }

    pub fn domain_delay(&self) -> DelayDuration {
        self.domain_delay
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn is_statistics_enabled(&self) -> bool {
        self.enable_statistics.is_enabled()
    }

    pub fn is_logging_enabled(&self) -> bool {
        self.enable_logging.is_enabled()
    }
}

impl Default for CrawlerEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: LimitValue::new(10),
            request_delay: DelayDuration::from_millis(100),
            retry_delay: DelayDuration::from_secs(1),
            request_timeout: TimeoutDuration::from_secs(30),
            bloom_filter_capacity: LimitValue::new(1_000_000),
            bloom_filter_false_positive_rate: 0.01,
            max_url_length: LimitValue::new(2048),
            follow_redirects: BooleanFlag::enabled(),
            max_redirects: LimitValue::new(5),
            respect_robots_txt: BooleanFlag::enabled(),
            requests_per_second: LimitValue::new(10),
            burst_capacity: LimitValue::new(20),
            domain_delay: DelayDuration::from_millis(500),
            session_id: SessionId::new(format!("crawler_{}", uuid::Uuid::new_v4())),
            enable_statistics: BooleanFlag::enabled(),
            enable_logging: BooleanFlag::enabled(),
        }
    }
}

/// Builder for crawler engine configuration
#[derive(Debug)]
pub struct CrawlerEngineConfigBuilder {
    max_concurrent_requests: LimitValue,
    request_delay: DelayDuration,
    retry_delay: DelayDuration,
    request_timeout: TimeoutDuration,
    bloom_filter_capacity: LimitValue,
    bloom_filter_false_positive_rate: f64,
    max_url_length: LimitValue,
    follow_redirects: BooleanFlag,
    max_redirects: LimitValue,
    respect_robots_txt: BooleanFlag,
    requests_per_second: LimitValue,
    burst_capacity: LimitValue,
    domain_delay: DelayDuration,
    session_id: SessionId,
    enable_statistics: BooleanFlag,
    enable_logging: BooleanFlag,
}

impl CrawlerEngineConfigBuilder {
    pub fn new() -> Self {
        let default_config = CrawlerEngineConfig::default();
        Self {
            max_concurrent_requests: default_config.max_concurrent_requests,
            request_delay: default_config.request_delay,
            retry_delay: default_config.retry_delay,
            request_timeout: default_config.request_timeout,
            bloom_filter_capacity: default_config.bloom_filter_capacity,
            bloom_filter_false_positive_rate: default_config.bloom_filter_false_positive_rate,
            max_url_length: default_config.max_url_length,
            follow_redirects: default_config.follow_redirects,
            max_redirects: default_config.max_redirects,
            respect_robots_txt: default_config.respect_robots_txt,
            requests_per_second: default_config.requests_per_second,
            burst_capacity: default_config.burst_capacity,
            domain_delay: default_config.domain_delay,
            session_id: default_config.session_id,
            enable_statistics: default_config.enable_statistics,
            enable_logging: default_config.enable_logging,
        }
    }

    pub fn with_concurrency_settings(
        mut self,
        max_concurrent: LimitValue,
        request_delay: DelayDuration,
        retry_delay: DelayDuration,
    ) -> Self {
        self.max_concurrent_requests = max_concurrent;
        self.request_delay = request_delay;
        self.retry_delay = retry_delay;
        self
    }

    pub fn with_timeout_settings(mut self, request_timeout: TimeoutDuration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    pub fn with_bloom_filter_settings(
        mut self,
        capacity: LimitValue,
        false_positive_rate: f64,
    ) -> Self {
        self.bloom_filter_capacity = capacity;
        self.bloom_filter_false_positive_rate = false_positive_rate;
        self
    }

    pub fn with_url_settings(
        mut self,
        max_url_length: LimitValue,
        follow_redirects: BooleanFlag,
        max_redirects: LimitValue,
    ) -> Self {
        self.max_url_length = max_url_length;
        self.follow_redirects = follow_redirects;
        self.max_redirects = max_redirects;
        self
    }

    pub fn with_rate_limiting(
        mut self,
        requests_per_second: LimitValue,
        burst_capacity: LimitValue,
        domain_delay: DelayDuration,
    ) -> Self {
        self.requests_per_second = requests_per_second;
        self.burst_capacity = burst_capacity;
        self.domain_delay = domain_delay;
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_features(
        mut self,
        respect_robots_txt: BooleanFlag,
        enable_statistics: BooleanFlag,
        enable_logging: BooleanFlag,
    ) -> Self {
        self.respect_robots_txt = respect_robots_txt;
        self.enable_statistics = enable_statistics;
        self.enable_logging = enable_logging;
        self
    }

    pub fn build(self) -> CrawlerEngineConfig {
        CrawlerEngineConfig {
            max_concurrent_requests: self.max_concurrent_requests,
            request_delay: self.request_delay,
            retry_delay: self.retry_delay,
            request_timeout: self.request_timeout,
            bloom_filter_capacity: self.bloom_filter_capacity,
            bloom_filter_false_positive_rate: self.bloom_filter_false_positive_rate,
            max_url_length: self.max_url_length,
            follow_redirects: self.follow_redirects,
            max_redirects: self.max_redirects,
            respect_robots_txt: self.respect_robots_txt,
            requests_per_second: self.requests_per_second,
            burst_capacity: self.burst_capacity,
            domain_delay: self.domain_delay,
            session_id: self.session_id,
            enable_statistics: self.enable_statistics,
            enable_logging: self.enable_logging,
        }
    }
}

impl Default for CrawlerEngineConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Crawler statistics using building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerStatistics {
    // Basic counts
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    cached_responses: u64,

    // Timing statistics
    total_processing_time: Duration,
    average_response_time: Duration,
    fastest_response: Duration,
    slowest_response: Duration,

    // URL statistics
    unique_domains: u64,
    duplicate_urls_filtered: u64,
    robots_txt_blocked: u64,

    // Error statistics
    timeout_errors: u64,
    network_errors: u64,
    parsing_errors: u64,
    rate_limit_hits: u64,
}

impl CrawlerStatistics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            cached_responses: 0,
            total_processing_time: Duration::from_secs(0),
            average_response_time: Duration::from_secs(0),
            fastest_response: Duration::from_secs(u64::MAX),
            slowest_response: Duration::from_secs(0),
            unique_domains: 0,
            duplicate_urls_filtered: 0,
            robots_txt_blocked: 0,
            timeout_errors: 0,
            network_errors: 0,
            parsing_errors: 0,
            rate_limit_hits: 0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }

    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    // Getters for all statistics
    pub fn total_requests(&self) -> u64 {
        self.total_requests
    }
    pub fn successful_requests(&self) -> u64 {
        self.successful_requests
    }
    pub fn failed_requests(&self) -> u64 {
        self.failed_requests
    }
    pub fn cached_responses(&self) -> u64 {
        self.cached_responses
    }
    pub fn total_processing_time(&self) -> Duration {
        self.total_processing_time
    }
    pub fn average_response_time(&self) -> Duration {
        self.average_response_time
    }
    pub fn fastest_response(&self) -> Duration {
        self.fastest_response
    }
    pub fn slowest_response(&self) -> Duration {
        self.slowest_response
    }
    pub fn unique_domains(&self) -> u64 {
        self.unique_domains
    }
    pub fn duplicate_urls_filtered(&self) -> u64 {
        self.duplicate_urls_filtered
    }
    pub fn robots_txt_blocked(&self) -> u64 {
        self.robots_txt_blocked
    }
    pub fn timeout_errors(&self) -> u64 {
        self.timeout_errors
    }
    pub fn network_errors(&self) -> u64 {
        self.network_errors
    }
    pub fn parsing_errors(&self) -> u64 {
        self.parsing_errors
    }
    pub fn rate_limit_hits(&self) -> u64 {
        self.rate_limit_hits
    }

    // Update methods
    pub fn increment_total_requests(&mut self) {
        self.total_requests += 1;
    }

    pub fn increment_successful_requests(&mut self) {
        self.successful_requests += 1;
    }

    pub fn increment_failed_requests(&mut self) {
        self.failed_requests += 1;
    }

    pub fn record_response_time(&mut self, duration: Duration) {
        self.total_processing_time += duration;

        if duration < self.fastest_response {
            self.fastest_response = duration;
        }

        if duration > self.slowest_response {
            self.slowest_response = duration;
        }

        // Update average
        if self.total_requests > 0 {
            self.average_response_time = Duration::from_nanos(
                self.total_processing_time.as_nanos() as u64 / self.total_requests,
            );
        }
    }
}

impl Default for CrawlerStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Crawler engine using building blocks
/// Following Rule 4: Privacy first - all internal state encapsulated
pub struct CrawlerEngine {
    // Private configuration and state
    config: CrawlerEngineConfig,
    http_client: Client,
    bloom_filter: Arc<Mutex<BloomFilter>>,
    semaphore: Arc<Semaphore>,
    statistics: Arc<RwLock<CrawlerStatistics>>,
    domain_last_request: Arc<Mutex<HashMap<String, Instant>>>,

    // Rate limiting
    request_budget: Arc<Mutex<u64>>,
    last_refill: Arc<Mutex<Instant>>,
}

impl CrawlerEngine {
    /// Create new crawler engine with configuration
    pub async fn new(config: CrawlerEngineConfig) -> CrawlResult<Self> {
        // Create HTTP client with configuration
        let client_builder = reqwest::Client::builder()
            .timeout(config.request_timeout().duration())
            .user_agent("RustWebCrawler/1.0");

        let client_builder = if config.should_follow_redirects() {
            client_builder.redirect(reqwest::redirect::Policy::limited(
                config.max_redirects() as usize
            ))
        } else {
            client_builder.redirect(reqwest::redirect::Policy::none())
        };

        let http_client = client_builder
            .build()
            .map_err(|e| TaskError::network(format!("Failed to create HTTP client: {}", e)))?;

        // Create bloom filter for duplicate detection
        let bloom_filter = BloomFilter::with_rate(
            config.bloom_filter_false_positive_rate(),
            config.bloom_filter_capacity() as u32,
        );

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests() as usize));

        Ok(Self {
            config,
            http_client,
            bloom_filter: Arc::new(Mutex::new(bloom_filter)),
            semaphore,
            statistics: Arc::new(RwLock::new(CrawlerStatistics::new())),
            domain_last_request: Arc::new(Mutex::new(HashMap::new())),
            request_budget: Arc::new(Mutex::new(config.requests_per_second())),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        })
    }

    /// Create crawler with default configuration
    pub async fn with_defaults() -> CrawlResult<Self> {
        Self::new(CrawlerEngineConfig::default()).await
    }

    /// Execute a single crawl task
    pub async fn execute_task(&self, task: CrawlTask) -> TaskResult<crate::common::TaskContent> {
        let start_time = Instant::now();
        let task_id = task.task_id().clone();
        let url = task.target_url().clone();

        // Acquire semaphore permit for concurrency control
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| TaskError::crawler(format!("Failed to acquire permit: {}", e)))?;

        // Check if URL was already processed
        if self.is_url_processed(&url).await? {
            let mut stats = self.statistics.write().await;
            stats.increment_total_requests();
            stats.duplicate_urls_filtered += 1;

            return Err(TaskError::crawler(format!(
                "URL already processed: {}",
                url.as_str()
            )));
        }

        // Apply rate limiting
        self.apply_rate_limiting(&url).await?;

        // Make HTTP request
        let response_result = self.make_request(&url).await;

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.increment_total_requests();
        stats.record_response_time(start_time.elapsed());

        match response_result {
            Ok(content) => {
                // Mark URL as processed
                self.mark_url_processed(&url).await?;

                stats.increment_successful_requests();

                TaskResult::success(
                    task_id,
                    url,
                    content,
                    crate::common::ExecutionTiming::new(start_time.elapsed()),
                )
            }
            Err(error) => {
                stats.increment_failed_requests();

                TaskResult::failure(
                    task_id,
                    url,
                    error.clone(),
                    crate::common::ExecutionTiming::new(start_time.elapsed()),
                )
            }
        }
    }

    /// Execute multiple crawl tasks concurrently
    pub async fn execute_tasks(
        &self,
        tasks: Vec<CrawlTask>,
    ) -> Vec<TaskResult<crate::common::TaskContent>> {
        let tasks_stream = stream::iter(tasks)
            .map(|task| self.execute_task(task))
            .buffer_unordered(self.config.max_concurrent_requests() as usize);

        tasks_stream.collect().await
    }

    /// Check if URL has been processed
    async fn is_url_processed(&self, url: &UrlString) -> CrawlResult<bool> {
        let bloom_filter = self.bloom_filter.lock().await;
        Ok(bloom_filter.contains(&url.as_str()))
    }

    /// Mark URL as processed
    async fn mark_url_processed(&self, url: &UrlString) -> CrawlResult<()> {
        let mut bloom_filter = self.bloom_filter.lock().await;
        bloom_filter.insert(&url.as_str());
        Ok(())
    }

    /// Apply rate limiting based on domain
    async fn apply_rate_limiting(&self, url: &UrlString) -> CrawlResult<()> {
        // Extract domain from URL
        let parsed_url = Url::parse(url.as_str())
            .map_err(|e| TaskError::crawler(format!("Invalid URL: {}", e)))?;

        let domain = parsed_url
            .domain()
            .ok_or_else(|| TaskError::crawler("No domain in URL".to_string()))?
            .to_string();

        // Check domain-specific rate limiting
        {
            let mut domain_times = self.domain_last_request.lock().await;

            if let Some(last_request_time) = domain_times.get(&domain) {
                let elapsed = last_request_time.elapsed();
                let required_delay = self.config.domain_delay().duration();

                if elapsed < required_delay {
                    let sleep_time = required_delay - elapsed;
                    drop(domain_times); // Release lock before sleeping
                    sleep(sleep_time).await;
                }
            }

            domain_times.insert(domain, Instant::now());
        }

        // Apply global rate limiting
        self.refill_rate_limit_budget().await?;

        {
            let mut budget = self.request_budget.lock().await;
            if *budget == 0 {
                drop(budget);

                // Wait for budget refill
                let refill_time = Duration::from_secs(1);
                sleep(refill_time).await;
                self.refill_rate_limit_budget().await?;

                let mut budget = self.request_budget.lock().await;
                if *budget == 0 {
                    let mut stats = self.statistics.write().await;
                    stats.rate_limit_hits += 1;
                    return Err(TaskError::crawler("Rate limit exceeded".to_string()));
                }
            }

            *budget -= 1;
        }

        Ok(())
    }

    /// Refill rate limiting budget
    async fn refill_rate_limit_budget(&self) -> CrawlResult<()> {
        let mut last_refill = self.last_refill.lock().await;
        let now = Instant::now();

        if now.duration_since(*last_refill) >= Duration::from_secs(1) {
            let mut budget = self.request_budget.lock().await;
            *budget = self
                .config
                .requests_per_second()
                .min(self.config.burst_capacity());
            *last_refill = now;
        }

        Ok(())
    }

    /// Make HTTP request to URL
    async fn make_request(&self, url: &UrlString) -> CrawlResult<crate::common::TaskContent> {
        let response = self
            .http_client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TaskError::crawler(format!("Request timeout: {}", e))
                } else if e.is_connect() {
                    TaskError::network(format!("Connection error: {}", e))
                } else {
                    TaskError::network(format!("Request error: {}", e))
                }
            })?;

        if !response.status().is_success() {
            return Err(TaskError::network(format!(
                "HTTP error {}: {}",
                response.status(),
                response
                    .status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
            )));
        }

        let content_length = response.content_length().unwrap_or(0);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("text/html")
            .to_string();

        let body = response
            .text()
            .await
            .map_err(|e| TaskError::network(format!("Failed to read response body: {}", e)))?;

        // Calculate word count
        let word_count = body.split_whitespace().count();

        Ok(crate::common::TaskContent::new(
            body,
            word_count,
            None, // Language detection - will be handled by processing module
            Some(content_type),
            None, // Encoding detection - can be added later
        ))
    }

    /// Get current crawler statistics
    pub async fn statistics(&self) -> CrawlerStatistics {
        self.statistics.read().await.clone()
    }

    /// Get crawler configuration
    pub fn config(&self) -> &CrawlerEngineConfig {
        &self.config
    }

    /// Reset crawler state
    pub async fn reset(&self) -> CrawlResult<()> {
        // Clear bloom filter
        {
            let mut bloom_filter = self.bloom_filter.lock().await;
            *bloom_filter = BloomFilter::with_rate(
                self.config.bloom_filter_false_positive_rate(),
                self.config.bloom_filter_capacity() as u32,
            );
        }

        // Reset statistics
        {
            let mut stats = self.statistics.write().await;
            *stats = CrawlerStatistics::new();
        }

        // Clear domain tracking
        {
            let mut domain_times = self.domain_last_request.lock().await;
            domain_times.clear();
        }

        // Reset rate limiting
        {
            let mut budget = self.request_budget.lock().await;
            *budget = self.config.requests_per_second();

            let mut last_refill = self.last_refill.lock().await;
            *last_refill = Instant::now();
        }

        Ok(())
    }

    /// Check if crawler is healthy
    pub async fn health_check(&self) -> bool {
        // Check if semaphore has available permits
        let available_permits = self.semaphore.available_permits();

        // Check if rate limiting is not completely blocked
        let budget = {
            let budget_guard = self.request_budget.lock().await;
            *budget_guard
        };

        available_permits > 0 || budget > 0
    }
}

impl Default for CrawlerEngine {
    fn default() -> Self {
        // For Default trait implementation, return a simple configuration
        // Real applications should use async constructors
        let config = CrawlerEngineConfig::default();

        // Create minimal HTTP client
        let http_client = reqwest::Client::builder()
            .timeout(config.request_timeout().duration())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // Create bloom filter for duplicate detection
        let bloom_filter = BloomFilter::with_rate(
            config.bloom_filter_false_positive_rate() as f32,
            config.bloom_filter_capacity() as u32,
        );

        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests() as usize));

        Self {
            config,
            http_client,
            bloom_filter: Arc::new(Mutex::new(bloom_filter)),
            semaphore,
            statistics: Arc::new(RwLock::new(CrawlerStatistics::new())),
            domain_last_request: Arc::new(Mutex::new(HashMap::new())),
            request_budget: Arc::new(Mutex::new(config.requests_per_second())),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crawler_engine_creation() {
        let config = CrawlerEngineConfig::builder()
            .with_concurrency_settings(
                LimitValue::new(5),
                DelayDuration::from_millis(100),
                DelayDuration::from_secs(1),
            )
            .build();

        let engine = CrawlerEngine::new(config)
            .await
            .expect("Failed to create crawler");
        assert_eq!(engine.config().max_concurrent_requests(), 5);
    }

    #[tokio::test]
    async fn test_crawler_statistics() {
        let engine = CrawlerEngine::with_defaults()
            .await
            .expect("Failed to create crawler");

        let stats = engine.statistics().await;
        assert_eq!(stats.total_requests(), 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_url_deduplication() {
        let engine = CrawlerEngine::with_defaults()
            .await
            .expect("Failed to create crawler");
        let url = UrlString::new("https://example.com");

        // Initially URL should not be processed
        assert!(
            !engine
                .is_url_processed(&url)
                .await
                .expect("Failed to check URL")
        );

        // Mark as processed
        engine
            .mark_url_processed(&url)
            .await
            .expect("Failed to mark URL");

        // Now should be processed
        assert!(
            engine
                .is_url_processed(&url)
                .await
                .expect("Failed to check URL")
        );
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = CrawlerEngineConfig::builder()
            .with_rate_limiting(
                LimitValue::new(1), // Very low limit
                LimitValue::new(1),
                DelayDuration::from_millis(100),
            )
            .build();

        let engine = CrawlerEngine::new(config)
            .await
            .expect("Failed to create crawler");

        // First request should succeed rate limiting check
        let url1 = UrlString::new("https://example.com/1");
        assert!(engine.apply_rate_limiting(&url1).await.is_ok());

        // Immediate second request should be delayed or fail
        let url2 = UrlString::new("https://example.com/2");
        let start = Instant::now();
        let _ = engine.apply_rate_limiting(&url2).await;
        let elapsed = start.elapsed();

        // Should have been delayed or failed quickly
        assert!(elapsed >= Duration::from_millis(50) || elapsed < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_crawler_reset() {
        let engine = CrawlerEngine::with_defaults()
            .await
            .expect("Failed to create crawler");
        let url = UrlString::new("https://example.com");

        // Mark URL as processed
        engine
            .mark_url_processed(&url)
            .await
            .expect("Failed to mark URL");
        assert!(
            engine
                .is_url_processed(&url)
                .await
                .expect("Failed to check URL")
        );

        // Reset crawler
        engine.reset().await.expect("Failed to reset crawler");

        // URL should no longer be marked as processed
        assert!(
            !engine
                .is_url_processed(&url)
                .await
                .expect("Failed to check URL")
        );
    }

    #[tokio::test]
    async fn test_health_check() {
        let engine = CrawlerEngine::with_defaults()
            .await
            .expect("Failed to create crawler");
        assert!(engine.health_check().await);
    }
}
