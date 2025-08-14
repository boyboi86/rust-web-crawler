/// Content processing and analysis
///
/// This module consolidates all content-related processing including:
/// - HTML parsing and content extraction
/// - Language detection and analysis  
/// - Link discovery and URL validation
///
/// Enhanced with Level 3 features integrated as extensions:
/// - content: Enhanced with keyword-based content filtering (Feature 1)
/// - discovery: Enhanced with extensive crawling and auto-queue expansion (Feature 2)  
/// - language: Enhanced with advanced text cleaning and preprocessing (Feature 3)
///
/// Building blocks are assembled here for unified content processing capabilities.
// Core processing modules (each enhanced with Level 3 features)
pub mod content; // Basic content + keyword filtering (Feature 1)
pub mod discovery; // Basic discovery + extensive crawling (Feature 2)
pub mod language; // Basic language + text cleaning (Feature 3)

// Level 3 feature modules (internal organization only)
mod cleaning; // Feature 3: Text cleaning
mod extensive; // Feature 2: Extensive crawling
mod keyword; // Feature 1: Keyword-based filtering

// Re-export main processing components (unified interface)
pub use content::{
    ContentExtractor,
    // Enhanced Feature 1: Keyword-based content filtering
    KeywordConfig,
    KeywordExtractor,
    KeywordMatchInfo,
    KeywordMatcher,
    KeywordMode,
    KeywordOptions,
    MatchResult,
    MatchStats,
    extract_links_from_html,
    extract_title_from_html,
};
pub use discovery::{
    CategoryPriorityAdjustments,
    // Enhanced Feature 2: Extensive crawling with auto-queue expansion
    CrawlDepth,
    DepthPriorityAdjustments,
    DiscoveryStats,
    DomainScope,
    ExtensiveConfig,
    ExtensiveQueueManager,
    ExtractedLink,
    LinkCategory,
    LinkExtractor,
    LinkFilter,
    LinkProcessor,
    LinkStats,
    LinkType,
    PriorityConfig,
    PriorityThresholds,
    ProcessedLink,
    QueueStatus,
    is_asset_url,
    is_document_url,
    is_same_domain,
    is_valid_crawl_url,
    normalize_url,
};
pub use language::{
    // Enhanced Feature 3: Advanced text cleaning and preprocessing
    CharacterFilter,
    CleaningConfig,
    CleaningEngine,
    CleaningResult,
    CleaningRule,
    CleaningStats,
    ContentDifficulty,
    LanguageFilter,
    LengthFilter,
    RuleType,
    TextCleaner,
    WordFilter,
    analyze_language_stats,
    detect_language,
    detect_language_type,
    estimate_content_difficulty,
    estimate_reading_time,
    get_language_confidence,
};
