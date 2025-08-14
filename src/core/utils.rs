use crate::core::error::CrawlError;
use anyhow::Error;
use std::time::Duration;

/// Common error handling utilities
pub struct ErrorUtils;

impl ErrorUtils {
    /// Convert an anyhow error to CrawlError and check if retryable
    pub fn categorize_and_check_retry(error: &Error) -> (CrawlError, bool) {
        let crawl_error = CrawlError::from_anyhow_error(error);
        let is_retryable = crawl_error.is_retryable();
        (crawl_error, is_retryable)
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

    /// URL validation utilities - functional approach
    pub fn is_valid_crawl_url(url: &str) -> bool {
        const BLOCKED_EXTENSIONS: &[&str] = &[".pdf", ".zip", ".exe", ".dmg"];
        const VALID_SCHEMES: &[&str] = &["http", "https"];

        url::Url::parse(url)
            .map(|parsed| {
                let scheme_valid = VALID_SCHEMES.contains(&parsed.scheme());
                let path = parsed.path().to_lowercase();

                let extension_valid = !BLOCKED_EXTENSIONS.iter().any(|ext| path.ends_with(ext));

                let redirect_safe = !(path.contains("redirect") && path.len() > 200);

                scheme_valid && extension_valid && redirect_safe
            })
            .unwrap_or(false)
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

    /// Convert HTTP status code to descriptive error - functional approach
    pub fn http_status_to_error(status: u16) -> String {
        const HTTP_STATUS_MAP: &[(u16, &str)] = &[
            (400, "Bad Request"),
            (401, "Unauthorized"),
            (403, "Forbidden"),
            (404, "Not Found"),
            (429, "Too Many Requests"),
            (500, "Internal Server Error"),
            (502, "Bad Gateway"),
            (503, "Service Unavailable"),
            (504, "Gateway Timeout"),
        ];

        HTTP_STATUS_MAP
            .iter()
            .find_map(|(code, msg)| (*code == status).then_some(*msg))
            .map(String::from)
            .unwrap_or_else(|| format!("HTTP {}", status))
    }

    /// Detect language from content using whatlang
    /// This is a common utility that can be used across modules
    pub fn detect_language_simple(content: &str) -> Option<String> {
        use whatlang::{Lang, detect};

        if let Some(info) = detect(content) {
            match info.lang() {
                Lang::Eng => Some("en".to_string()),
                Lang::Fra => Some("fr".to_string()),
                Lang::Deu => Some("de".to_string()),
                Lang::Cmn => Some("zh".to_string()),
                Lang::Jpn => Some("ja".to_string()),
                Lang::Kor => Some("ko".to_string()),
                _ => Some(format!("{:?}", info.lang()).to_lowercase()),
            }
        } else {
            None
        }
    }

    /// Detect language and convert to LangType
    pub fn detect_language_typed(content: &str) -> Option<crate::core::types::LangType> {
        use crate::core::types::LangType;
        use whatlang::detect;

        if let Some(info) = detect(content) {
            LangType::from_detected_lang(info.lang())
        } else {
            None
        }
    }
}
