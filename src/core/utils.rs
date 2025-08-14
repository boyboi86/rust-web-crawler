use anyhow::Error;
use std::time::Duration;

/// Common error handling utilities
pub struct ErrorUtils;

impl ErrorUtils {
    /// Check if an error should trigger a retry
    pub fn is_retryable_error(error: &Error) -> bool {
        let error_string = error.to_string().to_lowercase();

        // Network-related errors that should be retried
        error_string.contains("timeout")
            || error_string.contains("connection")
            || error_string.contains("network")
            || error_string.contains("dns")
            || error_string.contains("temporary failure")
            || error_string.contains("503")
            || error_string.contains("502")
            || error_string.contains("429") // Rate limited
    }

    /// Get retry delay with exponential backoff
    pub fn calculate_retry_delay(
        attempt: u32,
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    ) -> Duration {
        let delay_ms = (base_delay.as_millis() as f64 * multiplier.powi(attempt as i32)) as u64;
        Duration::from_millis(delay_ms.min(max_delay.as_millis() as u64))
    }

    /// URL validation utilities
    pub fn is_valid_crawl_url(url: &str) -> bool {
        if let Ok(parsed) = url::Url::parse(url) {
            // Check scheme
            if !matches!(parsed.scheme(), "http" | "https") {
                return false;
            }

            // Check for suspicious patterns
            let path = parsed.path().to_lowercase();
            if path.ends_with(".pdf")
                || path.ends_with(".zip")
                || path.ends_with(".exe")
                || path.ends_with(".dmg")
            {
                return false;
            }

            // Check for infinite redirect patterns
            if path.contains("redirect") && path.len() > 200 {
                return false;
            }

            true
        } else {
            false
        }
    }

    /// Normalize URL for deduplication
    pub fn normalize_url(url: &str) -> String {
        if let Ok(mut parsed) = url::Url::parse(url) {
            // Remove fragment
            parsed.set_fragment(None);

            // Sort query parameters for consistent hashing
            if let Some(query) = parsed.query() {
                let mut params: Vec<_> = query.split('&').collect();
                params.sort();
                parsed.set_query(Some(&params.join("&")));
            }

            // Remove trailing slash from path
            if parsed.path().ends_with('/') && parsed.path().len() > 1 {
                let trimmed = parsed.path().trim_end_matches('/');
                let mut new_url = parsed.clone();
                new_url.set_path(trimmed);
                return new_url.to_string();
            }

            parsed.to_string()
        } else {
            url.to_string()
        }
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    /// Convert HTTP status code to descriptive error
    pub fn http_status_to_error(status: u16) -> String {
        match status {
            400 => "Bad Request".to_string(),
            401 => "Unauthorized".to_string(),
            403 => "Forbidden".to_string(),
            404 => "Not Found".to_string(),
            429 => "Too Many Requests".to_string(),
            500 => "Internal Server Error".to_string(),
            502 => "Bad Gateway".to_string(),
            503 => "Service Unavailable".to_string(),
            504 => "Gateway Timeout".to_string(),
            _ => format!("HTTP {}", status),
        }
    }
}
