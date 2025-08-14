/// Enhanced crawl error types with more specific categorization
#[derive(Debug, Clone)]
pub enum CrawlError {
    // Network-related errors
    NetworkError(String),
    NetworkTimeout,
    ConnectionRefused,
    DnsResolutionFailed(String),
    ProxyError,

    // HTTP-related errors
    HttpError(u16),
    RedirectLoop,
    InvalidUrl(String),

    // Content-related errors
    ContentTooShort,
    LanguageNotSupported,
    ParsingError,
    EncodingError,

    // Processing feature errors
    KeywordConfigError(String),
    KeywordNotFound,
    ExtensiveConfigError(String),
    CleaningConfigError(String),
    CleaningRuleError(String),

    // Policy-related errors
    RobotsBlocked,
    RateLimited,
    Forbidden,

    // System errors
    UnknownError(String),
}

impl std::fmt::Display for CrawlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrawlError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            CrawlError::NetworkTimeout => write!(f, "Network timeout"),
            CrawlError::ConnectionRefused => write!(f, "Connection refused"),
            CrawlError::DnsResolutionFailed(domain) => {
                write!(f, "DNS resolution failed for {}", domain)
            }
            CrawlError::ProxyError => write!(f, "Proxy connection error"),
            CrawlError::HttpError(code) => write!(f, "HTTP error: {}", code),
            CrawlError::RedirectLoop => write!(f, "Redirect loop detected"),
            CrawlError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            CrawlError::ContentTooShort => write!(f, "Content too short"),
            CrawlError::LanguageNotSupported => write!(f, "Language not supported"),
            CrawlError::ParsingError => write!(f, "HTML parsing error"),
            CrawlError::EncodingError => write!(f, "Text encoding error"),
            CrawlError::KeywordConfigError(msg) => {
                write!(f, "Keyword configuration error: {}", msg)
            }
            CrawlError::KeywordNotFound => write!(f, "Target keywords not found in content"),
            CrawlError::ExtensiveConfigError(msg) => {
                write!(f, "Extensive crawling configuration error: {}", msg)
            }
            CrawlError::CleaningConfigError(msg) => {
                write!(f, "Text cleaning configuration error: {}", msg)
            }
            CrawlError::CleaningRuleError(msg) => write!(f, "Text cleaning rule error: {}", msg),
            CrawlError::RobotsBlocked => write!(f, "Blocked by robots.txt"),
            CrawlError::RateLimited => write!(f, "Rate limited"),
            CrawlError::Forbidden => write!(f, "Access forbidden"),
            CrawlError::UnknownError(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for CrawlError {}

impl CrawlError {
    /// Check if this error type should trigger a retry
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CrawlError::NetworkError(_)
                | CrawlError::NetworkTimeout
                | CrawlError::ConnectionRefused
                | CrawlError::DnsResolutionFailed(_)
                | CrawlError::HttpError(500..=599)
                | CrawlError::ProxyError
                | CrawlError::RateLimited
        )
    }

    /// Get the severity level of the error
    pub fn severity(&self) -> crate::core::types::ErrorSeverity {
        use crate::core::types::ErrorSeverity;

        match self {
            CrawlError::NetworkError(_)
            | CrawlError::NetworkTimeout
            | CrawlError::ConnectionRefused
            | CrawlError::ProxyError => ErrorSeverity::High,
            CrawlError::HttpError(500..=599) | CrawlError::DnsResolutionFailed(_) => {
                ErrorSeverity::Medium
            }
            CrawlError::RateLimited => ErrorSeverity::Low,
            CrawlError::RedirectLoop | CrawlError::InvalidUrl(_) => ErrorSeverity::Medium,
            CrawlError::ContentTooShort | CrawlError::LanguageNotSupported => ErrorSeverity::Low,
            CrawlError::ParsingError | CrawlError::EncodingError => ErrorSeverity::Medium,
            CrawlError::KeywordConfigError(_)
            | CrawlError::ExtensiveConfigError(_)
            | CrawlError::CleaningConfigError(_) => ErrorSeverity::High,
            CrawlError::KeywordNotFound => ErrorSeverity::Low,
            CrawlError::CleaningRuleError(_) => ErrorSeverity::Medium,
            CrawlError::RobotsBlocked | CrawlError::Forbidden => ErrorSeverity::Low,
            CrawlError::HttpError(_) => ErrorSeverity::Medium,
            CrawlError::UnknownError(_) => ErrorSeverity::Critical,
        }
    }

    /// Categorize a generic error into CrawlError
    pub fn from_anyhow_error(error: &anyhow::Error) -> Self {
        let error_str = error.to_string().to_lowercase(); // Scoped variable for DRY
        let error_msg = error.to_string(); // Scoped variable for original message

        match error_str.as_str() {
            s if s.contains("timeout") => CrawlError::NetworkTimeout,
            s if s.contains("connection refused") => CrawlError::ConnectionRefused,
            s if s.contains("dns") => CrawlError::DnsResolutionFailed(error_msg),
            s if s.contains("proxy") => CrawlError::ProxyError,
            s if s.contains("redirect") => CrawlError::RedirectLoop,
            s if s.contains("url") || s.contains("parse") => CrawlError::InvalidUrl(error_msg),
            _ => CrawlError::UnknownError(error_msg),
        }
    }
}
