/// Error handling module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all error configurations external
/// Following Rule 4: Privacy first - controlled error information access
/// Following Rule 6: Feature-based organization - errors categorized by functionality
/// Following Rule 8: Idiomatic Rust - Result<T,E>, proper error handling
use crate::common::{ConfigResult, ProcessingResult, TaskError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enhanced error configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorConfig {
    // Error reporting settings
    include_stack_traces: bool,
    include_request_details: bool,
    include_response_headers: bool,

    // Error categorization
    categorize_network_errors: bool,
    categorize_content_errors: bool,
    categorize_policy_errors: bool,

    // Error limits
    max_error_message_length: usize,
    max_error_context_depth: usize,
}

impl ErrorConfig {
    pub fn builder() -> ErrorConfigBuilder {
        ErrorConfigBuilder::new()
    }

    // Getters for configuration
    pub fn should_include_stack_traces(&self) -> bool {
        self.include_stack_traces
    }
    pub fn should_include_request_details(&self) -> bool {
        self.include_request_details
    }
    pub fn should_include_response_headers(&self) -> bool {
        self.include_response_headers
    }
    pub fn should_categorize_network_errors(&self) -> bool {
        self.categorize_network_errors
    }
    pub fn should_categorize_content_errors(&self) -> bool {
        self.categorize_content_errors
    }
    pub fn should_categorize_policy_errors(&self) -> bool {
        self.categorize_policy_errors
    }
    pub fn max_error_message_length(&self) -> usize {
        self.max_error_message_length
    }
    pub fn max_error_context_depth(&self) -> usize {
        self.max_error_context_depth
    }
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            include_stack_traces: false,
            include_request_details: true,
            include_response_headers: false,
            categorize_network_errors: true,
            categorize_content_errors: true,
            categorize_policy_errors: true,
            max_error_message_length: 1000,
            max_error_context_depth: 5,
        }
    }
}

/// Builder for error configuration
#[derive(Debug)]
pub struct ErrorConfigBuilder {
    include_stack_traces: bool,
    include_request_details: bool,
    include_response_headers: bool,
    categorize_network_errors: bool,
    categorize_content_errors: bool,
    categorize_policy_errors: bool,
    max_error_message_length: usize,
    max_error_context_depth: usize,
}

impl ErrorConfigBuilder {
    pub fn new() -> Self {
        let default_config = ErrorConfig::default();
        Self {
            include_stack_traces: default_config.include_stack_traces,
            include_request_details: default_config.include_request_details,
            include_response_headers: default_config.include_response_headers,
            categorize_network_errors: default_config.categorize_network_errors,
            categorize_content_errors: default_config.categorize_content_errors,
            categorize_policy_errors: default_config.categorize_policy_errors,
            max_error_message_length: default_config.max_error_message_length,
            max_error_context_depth: default_config.max_error_context_depth,
        }
    }

    pub fn with_reporting_options(
        mut self,
        stack_traces: bool,
        request_details: bool,
        response_headers: bool,
    ) -> Self {
        self.include_stack_traces = stack_traces;
        self.include_request_details = request_details;
        self.include_response_headers = response_headers;
        self
    }

    pub fn with_categorization_options(
        mut self,
        network: bool,
        content: bool,
        policy: bool,
    ) -> Self {
        self.categorize_network_errors = network;
        self.categorize_content_errors = content;
        self.categorize_policy_errors = policy;
        self
    }

    pub fn with_limits(mut self, max_message_length: usize, max_context_depth: usize) -> Self {
        self.max_error_message_length = max_message_length;
        self.max_error_context_depth = max_context_depth;
        self
    }

    pub fn build(self) -> ErrorConfig {
        ErrorConfig {
            include_stack_traces: self.include_stack_traces,
            include_request_details: self.include_request_details,
            include_response_headers: self.include_response_headers,
            categorize_network_errors: self.categorize_network_errors,
            categorize_content_errors: self.categorize_content_errors,
            categorize_policy_errors: self.categorize_policy_errors,
            max_error_message_length: self.max_error_message_length,
            max_error_context_depth: self.max_error_context_depth,
        }
    }
}

