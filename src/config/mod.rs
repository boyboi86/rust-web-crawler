// Configuration management module

pub mod crawler;
pub mod environment;
pub mod presets;

// Re-export common configuration types
pub use crawler::{HttpClientFactory, LatinWordFilter, LoggingConfig, WebCrawlerConfig, defaults};
pub use environment::EnvironmentConfig;
pub use presets::*;
