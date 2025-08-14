// Core types, traits, and error handling

pub mod error;
pub mod traits;
pub mod types;
pub mod utils;

// Re-export common items for convenience
pub use error::CrawlError;
pub use traits::{
    Categorizable, ContentProcessor, DnsResolver, ErrorHandler, HttpClientManager, Normalizable,
    RateLimiter, Retryable, RobotsChecker, TimestampedTask, Validatable,
};
pub use types::{
    CrawlResult, CrawlTask, DomainRateLimit, ErrorSeverity, LangType, OptionInstant, QueueStats,
    RetryConfig, SkipReason, TaskPriority, TaskResult, TaskStatus, TaskTiming,
};
pub use utils::ErrorUtils;