impl Default for ErrorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Error severity levels for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,      // Recoverable errors, warnings
    Medium,   // Processing errors that can be retried
    High,     // Network errors, timeouts
    Critical, // System errors, configuration errors
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
            ErrorSeverity::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(ErrorSeverity::Low),
            "medium" => Some(ErrorSeverity::Medium),
            "high" => Some(ErrorSeverity::High),
            "critical" => Some(ErrorSeverity::Critical),
            _ => None,
        }
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    url: Option<String>,
    domain: Option<String>,
    request_method: Option<String>,
    response_status: Option<u16>,
    response_headers: Option<std::collections::HashMap<String, String>>,
    user_agent: Option<String>,
    timestamp: std::time::SystemTime,
    retry_count: u32,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            url: None,
            domain: None,
            request_method: None,
            response_status: None,
            response_headers: None,
            user_agent: None,
            timestamp: std::time::SystemTime::now(),
            retry_count: 0,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    pub fn with_request_method(mut self, method: impl Into<String>) -> Self {
        self.request_method = Some(method.into());
        self
    }

    pub fn with_response_status(mut self, status: u16) -> Self {
        self.response_status = Some(status);
        self
    }

    pub fn with_response_headers(
        mut self,
        headers: std::collections::HashMap<String, String>,
    ) -> Self {
        self.response_headers = Some(headers);
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    // Getters for all fields
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }
    pub fn request_method(&self) -> Option<&str> {
        self.request_method.as_deref()
    }
    pub fn response_status(&self) -> Option<u16> {
        self.response_status
    }
    pub fn response_headers(&self) -> Option<&std::collections::HashMap<String, String>> {
        self.response_headers.as_ref()
    }
    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }
    pub fn timestamp(&self) -> std::time::SystemTime {
        self.timestamp
    }
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced crawl error with building blocks integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlError {
    // Error identification
    error_type: String,
    error_message: String,
    error_severity: ErrorSeverity,

    // Error context
    context: ErrorContext,

    // Error chain
    source_error: Option<String>,
    caused_by: Vec<String>,

    // Metadata
    is_retryable: bool,
    suggested_action: Option<String>,
}

impl CrawlError {
    /// Create a new crawl error
    pub fn new(
        error_type: impl Into<String>,
        message: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        Self {
            error_type: error_type.into(),
            error_message: message.into(),
            error_severity: severity,
            context: ErrorContext::new(),
            source_error: None,
            caused_by: Vec::new(),
            is_retryable: false,
            suggested_action: None,
        }
    }

    /// Network error constructors
    pub fn network_timeout(context: ErrorContext) -> Self {
        Self::new(
            "network_timeout",
            "Network request timed out",
            ErrorSeverity::High,
        )
        .with_context(context)
        .with_retryable(true)
        .with_suggested_action("Retry with longer timeout or check network connectivity")
    }

    pub fn connection_refused(url: &str) -> Self {
        let context = ErrorContext::new().with_url(url);
        Self::new(
            "connection_refused",
            "Connection refused by server",
            ErrorSeverity::High,
        )
        .with_context(context)
        .with_retryable(true)
        .with_suggested_action("Check if server is running and accessible")
    }

    pub fn dns_resolution_failed(domain: &str) -> Self {
        let context = ErrorContext::new().with_domain(domain);
        Self::new(
            "dns_resolution_failed",
            format!("Failed to resolve domain: {}", domain),
            ErrorSeverity::High,
        )
        .with_context(context)
        .with_retryable(true)
        .with_suggested_action("Check domain name or DNS server configuration")
    }

    /// HTTP error constructors
    pub fn http_error(status: u16, context: ErrorContext) -> Self {
        let message = match status {
            400 => "Bad Request".to_string(),
            401 => "Unauthorized".to_string(),
            403 => "Forbidden".to_string(),
            404 => "Not Found".to_string(),
            429 => "Too Many Requests".to_string(),
            500 => "Internal Server Error".to_string(),
            502 => "Bad Gateway".to_string(),
            503 => "Service Unavailable".to_string(),
            504 => "Gateway Timeout".to_string(),
            _ => format!("HTTP Error {}", status),
        };

        let severity = match status {
            400..=499 => ErrorSeverity::Medium,
            500..=599 => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        };

        let retryable = matches!(status, 408 | 429 | 500..=599);

        Self::new("http_error", message, severity)
            .with_context(context.with_response_status(status))
            .with_retryable(retryable)
    }

