// Structured logging events for crawler operations
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};
use url::Url;

/// Comprehensive crawl event logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlEvent {
    pub url: String,
    pub event_type: CrawlEventType,
    pub timestamp: SystemTime,
    pub duration_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub content_length: Option<u64>,
    pub word_count: Option<usize>,
    pub language: Option<String>,
    pub depth: Option<u32>,
    pub retry_count: Option<u32>,
    pub user_agent: Option<String>,
    pub proxy_used: Option<String>,
    pub error_message: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrawlEventType {
    Started,
    Completed,
    Failed,
    Retrying,
    Blocked, // By robots.txt
    RateLimited,
    Cached, // Content was cached
    Redirected,
    Timeout,
}

/// Performance monitoring events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEvent {
    pub event_type: PerformanceEventType,
    pub timestamp: SystemTime,
    pub duration_ms: u64,
    pub memory_usage_mb: Option<u64>,
    pub cpu_usage_percent: Option<f64>,
    pub active_connections: Option<usize>,
    pub queue_size: Option<usize>,
    pub cache_hit_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceEventType {
    RequestProcessed,
    QueueProcessed,
    CacheOperation,
    DnsResolution,
    RobotsTxtCheck,
    ContentExtraction,
    LinkExtraction,
    SessionComplete,
}

/// Error event logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub error_type: ErrorType,
    pub timestamp: SystemTime,
    pub url: Option<String>,
    pub error_message: String,
    pub error_code: Option<String>,
    pub context: Option<String>,
    pub retry_count: Option<u32>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    NetworkError,
    HttpError,
    ParseError,
    ConfigError,
    RateLimitError,
    TimeoutError,
    ValidationError,
    SystemError,
}

/// Main crawler event logger
pub struct CrawlEventLogger {
    session_id: String,
}

