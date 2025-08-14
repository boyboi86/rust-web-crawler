/// Extensive crawling functionality
///
/// This module provides automatic queue expansion by discovering and adding new URLs
/// from crawled pages. When enabled, discovered links are automatically added to the
/// crawl queue for future processing.
pub mod config;
pub mod link_processor;
pub mod queue_manager;

// Re-export all extensive crawling components
pub use config::{
    CategoryPriorityAdjustments, CrawlDepth, DepthPriorityAdjustments, DomainScope,
    ExtensiveConfig, LinkFilter, PriorityConfig, PriorityThresholds,
};
pub use link_processor::{LinkCategory, LinkProcessor, ProcessedLink};
pub use queue_manager::{DiscoveryStats, ExtensiveQueueManager, QueueStatus};