    /// Content error constructors
    pub fn parsing_error(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::new("parsing_error", message, ErrorSeverity::Medium)
            .with_context(context)
            .with_retryable(false)
            .with_suggested_action("Check content format or parsing rules")
    }

    pub fn encoding_error(encoding: &str, context: ErrorContext) -> Self {
        Self::new(
            "encoding_error",
            format!("Unsupported encoding: {}", encoding),
            ErrorSeverity::Low,
        )
        .with_context(context)
        .with_retryable(false)
    }

    /// Policy error constructors
    pub fn robots_blocked(url: &str) -> Self {
        let context = ErrorContext::new().with_url(url);
        Self::new(
            "robots_blocked",
            "Blocked by robots.txt",
            ErrorSeverity::Low,
        )
        .with_context(context)
        .with_retryable(false)
        .with_suggested_action("Respect robots.txt or request permission")
    }

    pub fn rate_limited(context: ErrorContext) -> Self {
        Self::new(
            "rate_limited",
            "Request rate limit exceeded",
            ErrorSeverity::Medium,
        )
        .with_context(context)
        .with_retryable(true)
        .with_suggested_action("Reduce request rate or implement exponential backoff")
    }

    /// System error constructors
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::new("configuration_error", message, ErrorSeverity::Critical)
            .with_retryable(false)
            .with_suggested_action("Check configuration settings")
    }

    pub fn system_error(message: impl Into<String>) -> Self {
        Self::new("system_error", message, ErrorSeverity::Critical)
            .with_retryable(false)
            .with_suggested_action("Check system resources and permissions")
    }

    /// Builder pattern methods
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_source_error(mut self, source: impl Into<String>) -> Self {
        self.source_error = Some(source.into());
        self
    }

    pub fn with_caused_by(mut self, caused_by: Vec<String>) -> Self {
        self.caused_by = caused_by;
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.is_retryable = retryable;
        self
    }

    pub fn with_suggested_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }

    /// Truncate error message to configured length
    pub fn truncate_message(&mut self, config: &ErrorConfig) {
        if self.error_message.len() > config.max_error_message_length() {
            self.error_message
                .truncate(config.max_error_message_length() - 3);
            self.error_message.push_str("...");
        }
    }

    // Getters for all fields
    pub fn error_type(&self) -> &str {
        &self.error_type
    }
    pub fn error_message(&self) -> &str {
        &self.error_message
    }
    pub fn error_severity(&self) -> ErrorSeverity {
        self.error_severity
    }
    pub fn context(&self) -> &ErrorContext {
        &self.context
    }
    pub fn source_error(&self) -> Option<&str> {
        self.source_error.as_deref()
    }
    pub fn caused_by(&self) -> &[String] {
        &self.caused_by
    }
    pub fn is_retryable(&self) -> bool {
        self.is_retryable
    }
    pub fn suggested_action(&self) -> Option<&str> {
        self.suggested_action.as_deref()
    }

    /// Get formatted error report
    pub fn format_report(&self, config: &ErrorConfig) -> String {
        let mut report = format!("{}: {}", self.error_type, self.error_message);

        if config.should_include_request_details() {
            if let Some(url) = self.context.url() {
                report.push_str(&format!("\n  URL: {}", url));
            }
            if let Some(method) = self.context.request_method() {
                report.push_str(&format!("\n  Method: {}", method));
            }
            if let Some(status) = self.context.response_status() {
                report.push_str(&format!("\n  Status: {}", status));
            }
        }

        if config.should_include_response_headers() {
            if let Some(headers) = self.context.response_headers() {
                report.push_str("\n  Headers:");
                for (key, value) in headers.iter().take(5) {
                    // Limit headers
                    report.push_str(&format!("\n    {}: {}", key, value));
                }
            }
        }

        if !self.caused_by.is_empty() {
            report.push_str("\n  Caused by:");
            for (i, cause) in self
                .caused_by
                .iter()
                .enumerate()
                .take(config.max_error_context_depth())
            {
                report.push_str(&format!("\n    {}: {}", i + 1, cause));
            }
        }

        if let Some(action) = &self.suggested_action {
            report.push_str(&format!("\n  Suggested action: {}", action));
        }

        report
    }
}

