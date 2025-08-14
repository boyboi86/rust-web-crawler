use crate::config::WebCrawlerConfig;
/// URL validation utilities
use url::Url;

/// Check if a URL is valid for crawling based on configuration
pub fn is_valid_crawl_url(url: &Url, config: &WebCrawlerConfig) -> bool {
    // Check if URL scheme is supported
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }

    // Check if domain is blocked
    if let Some(domain) = url.domain() {
        // Check against common blocked domains
        let blocked_domains = [
            "facebook.com",
            "twitter.com",
            "instagram.com",
            "linkedin.com",
            "youtube.com",
            "tiktok.com",
            "pinterest.com",
        ];
        if blocked_domains
            .iter()
            .any(|&blocked| domain.contains(blocked))
        {
            return false;
        }
    }

    // Check file extensions that should be avoided
    if let Some(path) = url.path_segments() {
        if let Some(last_segment) = path.last() {
            // Check against configured avoid_url_extensions
            for extension in &config.avoid_url_extensions {
                if last_segment.ends_with(extension) {
                    return false;
                }
            }

            // Check against common binary file extensions
            let blocked_extensions = [
                ".pdf", ".jpg", ".png", ".gif", ".mp4", ".zip", ".exe", ".rar", ".tar", ".gz",
                ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".mp3", ".wav", ".avi", ".mov",
                ".flv",
            ];
            if blocked_extensions
                .iter()
                .any(|&ext| last_segment.ends_with(ext))
            {
                return false;
            }
        }
    }

    // Check URL length (very long URLs might be problematic)
    if url.as_str().len() > 2048 {
        return false;
    }

    // Check for suspicious URL patterns
    let url_str = url.as_str().to_lowercase();
    let suspicious_patterns = [
        "javascript:",
        "data:",
        "mailto:",
        "ftp:",
        "file:",
        "admin",
        "login",
        "signin",
        "signup",
        "register",
        "download",
        "upload",
        "api/",
        "ajax",
        "json",
    ];

    if suspicious_patterns
        .iter()
        .any(|&pattern| url_str.contains(pattern))
    {
        return false;
    }

    true
}

/// Check if URL is suitable for deep crawling (following links)
pub fn is_suitable_for_deep_crawl(url: &Url) -> bool {
    // Only crawl deeper into pages, not files
    let path = url.path();

    // Avoid crawling into obvious non-content areas
    let avoid_paths = [
        "/api/",
        "/ajax/",
        "/json/",
        "/xml/",
        "/rss/",
        "/admin/",
        "/wp-admin/",
        "/login/",
        "/auth/",
        "/static/",
        "/assets/",
        "/css/",
        "/js/",
        "/images/",
    ];

    for avoid_path in &avoid_paths {
        if path.contains(avoid_path) {
            return false;
        }
    }

    // Prefer pages that look like content
    let content_indicators = [
        "/news/",
        "/article/",
        "/post/",
        "/blog/",
        "/content/",
        "/page/",
        "/story/",
        "/report/",
        "/review/",
        "/guide/",
    ];

    for indicator in &content_indicators {
        if path.contains(indicator) {
            return true;
        }
    }

    // Default to allowing if no specific indicators
    true
}

/// Normalize URL for comparison and deduplication
pub fn normalize_url(url: &Url) -> String {
    let mut normalized = url.clone();

    // Remove fragment
    normalized.set_fragment(None);

    // Remove common tracking parameters
    let tracking_params = [
        "utm_source",
        "utm_medium",
        "utm_campaign",
        "utm_content",
        "utm_term",
        "fbclid",
        "gclid",
        "ref",
        "source",
        "campaign",
    ];

    let query_pairs: Vec<(String, String)> = normalized
        .query_pairs()
        .filter(|(key, _)| !tracking_params.contains(&key.as_ref()))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // Rebuild query string without tracking parameters
    if query_pairs.is_empty() {
        normalized.set_query(None);
    } else {
        let query = query_pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        normalized.set_query(Some(&query));
    }

    // Ensure trailing slash consistency
    let mut path = normalized.path().to_string();
    if path.ends_with('/') && path.len() > 1 {
        path = path.trim_end_matches('/').to_string();
        normalized.set_path(&path);
    }

    normalized.to_string().to_lowercase()
}

/// Extract domain from URL
pub fn extract_domain(url: &Url) -> Option<String> {
    url.domain().map(|d| d.to_string())
}

/// Check if URL belongs to same domain
pub fn is_same_domain(url1: &Url, url2: &Url) -> bool {
    url1.domain() == url2.domain()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WebCrawlerConfig;

    #[test]
    fn test_is_valid_crawl_url() {
        let config = WebCrawlerConfig::default();

        let valid_url = Url::parse("https://example.com/article").unwrap();
        assert!(is_valid_crawl_url(&valid_url, &config));

        let invalid_url = Url::parse("https://example.com/file.pdf").unwrap();
        assert!(!is_valid_crawl_url(&invalid_url, &config));
    }

    #[test]
    fn test_normalize_url() {
        let url =
            Url::parse("https://example.com/page?utm_source=test&important=value#section").unwrap();
        let normalized = normalize_url(&url);
        assert!(!normalized.contains("utm_source"));
        assert!(normalized.contains("important=value"));
        assert!(!normalized.contains("#section"));
    }

    #[test]
    fn test_is_same_domain() {
        let url1 = Url::parse("https://example.com/page1").unwrap();
        let url2 = Url::parse("https://example.com/page2").unwrap();
        let url3 = Url::parse("https://other.com/page").unwrap();

        assert!(is_same_domain(&url1, &url2));
        assert!(!is_same_domain(&url1, &url3));
    }
}
