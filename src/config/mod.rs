// Configuration management module

pub mod crawler;
pub mod environment;

// Re-export common configuration types
pub use crawler::{HttpClientFactory, LoggingConfig, WebCrawlerConfig, defaults};
pub use environment::EnvironmentConfig;
