use crate::core::ErrorUtils;
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
        if self.is_asset_url(&path) {
            return LinkType::Asset;
        }

        // Check if it's a document
        if self.is_document_url(&path) {
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

    /// Check if URL is an asset (CSS, JS, images)
    fn is_asset_url(&self, path: &str) -> bool {
        let asset_extensions = [
            ".css", ".js", ".jpg", ".jpeg", ".png", ".gif", ".svg", ".ico", ".woff", ".woff2",
            ".ttf", ".eot",
        ];

        asset_extensions.iter().any(|ext| path.ends_with(ext))
    }

    /// Check if URL is a document
    fn is_document_url(&self, path: &str) -> bool {
        let doc_extensions = [
            ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".zip", ".rar", ".tar",
            ".gz",
        ];

        doc_extensions.iter().any(|ext| path.ends_with(ext))
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
            let normalized_url = ErrorUtils::normalize_url(link.url.as_str());

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

#[derive(Debug, Default)]
pub struct LinkStats {
    pub total: usize,
    pub internal: usize,
    pub external: usize,
    pub subdomain: usize,
    pub assets: usize,
    pub documents: usize,
}
