/// Session management module
///
/// This module provides high-level session orchestration for crawl operations,
/// abstracting away the complexity of managing crawlers, queues, and results.
pub mod manager;
pub mod statistics;

// Re-export main functionality
pub use manager::{CrawlResultData, CrawlSession, CrawlSessionConfig, SessionResult};
pub use statistics::{RealTimeStats, SessionStatistics};
