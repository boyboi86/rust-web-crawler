/// Enhanced crawl error types with more specific categorization
#[derive(Debug, Clone)]
pub enum CrawlError {
    // Network-related errors
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
            CrawlError::NetworkTimeout
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
            CrawlError::NetworkTimeout | CrawlError::ConnectionRefused | CrawlError::ProxyError => {
                ErrorSeverity::High
            }
            CrawlError::HttpError(500..=599) | CrawlError::DnsResolutionFailed(_) => {
                ErrorSeverity::Medium
            }
            CrawlError::RateLimited => ErrorSeverity::Low,
            CrawlError::RedirectLoop | CrawlError::InvalidUrl(_) => ErrorSeverity::Medium,
            CrawlError::ContentTooShort | CrawlError::LanguageNotSupported => ErrorSeverity::Low,
            CrawlError::ParsingError | CrawlError::EncodingError => ErrorSeverity::Medium,
            CrawlError::RobotsBlocked | CrawlError::Forbidden => ErrorSeverity::Low,
            CrawlError::HttpError(_) => ErrorSeverity::Medium,
            CrawlError::UnknownError(_) => ErrorSeverity::Critical,
        }
    }

    /// Categorize a generic error into CrawlError
    pub fn from_anyhow_error(error: &anyhow::Error) -> Self {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("timeout") {
            CrawlError::NetworkTimeout
        } else if error_str.contains("connection refused") {
            CrawlError::ConnectionRefused
        } else if error_str.contains("dns") {
            CrawlError::DnsResolutionFailed(error.to_string())
        } else if error_str.contains("proxy") {
            CrawlError::ProxyError
        } else if error_str.contains("redirect") {
            CrawlError::RedirectLoop
        } else if error_str.contains("url") || error_str.contains("parse") {
            CrawlError::InvalidUrl(error.to_string())
        } else {
            CrawlError::UnknownError(error.to_string())
        }
    }
}
