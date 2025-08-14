/// Link discovery and URL validation
/// 
/// This module provides comprehensive link extraction, discovery, and URL validation
/// functionality for web crawling operations.

use crate::core::ErrorUtils;
use crate::config::WebCrawlerConfig;
use anyhow::Result;
use std::collections::HashSet;
use url::Url;

/// Link extraction and discovery functionality
pub struct LinkExtractor {
    base_url: Url,
    allowed_domains: HashSet<String>,
    max_depth: usize,
    _respect_robots_txt: bool, // Prefixed with _ to indicate intentionally unused for now
}

#[derive(Debug, Clone)]
pub struct ExtractedLink {
    pub url: Url,
    pub anchor_text: String,
    pub link_type: LinkType,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Internal,  // Same domain
    External,  // Different domain
    Subdomain, // Subdomain of allowed domain
    Asset,     // CSS, JS, images
    Document,  // PDF, DOC, etc.
}

#[derive(Debug, Default)]
pub struct LinkStats {
    pub total: usize,
    pub internal: usize,
    pub external: usize,
    pub subdomain: usize,
    pub assets: usize,
    pub documents: usize,
}

impl LinkExtractor {
    pub fn new(base_url: Url, allowed_domains: Vec<String>, max_depth: usize) -> Self {
        let mut domains = HashSet::new();

        // Add base domain
        if let Some(host) = base_url.host_str() {
            domains.insert(host.to_string());
        }

        // Add allowed domains
        for domain in allowed_domains {
            domains.insert(domain);
        }

        Self {
            base_url,
            allowed_domains: domains,
            max_depth,
            _respect_robots_txt: true,
        }
    }

    /// Extract all links from HTML content
    pub async fn extract_links(
        &self,
        html: &str,
        current_url: &Url,
        current_depth: usize,
    ) -> Result<Vec<ExtractedLink>> {
        if current_depth >= self.max_depth {
            return Ok(vec![]);
        }

        // Use simple regex-based extraction instead of HTML rewriter to avoid borrowing issues
        self.extract_links_regex(html, current_url, current_depth)
            .await
    }

