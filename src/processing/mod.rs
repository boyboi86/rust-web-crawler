/// Content processing and analysis
///
/// This module consolidates all content-related processing including:
/// - HTML parsing and content extraction
/// - Language detection and analysis
/// - Link discovery and URL validation
///
/// Building blocks are assembled here for unified content processing capabilities.
pub mod content;
pub mod discovery;
pub mod language;

// Re-export all processing components
pub use content::{ContentExtractor, extract_links_from_html, extract_title_from_html};
pub use discovery::{
    ExtractedLink, LinkExtractor, LinkStats, LinkType, is_asset_url, is_document_url,
    is_same_domain, is_valid_crawl_url, normalize_url,
};
pub use language::{
    ContentDifficulty, analyze_language_stats, detect_language, detect_language_type,
    estimate_content_difficulty, estimate_reading_time, get_language_confidence,
};
