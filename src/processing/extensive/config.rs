/// Configuration for extensive crawling
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

use crate::core::error::CrawlError;

/// Crawl depth configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrawlDepth {
    /// Unlimited depth
    Unlimited,
    /// Maximum depth limit
    Limited(usize),
    /// Fixed depth only
    Exact(usize),
}

impl Default for CrawlDepth {
    fn default() -> Self {
        CrawlDepth::Limited(3)
    }
}

/// Domain scope for extensive crawling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DomainScope {
    /// Stay within the same domain
    SameDomain,
    /// Allow subdomains of the original domain
    Subdomains,
    /// Allow specific domains
    AllowList(HashSet<String>),
    /// Block specific domains
    BlockList(HashSet<String>),
    /// No domain restrictions
    Unrestricted,
}

impl Default for DomainScope {
    fn default() -> Self {
        DomainScope::SameDomain
    }
}

/// Link filtering rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkFilter {
    /// File extensions to include
    pub include_extensions: Option<HashSet<String>>,
    /// File extensions to exclude
    pub exclude_extensions: Option<HashSet<String>>,
    /// URL patterns to include (regex)
    pub include_patterns: Option<Vec<String>>,
    /// URL patterns to exclude (regex)
    pub exclude_patterns: Option<Vec<String>>,
    /// Minimum URL length
    pub min_url_length: Option<usize>,
    /// Maximum URL length
    pub max_url_length: Option<usize>,
    /// Exclude query parameters
    pub exclude_query_params: bool,
    /// Exclude fragment identifiers
    pub exclude_fragments: bool,
}

impl Default for LinkFilter {
    fn default() -> Self {
        let mut exclude_extensions = HashSet::new();
        exclude_extensions.extend(
            [
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "rar", "tar", "gz",
                "7z", "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "mp3", "mp4", "avi",
                "mov", "wmv", "flv", "exe", "msi", "dmg", "pkg",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        Self {
            include_extensions: None,
            exclude_extensions: Some(exclude_extensions),
            include_patterns: None,
            exclude_patterns: None,
            min_url_length: Some(10),
            max_url_length: Some(2048),
            exclude_query_params: false,
            exclude_fragments: true,
        }
    }
}

/// Priority scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityConfig {
    /// Base priority for all links
    pub base_priority: u8,
    /// Priority adjustments by link category
    pub category_adjustments: CategoryPriorityAdjustments,
    /// Priority boost for links with anchor text
    pub anchor_text_boost: u8,
    /// Priority adjustments by URL depth
    pub depth_adjustments: DepthPriorityAdjustments,
    /// Priority boost for priority pattern matches
    pub pattern_boost: u8,
    /// Priority penalty for query parameters
    pub query_penalty: u8,
    /// Priority penalty for fragments
    pub fragment_penalty: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryPriorityAdjustments {
    pub internal_boost: u8,
    pub external_boost: u8,
    pub document_boost: u8,
    pub resource_penalty: u8,
    pub media_penalty: u8,
    pub other_boost: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthPriorityAdjustments {
    /// Boost for root-level pages (depth 1)
    pub root_boost: u8,
    /// Boost for shallow pages (depth 2-3)
    pub shallow_boost: u8,
    /// Penalty for deep pages (depth > 3)
    pub deep_penalty: u8,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            base_priority: 50,
            category_adjustments: CategoryPriorityAdjustments {
                internal_boost: 20,
                external_boost: 10,
                document_boost: 15,
                resource_penalty: 30,
                media_penalty: 20,
                other_boost: 5,
            },
            anchor_text_boost: 10,
            depth_adjustments: DepthPriorityAdjustments {
                root_boost: 15,
                shallow_boost: 10,
                deep_penalty: 10,
            },
            pattern_boost: 30,
            query_penalty: 5,
            fragment_penalty: 3,
        }
    }
}

/// Priority threshold configuration for task categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityThresholds {
    /// Threshold for low priority (0 to low_threshold)
    pub low_threshold: u8,
    /// Threshold for normal priority (low_threshold+1 to normal_threshold)
    pub normal_threshold: u8,
    /// Threshold for high priority (normal_threshold+1 to high_threshold)
    pub high_threshold: u8,
}

impl Default for PriorityThresholds {
    fn default() -> Self {
        Self {
            low_threshold: 50,
            normal_threshold: 100,
            high_threshold: 150,
        }
    }
}

/// Configuration for extensive crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensiveConfig {
    /// Enable extensive crawling
    pub enabled: bool,
    /// Maximum crawl depth
    pub max_depth: CrawlDepth,
    /// Domain scope restrictions
    pub domain_scope: DomainScope,
    /// Link filtering rules
    pub link_filter: LinkFilter,
    /// Priority scoring configuration
    pub priority_config: PriorityConfig,
    /// Priority threshold configuration
    pub priority_thresholds: PriorityThresholds,
    /// Maximum number of URLs to queue
    pub max_queue_size: Option<usize>,
    /// Maximum URLs per page to process
    pub max_links_per_page: Option<usize>,
    /// Delay between adding URLs to queue (milliseconds)
    pub queue_delay_ms: Option<u64>,
    /// Priority boost for certain URL patterns
    pub priority_patterns: Option<Vec<String>>,
    /// Respect robots.txt for discovered URLs
    pub respect_robots: bool,
}