    /// Extract links using regex patterns
    async fn extract_links_regex(
        &self,
        html: &str,
        current_url: &Url,
        current_depth: usize,
    ) -> Result<Vec<ExtractedLink>> {
        let mut links = Vec::new();

        // Extract href attributes from anchor tags
        let href_regex =
            regex::Regex::new(r#"<a[^>]+href\s*=\s*["']([^"']+)["'][^>]*>([^<]*)</a>"#)?;
        for capture in href_regex.captures_iter(html) {
            if let Some(href) = capture.get(1) {
                let anchor_text = capture.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
                if let Ok(resolved_url) = current_url.join(href.as_str())
                    && let Some(extracted_link) =
                        self.process_link(resolved_url, anchor_text, current_depth + 1)
                {
                    links.push(extracted_link);
                }
            }
        }

        // Extract src attributes from img tags
        let img_regex = regex::Regex::new(r#"<img[^>]+src\s*=\s*["']([^"']+)["'][^>]*>"#)?;
        for capture in img_regex.captures_iter(html) {
            if let Some(src) = capture.get(1)
                && let Ok(resolved_url) = current_url.join(src.as_str())
                && let Some(extracted_link) =
                    self.process_link(resolved_url, "image".to_string(), current_depth + 1)
            {
                links.push(extracted_link);
            }
        }

        // Extract href attributes from link tags
        let link_regex = regex::Regex::new(r#"<link[^>]+href\s*=\s*["']([^"']+)["'][^>]*>"#)?;
        for capture in link_regex.captures_iter(html) {
            if let Some(href) = capture.get(1)
                && let Ok(resolved_url) = current_url.join(href.as_str())
                && let Some(extracted_link) =
                    self.process_link(resolved_url, "stylesheet".to_string(), current_depth + 1)
            {
                links.push(extracted_link);
            }
        }

        Ok(links)
    }

    /// Process a single link and determine its type
    fn process_link(&self, url: Url, anchor_text: String, depth: usize) -> Option<ExtractedLink> {
        // Skip invalid URLs
        if !ErrorUtils::is_valid_crawl_url(url.as_str()) {
            return None;
        }

        let link_type = self.classify_link(&url);

        // Filter based on link type and policy
        match link_type {
            LinkType::External => {
                // Only include external links if explicitly allowed
                if !self.should_include_external() {
                    return None;
                }
            }
            LinkType::Asset => {
                // Usually skip assets unless specifically needed
                if !self.should_include_assets() {
                    return None;
                }
            }
            _ => {}
        }

        Some(ExtractedLink {
            url,
            anchor_text,
            link_type,
            depth,
        })
    }

    /// Classify a link based on its URL
    fn classify_link(&self, url: &Url) -> LinkType {
        let host = url.host_str().unwrap_or("");
        let path = url.path().to_lowercase();

        // Check if it's an asset
        if is_asset_url(&path) {
            return LinkType::Asset;
        }

        // Check if it's a document
        if is_document_url(&path) {
            return LinkType::Document;
        }

        // Check domain relationship
        if let Some(base_host) = self.base_url.host_str() {
            if host == base_host {
                return LinkType::Internal;
            }

            // Check if it's a subdomain
            if host.ends_with(&format!(".{}", base_host)) {
                return LinkType::Subdomain;
            }

            // Check if it's in allowed domains
            if self.allowed_domains.contains(host) {
                return LinkType::Internal;
            }
        }

        LinkType::External
    }

    /// Check if external links should be included
    fn should_include_external(&self) -> bool {
        false // Default: don't follow external links
    }

    /// Check if asset links should be included
    fn should_include_assets(&self) -> bool {
        false // Default: don't download assets
    }

    /// Filter and deduplicate links
    #[allow(dead_code)] // Will be used in future features
    fn filter_and_deduplicate(&self, links: Vec<ExtractedLink>) -> Result<Vec<ExtractedLink>> {
        let mut seen_urls = HashSet::new();
        let mut filtered_links = Vec::new();

        for link in links {
            let normalized_url = normalize_url(&link.url);

            if seen_urls.insert(normalized_url) {
                filtered_links.push(link);
            }
        }

        // Sort by priority (internal links first, then by depth)
        filtered_links.sort_by(|a, b| {
            // Internal links have higher priority
            let priority_a = match a.link_type {
                LinkType::Internal => 0,
                LinkType::Subdomain => 1,
                LinkType::External => 2,
                LinkType::Asset => 3,
                LinkType::Document => 4,
            };

            let priority_b = match b.link_type {
                LinkType::Internal => 0,
                LinkType::Subdomain => 1,
                LinkType::External => 2,
                LinkType::Asset => 3,
                LinkType::Document => 4,
            };

            priority_a
                .cmp(&priority_b)
                .then_with(|| a.depth.cmp(&b.depth))
        });

        Ok(filtered_links)
    }

    /// Get link discovery statistics
    pub fn get_stats(&self, links: &[ExtractedLink]) -> LinkStats {
        let mut stats = LinkStats::default();

        for link in links {
            stats.total += 1;
            match link.link_type {
                LinkType::Internal => stats.internal += 1,
                LinkType::External => stats.external += 1,
                LinkType::Subdomain => stats.subdomain += 1,
                LinkType::Asset => stats.assets += 1,
                LinkType::Document => stats.documents += 1,
            }
        }

        stats
    }
}

// URL Validation Functions

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

/// Check if URL is an asset (CSS, JS, images)
pub fn is_asset_url(path: &str) -> bool {
    let asset_extensions = [
        ".css", ".js", ".jpg", ".jpeg", ".png", ".gif", ".svg", ".ico", ".woff", ".woff2",
        ".ttf", ".eot",
    ];

    asset_extensions.iter().any(|ext| path.ends_with(ext))
}

/// Check if URL is a document
pub fn is_document_url(path: &str) -> bool {
    let doc_extensions = [
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".zip", ".rar", ".tar",
        ".gz",
    ];

    doc_extensions.iter().any(|ext| path.ends_with(ext))
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
