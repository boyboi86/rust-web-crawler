// Modern modular organization for Rust web crawler
pub mod config;
pub mod core;
pub mod crawler;
pub mod logging;
pub mod network;
pub mod processing;
pub mod queue;
pub mod session;
pub mod storage;
pub mod utils;

// Re-exports for convenience
// Core types and utilities
pub use core::{
    ContentProcessor, CrawlError, CrawlResult, CrawlTask, DnsResolver, DomainRateLimit,
    ErrorHandler, ErrorSeverity, ErrorUtils, HttpClientManager, LangType, QueueStats, RateLimiter,
    RetryConfig, RobotsChecker, SkipReason, TaskPriority, TaskResult, TaskStatus,
};

// Configuration
pub use config::{EnvironmentConfig, HttpClientFactory, LatinWordFilter, WebCrawlerConfig};

// Network components
pub use network::{
    ClientManager, DnsCache, DomainRequestTracker, GlobalRateLimiter, RobotsCache, RobotsHandler,
};

// Processing components
pub use processing::{ContentExtractor, LinkExtractor};

// Queue management
pub use queue::{TaskQueue, TtlCache};

// Session management
pub use session::{CrawlSession, CrawlSessionConfig, SessionResult};

// Storage and analytics
pub use storage::{
    CrawlAnalytics, CrawlMetadata, CrawlSessionSummary, CrawlerMetrics, DataStorage,
    MetricsSnapshot, OutputFormat, StoredCrawlResult,
};

// Logging components
pub use logging::{CrawlEvent, CrawlEventLogger, ErrorEvent, PerformanceEvent};

// Configuration (including logging config and presets)
pub use config::LoggingConfig;

// Utilities
pub use utils::{
    detect_language, extract_links_from_html, extract_title_from_html, init_logging,
    is_valid_crawl_url,
};

// Main crawler
pub use crawler::WebCrawler;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
