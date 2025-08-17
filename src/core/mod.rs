// Core types, traits, and error handling

pub mod error;
pub mod traits;
pub mod types;
pub mod utils;

// Refactored modules with building blocks compliance
pub mod error_refactored;
pub mod types_refactored;

// Re-export original items for compatibility
pub use error::CrawlError as OriginalCrawlError;
pub use traits::{
    Categorizable, ContentProcessor, DnsResolver, ErrorHandler, HttpClientManager, Normalizable,
    RateLimiter, Retryable, RobotsChecker, TimestampedTask, Validatable,
};
pub use types::{
    CrawlResult as OriginalCrawlResult, CrawlTask as OriginalCrawlTask, DomainRateLimit,
    ErrorSeverity as OriginalErrorSeverity, LangType as OriginalLangType, OptionInstant,
    QueueStats, RetryConfig, SkipReason as OriginalSkipReason,
    TaskPriority as OriginalTaskPriority, TaskResult as OriginalTaskResult,
    TaskStatus as OriginalTaskStatus, TaskTiming,
};
pub use utils::ErrorUtils;

// Re-export refactored items for building blocks compliance
pub use error_refactored::{
    CrawlError, ErrorConfig, ErrorConfigBuilder, ErrorContext,
    ErrorHandler as RefactoredErrorHandler, ErrorSeverity,
};
pub use types_refactored::{
    CrawlTask, LangType, QueueStatistics, Region, SkipReason, TaskConfig, TaskConfigBuilder,
    TaskContent, TaskExecutionTiming, TaskPriority, TaskResult, TaskStatus,
};