impl Default for ExtensiveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_depth: CrawlDepth::default(),
            domain_scope: DomainScope::default(),
            link_filter: LinkFilter::default(),
            priority_config: PriorityConfig::default(),
            priority_thresholds: PriorityThresholds::default(),
            max_queue_size: Some(10000),
            max_links_per_page: Some(100),
            queue_delay_ms: Some(100),
            priority_patterns: None,
            respect_robots: true,
        }
    }
}

impl ExtensiveConfig {
    /// Create a new extensive crawling configuration
    pub fn new(max_depth: CrawlDepth, domain_scope: DomainScope) -> Self {
        Self {
            enabled: true,
            max_depth,
            domain_scope,
            ..Default::default()
        }
    }

    /// Create configuration for same-domain crawling
    pub fn same_domain(max_depth: usize) -> Self {
        Self::new(CrawlDepth::Limited(max_depth), DomainScope::SameDomain)
    }

    /// Create configuration for subdomain crawling
    pub fn with_subdomains(max_depth: usize) -> Self {
        Self::new(CrawlDepth::Limited(max_depth), DomainScope::Subdomains)
    }

    /// Create unrestricted configuration
    pub fn unrestricted(max_depth: usize) -> Self {
        Self::new(CrawlDepth::Limited(max_depth), DomainScope::Unrestricted)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CrawlError> {
        if !self.enabled {
            return Ok(());
        }

        // Validate max_queue_size
        if let Some(max_size) = self.max_queue_size {
            if max_size == 0 {
                return Err(CrawlError::ExtensiveConfigError(
                    "Maximum queue size must be greater than 0".to_string(),
                ));
            }
        }

        // Validate max_links_per_page
        if let Some(max_links) = self.max_links_per_page {
            if max_links == 0 {
                return Err(CrawlError::ExtensiveConfigError(
                    "Maximum links per page must be greater than 0".to_string(),
                ));
            }
        }

        // Validate URL length constraints
        if let (Some(min_len), Some(max_len)) = (
            self.link_filter.min_url_length,
            self.link_filter.max_url_length,
        ) {
            if min_len > max_len {
                return Err(CrawlError::ExtensiveConfigError(
                    "Minimum URL length cannot be greater than maximum URL length".to_string(),
                ));
            }
        }

        // Validate regex patterns
        if let Some(ref patterns) = self.link_filter.include_patterns {
            for pattern in patterns {
                if let Err(e) = regex::Regex::new(pattern) {
                    return Err(CrawlError::ExtensiveConfigError(format!(
                        "Invalid include pattern '{}': {}",
                        pattern, e
                    )));
                }
            }
        }

        if let Some(ref patterns) = self.link_filter.exclude_patterns {
            for pattern in patterns {
                if let Err(e) = regex::Regex::new(pattern) {
                    return Err(CrawlError::ExtensiveConfigError(format!(
                        "Invalid exclude pattern '{}': {}",
                        pattern, e
                    )));
                }
            }
        }

        if let Some(ref patterns) = self.priority_patterns {
            for pattern in patterns {
                if let Err(e) = regex::Regex::new(pattern) {
                    return Err(CrawlError::ExtensiveConfigError(format!(
                        "Invalid priority pattern '{}': {}",
                        pattern, e
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if extensive crawling should be performed
    pub fn should_crawl_extensively(&self) -> bool {
        self.enabled
    }

    /// Check if a URL should be included based on domain scope
    pub fn is_domain_allowed(&self, url: &Url, base_url: &Url) -> bool {
        match &self.domain_scope {
            DomainScope::SameDomain => url.domain() == base_url.domain(),
            DomainScope::Subdomains => {
                if let (Some(url_domain), Some(base_domain)) = (url.domain(), base_url.domain()) {
                    url_domain == base_domain || url_domain.ends_with(&format!(".{}", base_domain))
                } else {
                    false
                }
            }
            DomainScope::AllowList(allowed) => url
                .domain()
                .map_or(false, |domain| allowed.contains(domain)),
            DomainScope::BlockList(blocked) => url
                .domain()
                .map_or(true, |domain| !blocked.contains(domain)),
            DomainScope::Unrestricted => true,
        }
    }

    /// Check if depth limit allows further crawling
    pub fn is_depth_allowed(&self, current_depth: usize) -> bool {
        match self.max_depth {
            CrawlDepth::Unlimited => true,
            CrawlDepth::Limited(max) => current_depth < max,
            CrawlDepth::Exact(exact) => current_depth == exact,
        }
    }

    /// Get normalized URL (apply fragment and query parameter filters)
    pub fn normalize_url(&self, url: &mut Url) {
        if self.link_filter.exclude_fragments {
            url.set_fragment(None);
        }

        if self.link_filter.exclude_query_params {
            url.set_query(None);
        }
    }
}
