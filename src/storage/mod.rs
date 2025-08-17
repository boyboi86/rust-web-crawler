// Data persistence and analytics

pub mod data;
pub mod metrics;

// Refactored modules with building blocks compliance
pub mod data_refactored;
pub mod metrics_refactored;

// Re-export original storage components
pub use data::{
    CrawlAnalytics, CrawlMetadata, CrawlSessionSummary, DataStorage as OriginalDataStorage,
    OutputFormat, StoredCrawlResult as OriginalStoredCrawlResult,
};
pub use metrics::{
    CrawlerMetrics as OriginalCrawlerMetrics, MetricsSnapshot as OriginalMetricsSnapshot,
};

// Re-export refactored storage components for building blocks compliance
pub use data_refactored::{
    CompressionType, DataStorage, StorageConfig, StorageConfigBuilder, StorageFormat,
    StorageStatistics, StoredCrawlResult,
};
pub use metrics_refactored::{
    BasicMetricsStats, CrawlerMetrics, DomainMetrics, ErrorStatistics, MetricsConfig,
    MetricsConfigBuilder, MetricsSnapshot, QueueMetrics, QueueMetricsSnapshot,
};
