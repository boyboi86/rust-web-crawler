// Main crawler logic and engine

pub mod engine;
pub mod engine_refactored;

// Re-export crawler components
pub use engine::WebCrawler;
pub use engine_refactored::{
    CrawlerEngine, CrawlerEngineConfig, CrawlerEngineConfigBuilder, CrawlerStatistics,
};
