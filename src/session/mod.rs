/// Session management module
///
/// This module provides high-level session orchestration for crawl operations,
/// abstracting away the complexity of managing crawlers, queues, and results.
pub mod manager;
pub mod manager_refactored;
pub mod statistics;

// Re-export main functionality
pub use manager::{CrawlResultData, CrawlSession, CrawlSessionConfig, SessionResult};
pub use manager_refactored::{
    CrawlSession as CrawlSessionRefactored, SessionConfig, SessionConfigBuilder, SessionMetrics,
    SessionState, SessionSummary,
};
pub use statistics::{RealTimeStats, SessionStatistics};
