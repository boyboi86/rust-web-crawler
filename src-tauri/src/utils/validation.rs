// =============================================================================
// VALIDATION - Input Validation and Error Handling
// =============================================================================
// This module contains validation logic for request data and error handling.

use crate::core::CrawlRequest;
use url::Url;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Custom error type for validation failures
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
}

impl ValidationError {
    pub fn with_field(message: &str, field: &str) -> Self {
        Self {
            message: message.to_string(),
            field: Some(field.to_string()),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.field {
            Some(field) => write!(f, "{}: {}", field, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate crawler configuration
pub fn validate_crawl_request(request: &CrawlRequest) -> ValidationResult<()> {
    // Validate base URL
    if request.base_url.is_empty() {
        return Err(ValidationError::with_field(
            "Base URL is required",
            "base_url",
        ));
    }

    // Validate URL format
    if Url::parse(&request.base_url).is_err() {
        return Err(ValidationError::with_field(
            "Invalid URL format",
            "base_url",
        ));
    }

    // Validate numeric fields
    if request.max_total_urls == 0 {
        return Err(ValidationError::with_field(
            "Max total URLs must be greater than 0",
            "max_total_urls",
        ));
    }

    if request.max_crawl_depth == 0 {
        return Err(ValidationError::with_field(
            "Max crawl depth must be greater than 0",
            "max_crawl_depth",
        ));
    }

    // Validate URL extensions format
    for ext in &request.avoid_url_extensions {
        if ext.is_empty() {
            return Err(ValidationError::with_field(
                "URL extensions cannot be empty",
                "avoid_url_extensions",
            ));
        }
    }

    // Validate match strategy
    if !["any", "all"].contains(&request.match_strategy.as_str()) {
        return Err(ValidationError::with_field(
            "Match strategy must be 'any' or 'all'",
            "match_strategy",
        ));
    }

    Ok(())
}
