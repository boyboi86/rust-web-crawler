// Core types, traits, and error handling

pub mod error;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export common items for convenience
pub use error::CrawlError;
pub use traits::{
    ContentProcessor, DnsResolver, ErrorHandler, HttpClientManager, RateLimiter, RobotsChecker,
};
pub use types::{
    CrawlResult, CrawlTask, DomainRateLimit, ErrorSeverity, LangType, QueueStats, RetryConfig,
    SkipReason, TaskPriority, TaskResult, TaskStatus,
};
pub use utils::ErrorUtils;
