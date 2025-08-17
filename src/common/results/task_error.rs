use crate::common::primitives::{DomainString, UrlString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TaskError {
    #[error("Network error: {message}")]
    Network { message: String },
    #[error("HTTP error: status {status}, message: {message}")]
    Http { status: u16, message: String },
    #[error("Timeout error: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("Parse error: {message}")]
    Parse { message: String },
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    #[error("Rate limit exceeded for domain: {domain}")]
    RateLimitExceeded { domain: DomainString },
    #[error("Content validation failed: {reason}")]
    ContentValidation { reason: String },
    #[error("Retry limit exceeded: {attempts} attempts failed")]
    RetryLimitExceeded { attempts: u32 },
    #[error("Proxy error: {message}")]
    Proxy { message: String },
    #[error("DNS resolution failed for domain: {domain}")]
    DnsResolution { domain: DomainString },
    #[error("Robots.txt disallows crawling: {url}")]
    RobotsDisallowed { url: UrlString },
    #[error("Session error: {message}")]
    Session { message: String },
    #[error("Storage error: {message}")]
    Storage { message: String },
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl TaskError {
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http {
            status,
            message: message.into(),
        }
    }
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }
    pub fn rate_limit_exceeded(domain: DomainString) -> Self {
        Self::RateLimitExceeded { domain }
    }
    pub fn content_validation(reason: impl Into<String>) -> Self {
        Self::ContentValidation {
            reason: reason.into(),
        }
    }
    pub fn retry_limit_exceeded(attempts: u32) -> Self {
        Self::RetryLimitExceeded { attempts }
    }
    pub fn proxy(message: impl Into<String>) -> Self {
        Self::Proxy {
            message: message.into(),
        }
    }
    pub fn dns_resolution(domain: DomainString) -> Self {
        Self::DnsResolution { domain }
    }
    pub fn robots_disallowed(url: UrlString) -> Self {
        Self::RobotsDisallowed { url }
    }
    pub fn session(message: impl Into<String>) -> Self {
        Self::Session {
            message: message.into(),
        }
    }
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage {
            message: message.into(),
        }
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TaskError::Network { .. }
                | TaskError::Http {
                    status: 500..=599,
                    ..
                }
                | TaskError::Timeout { .. }
                | TaskError::Proxy { .. }
                | TaskError::DnsResolution { .. }
        )
    }
    pub fn severity(&self) -> super::error_severity::ErrorSeverity {
        match self {
            TaskError::Network { .. } | TaskError::Timeout { .. } | TaskError::Proxy { .. } => {
                super::error_severity::ErrorSeverity::High
            }
            TaskError::Http {
                status: 400..=499, ..
            }
            | TaskError::Parse { .. }
            | TaskError::ContentValidation { .. }
            | TaskError::RobotsDisallowed { .. } => super::error_severity::ErrorSeverity::Medium,
            TaskError::Http {
                status: 500..=599, ..
            }
            | TaskError::DnsResolution { .. } => super::error_severity::ErrorSeverity::High,
            TaskError::Http { .. } => super::error_severity::ErrorSeverity::Medium,
            TaskError::RateLimitExceeded { .. } => super::error_severity::ErrorSeverity::Low,
            TaskError::Configuration { .. }
            | TaskError::RetryLimitExceeded { .. }
            | TaskError::Session { .. }
            | TaskError::Storage { .. }
            | TaskError::Internal { .. } => super::error_severity::ErrorSeverity::Critical,
        }
    }
}

impl From<String> for TaskError {
    fn from(message: String) -> Self {
        Self::Internal { message }
    }
}
