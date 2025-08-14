/// Session management module
///
/// This module provides high-level session orchestration for crawl operations,
/// abstracting away the complexity of managing crawlers, queues, and results.
pub mod manager;
pub mod result_collector;
pub mod statistics;

pub use manager::{CrawlSession, CrawlSessionConfig, SessionResult};
pub use result_collector::ResultCollector;
pub use statistics::{RealTimeStats, SessionStatistics};
