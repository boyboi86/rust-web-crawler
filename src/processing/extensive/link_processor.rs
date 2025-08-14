/// Link processing for extensive crawling
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

use super::config::ExtensiveConfig;
use crate::core::error::CrawlError;
use crate::processing::discovery::ExtractedLink;

/// Category of discovered link
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkCategory {
    /// Internal page link
    Internal,
    /// External page link
    External,
    /// Resource link (images, CSS, JS)
    Resource,
    /// Document link (PDF, DOC, etc.)
    Document,
    /// Media link (video, audio)
    Media,
    /// Other/Unknown
    Other,
}

/// Processed link with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedLink {
    /// Original extracted link
    pub extracted_link: ExtractedLink,
    /// Normalized URL
    pub normalized_url: String,
    /// Link category
    pub category: LinkCategory,
    /// Crawl depth for this link
    pub depth: usize,
    /// Priority score (higher = more important)
    pub priority: u8,
    /// Whether this link should be crawled
    pub should_crawl: bool,
    /// Reason for inclusion/exclusion
    pub reason: String,
}

/// Link processor for extensive crawling
pub struct LinkProcessor {
    config: ExtensiveConfig,
    include_patterns: Option<Vec<Regex>>,
    exclude_patterns: Option<Vec<Regex>>,
    priority_patterns: Option<Vec<Regex>>,
}

impl LinkProcessor {
    /// Create a new link processor
    pub fn new(config: ExtensiveConfig) -> Result<Self, CrawlError> {
        config.validate()?;

        let include_patterns = if let Some(ref patterns) = config.link_filter.include_patterns {
            let compiled: Result<Vec<Regex>, _> =
                patterns.iter().map(|pattern| Regex::new(pattern)).collect();
            Some(compiled.map_err(|e| {
                CrawlError::ExtensiveConfigError(format!(
                    "Failed to compile include pattern: {}",
                    e
                ))
            })?)
        } else {
            None
        };

        let exclude_patterns = if let Some(ref patterns) = config.link_filter.exclude_patterns {
            let compiled: Result<Vec<Regex>, _> =
                patterns.iter().map(|pattern| Regex::new(pattern)).collect();
            Some(compiled.map_err(|e| {
                CrawlError::ExtensiveConfigError(format!(
                    "Failed to compile exclude pattern: {}",
                    e
                ))
            })?)
        } else {
            None
        };

        let priority_patterns = if let Some(ref patterns) = config.priority_patterns {
            let compiled: Result<Vec<Regex>, _> =
                patterns.iter().map(|pattern| Regex::new(pattern)).collect();
            Some(compiled.map_err(|e| {
                CrawlError::ExtensiveConfigError(format!(
                    "Failed to compile priority pattern: {}",
                    e
                ))
            })?)
        } else {
            None
        };

        Ok(Self {
            config,
            include_patterns,
            exclude_patterns,
            priority_patterns,
        })
    }

    /// Process discovered links and determine which should be crawled
    pub fn process_links(
        &self,
        extracted_links: Vec<ExtractedLink>,
        base_url: &Url,
        current_depth: usize,
    ) -> Result<Vec<ProcessedLink>, CrawlError> {
        if !self.config.should_crawl_extensively() {
            return Ok(Vec::new());
        }

        let mut processed_links = Vec::new();
        let mut link_count = 0;

        for extracted_link in extracted_links {
            // Check max links per page limit
            if let Some(max_links) = self.config.max_links_per_page {
                if link_count >= max_links {
                    break;
                }
            }

            if let Ok(processed) = self.process_single_link(extracted_link, base_url, current_depth)
            {
                processed_links.push(processed);
                if processed_links.last().unwrap().should_crawl {
                    link_count += 1;
                }
            }
        }

        // Sort by priority (highest first)
        processed_links.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(processed_links)
    }

    /// Process a single extracted link
    fn process_single_link(
        &self,
        extracted_link: ExtractedLink,
        base_url: &Url,
        current_depth: usize,
    ) -> Result<ProcessedLink, CrawlError> {
        let url = extracted_link.url.clone(); // ExtractedLink.url is already a Url

        let mut normalized_url = url.clone();
        self.config.normalize_url(&mut normalized_url);

        let category = self.categorize_link(&url);
        let next_depth = current_depth + 1;

        let (should_crawl, reason) = self.should_crawl_link(&url, base_url, next_depth, &category);
        let priority = self.calculate_priority(&url, &category, &extracted_link);

        Ok(ProcessedLink {
            extracted_link,
            normalized_url: normalized_url.to_string(),
            category,
            depth: next_depth,
            priority,
            should_crawl,
            reason,
        })
    }