impl CrawlEventLogger {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }

    /// Log crawl start event
    pub fn log_crawl_start(&self, url: &Url, depth: Option<u32>, user_agent: Option<&str>) {
        let event = CrawlEvent {
            url: url.to_string(),
            event_type: CrawlEventType::Started,
            timestamp: SystemTime::now(),
            duration_ms: None,
            status_code: None,
            content_length: None,
            word_count: None,
            language: None,
            depth,
            retry_count: None,
            user_agent: user_agent.map(|s| s.to_string()),
            proxy_used: None,
            error_message: None,
            session_id: Some(self.session_id.clone()),
        };

        info!(
            url = %url,
            depth = ?depth,
            user_agent = ?user_agent,
            session_id = %self.session_id,
            event = "crawl_started",
            "Started crawling URL"
        );

        debug!(event = ?event, "Detailed crawl start event");
    }

    /// Log successful crawl completion
    #[allow(clippy::too_many_arguments)]
    pub fn log_crawl_success(
        &self,
        url: &Url,
        duration: Duration,
        status_code: u16,
        content_length: u64,
        word_count: usize,
        language: Option<&str>,
        depth: Option<u32>,
        proxy_used: Option<&str>,
    ) {
        let event = CrawlEvent {
            url: url.to_string(),
            event_type: CrawlEventType::Completed,
            timestamp: SystemTime::now(),
            duration_ms: Some(duration.as_millis() as u64),
            status_code: Some(status_code),
            content_length: Some(content_length),
            word_count: Some(word_count),
            language: language.map(|s| s.to_string()),
            depth,
            retry_count: None,
            user_agent: None,
            proxy_used: proxy_used.map(|s| s.to_string()),
            error_message: None,
            session_id: Some(self.session_id.clone()),
        };

        info!(
            url = %url,
            duration_ms = duration.as_millis(),
            status_code = status_code,
            content_length = content_length,
            word_count = word_count,
            language = ?language,
            depth = ?depth,
            proxy_used = ?proxy_used,
            session_id = %self.session_id,
            event = "crawl_completed",
            "Successfully crawled URL"
        );

        debug!(event = ?event, "Detailed crawl success event");
    }

    /// Log crawl failure
    pub fn log_crawl_failure(
        &self,
        url: &Url,
        duration: Duration,
        error_message: &str,
        retry_count: Option<u32>,
        depth: Option<u32>,
        will_retry: bool,
    ) {
        let event_type = if will_retry {
            CrawlEventType::Retrying
        } else {
            CrawlEventType::Failed
        };

        let event = CrawlEvent {
            url: url.to_string(),
            event_type: event_type.clone(),
            timestamp: SystemTime::now(),
            duration_ms: Some(duration.as_millis() as u64),
            status_code: None,
            content_length: None,
            word_count: None,
            language: None,
            depth,
            retry_count,
            user_agent: None,
            proxy_used: None,
            error_message: Some(error_message.to_string()),
            session_id: Some(self.session_id.clone()),
        };

        if will_retry {
            warn!(
                url = %url,
                duration_ms = duration.as_millis(),
                error_message = error_message,
                retry_count = ?retry_count,
                depth = ?depth,
                session_id = %self.session_id,
                event = "crawl_retrying",
                "Crawl failed, will retry"
            );
        } else {
            error!(
                url = %url,
                duration_ms = duration.as_millis(),
                error_message = error_message,
                retry_count = ?retry_count,
                depth = ?depth,
                session_id = %self.session_id,
                event = "crawl_failed",
                "Crawl failed permanently"
            );
        }

        debug!(event = ?event, "Detailed crawl failure event");
    }

    /// Log robots.txt blocking
    pub fn log_robots_blocked(&self, url: &Url, robots_url: &str) {
        let event = CrawlEvent {
            url: url.to_string(),
            event_type: CrawlEventType::Blocked,
            timestamp: SystemTime::now(),
            duration_ms: None,
            status_code: None,
            content_length: None,
            word_count: None,
            language: None,
            depth: None,
            retry_count: None,
            user_agent: None,
            proxy_used: None,
            error_message: Some(format!("Blocked by robots.txt: {}", robots_url)),
            session_id: Some(self.session_id.clone()),
        };

        warn!(
            url = %url,
            robots_url = robots_url,
            session_id = %self.session_id,
            event = "robots_blocked",
            "URL blocked by robots.txt"
        );

        debug!(event = ?event, "Detailed robots block event");
    }

    /// Log rate limiting
    pub fn log_rate_limited(&self, url: &Url, wait_time_ms: u64, domain: &str) {
        let event = CrawlEvent {
            url: url.to_string(),
            event_type: CrawlEventType::RateLimited,
            timestamp: SystemTime::now(),
            duration_ms: Some(wait_time_ms),
            status_code: None,
            content_length: None,
            word_count: None,
            language: None,
            depth: None,
            retry_count: None,
            user_agent: None,
            proxy_used: None,
            error_message: Some(format!("Rate limited for domain: {}", domain)),
            session_id: Some(self.session_id.clone()),
        };

        debug!(
            url = %url,
            wait_time_ms = wait_time_ms,
            domain = domain,
            session_id = %self.session_id,
            event = "rate_limited",
            "Request rate limited"
        );

        debug!(event = ?event, "Detailed rate limit event");
    }

    /// Log timeout events
    pub fn log_timeout(&self, url: &Url, timeout_duration: Duration) {
        let event = CrawlEvent {
            url: url.to_string(),
            event_type: CrawlEventType::Timeout,
            timestamp: SystemTime::now(),
            duration_ms: Some(timeout_duration.as_millis() as u64),
            status_code: None,
            content_length: None,
            word_count: None,
            language: None,
            depth: None,
            retry_count: None,
            user_agent: None,
            proxy_used: None,
            error_message: Some("Request timeout".to_string()),
            session_id: Some(self.session_id.clone()),
        };

        warn!(
            url = %url,
            timeout_duration_ms = timeout_duration.as_millis(),
            session_id = %self.session_id,
            event = "timeout",
            "Request timed out"
        );

        debug!(event = ?event, "Detailed timeout event");
    }

    /// Log performance metrics
    pub fn log_performance(
        &self,
        event_type: PerformanceEventType,
        duration: Duration,
        metadata: Option<&str>,
    ) {
        let event = PerformanceEvent {
            event_type: event_type.clone(),
            timestamp: SystemTime::now(),
            duration_ms: duration.as_millis() as u64,
            memory_usage_mb: None, // Could be implemented with system monitoring
            cpu_usage_percent: None,
            active_connections: None,
            queue_size: None,
            cache_hit_ratio: None,
        };

        debug!(
            event_type = ?event_type,
            duration_ms = duration.as_millis(),
            metadata = ?metadata,
            session_id = %self.session_id,
            event = "performance",
            "Performance metric recorded"
        );

        debug!(event = ?event, "Detailed performance event");
    }

    /// Log general errors
    pub fn log_error(
        &self,
        error_type: ErrorType,
        error_message: &str,
        url: Option<&Url>,
        context: Option<&str>,
    ) {
        let event = ErrorEvent {
            error_type: error_type.clone(),
            timestamp: SystemTime::now(),
            url: url.map(|u| u.to_string()),
            error_message: error_message.to_string(),
            error_code: None,
            context: context.map(|s| s.to_string()),
            retry_count: None,
            session_id: Some(self.session_id.clone()),
        };

        error!(
            error_type = ?error_type,
            error_message = error_message,
            url = ?url,
            context = ?context,
            session_id = %self.session_id,
            event = "error",
            "Error occurred during crawling"
        );

        debug!(event = ?event, "Detailed error event");
    }
}