impl fmt::Display for CrawlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.error_message)
    }
}

impl std::error::Error for CrawlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None // Could be enhanced to store actual source errors
    }
}

/// Convert CrawlError to TaskError for integration with building blocks
impl From<CrawlError> for TaskError {
    fn from(crawl_error: CrawlError) -> Self {
        match crawl_error.error_type.as_str() {
            "network_timeout" | "connection_refused" | "dns_resolution_failed" => {
                TaskError::network(crawl_error.error_message)
            }
            "http_error" => {
                TaskError::parse(crawl_error.error_message) // Using parse as closest match
            }
            "parsing_error" | "encoding_error" => TaskError::parse(crawl_error.error_message),
            "configuration_error" | "system_error" => {
                TaskError::configuration(crawl_error.error_message)
            }
            "robots_blocked" | "rate_limited" => {
                TaskError::configuration(crawl_error.error_message)
            }
            _ => TaskError::internal(crawl_error.error_message),
        }
    }
}

/// Error handler with configuration support
pub struct ErrorHandler {
    config: ErrorConfig,
}

impl ErrorHandler {
    pub fn new(config: ErrorConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ErrorConfig::default())
    }

    /// Process and format error according to configuration
    pub fn process_error(&self, mut error: CrawlError) -> CrawlError {
        error.truncate_message(&self.config);
        error
    }

    /// Convert error to ProcessingResult
    pub fn to_processing_result<T>(&self, error: CrawlError) -> ProcessingResult<T> {
        Err(TaskError::from(error))
    }

    /// Create error from standard error types
    pub fn from_std_error(
        &self,
        error: &dyn std::error::Error,
        context: ErrorContext,
    ) -> CrawlError {
        CrawlError::new("std_error", error.to_string(), ErrorSeverity::Medium)
            .with_context(context)
            .with_source_error(error.to_string())
    }

    /// Get configuration
    pub fn config(&self) -> &ErrorConfig {
        &self.config
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_config_builder() {
        let config = ErrorConfig::builder()
            .with_reporting_options(true, true, false)
            .with_categorization_options(true, false, true)
            .with_limits(500, 3)
            .build();

        assert!(config.should_include_stack_traces());
        assert!(config.should_include_request_details());
        assert!(!config.should_include_response_headers());
        assert_eq!(config.max_error_message_length(), 500);
        assert_eq!(config.max_error_context_depth(), 3);
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(ErrorSeverity::Low.as_str(), "low");
        assert_eq!(ErrorSeverity::Critical.as_str(), "critical");
        assert_eq!(
            ErrorSeverity::from_str("medium"),
            Some(ErrorSeverity::Medium)
        );
        assert_eq!(ErrorSeverity::from_str("invalid"), None);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new()
            .with_url("https://example.com")
            .with_domain("example.com")
            .with_response_status(404);

        assert_eq!(context.url(), Some("https://example.com"));
        assert_eq!(context.domain(), Some("example.com"));
        assert_eq!(context.response_status(), Some(404));
    }

    #[test]
    fn test_crawl_error_creation() {
        let context = ErrorContext::new().with_url("https://example.com");
        let error = CrawlError::connection_refused("https://example.com");

        assert_eq!(error.error_type(), "connection_refused");
        assert_eq!(error.error_severity(), ErrorSeverity::High);
        assert!(error.is_retryable());
        assert!(error.suggested_action().is_some());
    }

    #[test]
    fn test_http_error_creation() {
        let context = ErrorContext::new().with_url("https://example.com");
        let error = CrawlError::http_error(404, context);

        assert_eq!(error.error_type(), "http_error");
        assert_eq!(error.error_severity(), ErrorSeverity::Medium);
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_handler() {
        let config = ErrorConfig::builder().with_limits(50, 2).build();

        let handler = ErrorHandler::new(config);
        let mut error = CrawlError::new("test", "a".repeat(100), ErrorSeverity::Low);

        error = handler.process_error(error);
        assert!(error.error_message().len() <= 50);
    }

    #[test]
    fn test_error_to_task_error_conversion() {
        let crawl_error = CrawlError::network_timeout(ErrorContext::new());
        let task_error = TaskError::from(crawl_error);

        assert_eq!(task_error.error_type(), "network");
    }
}