    /// Determine if a link should be crawled
    fn should_crawl_link(
        &self,
        url: &Url,
        base_url: &Url,
        depth: usize,
        category: &LinkCategory,
    ) -> (bool, String) {
        // Check depth limit
        if !self.config.is_depth_allowed(depth) {
            return (false, format!("Depth limit exceeded: {}", depth));
        }

        // Check domain scope
        if !self.config.is_domain_allowed(url, base_url) {
            return (false, "Domain not allowed".to_string());
        }

        // Only crawl page links, not resources
        if matches!(category, LinkCategory::Resource | LinkCategory::Media) {
            return (false, "Resource/media link excluded".to_string());
        }

        // Check URL length
        if let Some(min_len) = self.config.link_filter.min_url_length {
            if url.as_str().len() < min_len {
                return (
                    false,
                    format!("URL too short: {} < {}", url.as_str().len(), min_len),
                );
            }
        }

        if let Some(max_len) = self.config.link_filter.max_url_length {
            if url.as_str().len() > max_len {
                return (
                    false,
                    format!("URL too long: {} > {}", url.as_str().len(), max_len),
                );
            }
        }

        // Check file extension
        if let Some(path) = url.path_segments().and_then(|segments| segments.last()) {
            if let Some(extension) = path.split('.').last() {
                let ext = extension.to_lowercase();

                // Check include extensions
                if let Some(ref include_exts) = self.config.link_filter.include_extensions {
                    if !include_exts.contains(&ext) {
                        return (false, format!("Extension '{}' not in include list", ext));
                    }
                }

                // Check exclude extensions
                if let Some(ref exclude_exts) = self.config.link_filter.exclude_extensions {
                    if exclude_exts.contains(&ext) {
                        return (false, format!("Extension '{}' in exclude list", ext));
                    }
                }
            }
        }

        // Check include patterns
        if let Some(ref patterns) = self.include_patterns {
            let url_str = url.as_str();
            if !patterns.iter().any(|pattern| pattern.is_match(url_str)) {
                return (false, "URL doesn't match include patterns".to_string());
            }
        }

        // Check exclude patterns
        if let Some(ref patterns) = self.exclude_patterns {
            let url_str = url.as_str();
            if patterns.iter().any(|pattern| pattern.is_match(url_str)) {
                return (false, "URL matches exclude patterns".to_string());
            }
        }

        (true, "Passed all filters".to_string())
    }

    /// Categorize a link based on its URL
    fn categorize_link(&self, url: &Url) -> LinkCategory {
        if let Some(path) = url.path_segments().and_then(|segments| segments.last()) {
            if let Some(extension) = path.split('.').last() {
                let ext = extension.to_lowercase();

                // Document extensions
                if [
                    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "rtf",
                ]
                .contains(&ext.as_str())
                {
                    return LinkCategory::Document;
                }

                // Media extensions
                if ["mp3", "mp4", "avi", "mov", "wmv", "flv", "wav", "ogg"].contains(&ext.as_str())
                {
                    return LinkCategory::Media;
                }

                // Resource extensions
                if [
                    "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "css", "js", "ico",
                ]
                .contains(&ext.as_str())
                {
                    return LinkCategory::Resource;
                }
            }
        }

        // Determine if internal or external
        if let (Some(url_host), Some(current_host)) = (url.host_str(), url.host_str()) {
            if url_host == current_host {
                LinkCategory::Internal
            } else {
                LinkCategory::External
            }
        } else {
            LinkCategory::Other
        }
    }

    /// Calculate priority score for a link
    fn calculate_priority(
        &self,
        url: &Url,
        category: &LinkCategory,
        extracted_link: &ExtractedLink,
    ) -> u8 {
        let mut priority = self.config.priority_config.base_priority;

        // Category-based priority using configurable values
        let adjustments = &self.config.priority_config.category_adjustments;
        match category {
            LinkCategory::Internal => {
                priority = priority.saturating_add(adjustments.internal_boost)
            }
            LinkCategory::External => {
                priority = priority.saturating_add(adjustments.external_boost)
            }
            LinkCategory::Document => {
                priority = priority.saturating_add(adjustments.document_boost)
            }
            LinkCategory::Resource => {
                priority = priority.saturating_sub(adjustments.resource_penalty)
            }
            LinkCategory::Media => priority = priority.saturating_sub(adjustments.media_penalty),
            LinkCategory::Other => priority = priority.saturating_add(adjustments.other_boost),
        }

        // Link text quality using configurable boost
        if !extracted_link.anchor_text.trim().is_empty() {
            priority = priority.saturating_add(self.config.priority_config.anchor_text_boost);
        }

        // URL structure quality using configurable adjustments
        let path_segments = url.path_segments().map(|s| s.count()).unwrap_or(0);
        let depth_adj = &self.config.priority_config.depth_adjustments;
        if path_segments == 1 {
            priority = priority.saturating_add(depth_adj.root_boost);
        } else if path_segments <= 3 {
            priority = priority.saturating_add(depth_adj.shallow_boost);
        } else if path_segments > 5 {
            priority = priority.saturating_sub(depth_adj.deep_penalty);
        }

        // Check priority patterns using configurable boost
        if let Some(ref patterns) = self.priority_patterns {
            let url_str = url.as_str();
            for pattern in patterns {
                if pattern.is_match(url_str) {
                    priority = priority.saturating_add(self.config.priority_config.pattern_boost);
                    break;
                }
            }
        }

        // Penalize query parameters and fragments using configurable penalties
        if url.query().is_some() {
            priority = priority.saturating_sub(self.config.priority_config.query_penalty);
        }
        if url.fragment().is_some() {
            priority = priority.saturating_sub(self.config.priority_config.fragment_penalty);
        }

        priority
    }

    /// Filter links to remove duplicates and apply additional constraints
    pub fn filter_processed_links(&self, links: Vec<ProcessedLink>) -> Vec<ProcessedLink> {
        let mut seen_urls = HashSet::new();
        let mut filtered_links = Vec::new();

        for link in links {
            if !link.should_crawl {
                continue;
            }

            // Remove duplicates
            if seen_urls.contains(&link.normalized_url) {
                continue;
            }

            seen_urls.insert(link.normalized_url.clone());
            filtered_links.push(link);

            // Check max queue size
            if let Some(max_size) = self.config.max_queue_size {
                if filtered_links.len() >= max_size {
                    break;
                }
            }
        }

        filtered_links
    }
}
