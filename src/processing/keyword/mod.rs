/// Keyword-based content filtering
///
/// This module provides functionality to crawl and filter content based on target keywords.
/// When enabled, only content containing the specified keywords will be returned.
pub mod config;
pub mod extractor;
pub mod matcher;

// Re-export all keyword processing components
pub use config::{KeywordConfig, KeywordMode, KeywordOptions};
pub use extractor::{KeywordExtractor, KeywordMatchInfo};
pub use matcher::{KeywordMatcher, MatchResult, MatchStats};
