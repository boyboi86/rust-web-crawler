// Centralized logging configuration and utilities

pub mod events;
pub mod formatter;

// Re-export logging components
pub use events::{
    CrawlEvent, CrawlEventLogger, ErrorEvent, ErrorType, PerformanceEvent, PerformanceEventType,
};
pub use formatter::{CrawlLogFormatter, JsonLogFormatter};
