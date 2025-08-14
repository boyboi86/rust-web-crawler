// Data persistence and analytics

pub mod data;
pub mod metrics;

// Re-export storage components
pub use data::{
    CrawlAnalytics, CrawlMetadata, CrawlSessionSummary, DataStorage, OutputFormat,
    StoredCrawlResult,
};
pub use metrics::{CrawlerMetrics, MetricsSnapshot};
