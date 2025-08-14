/// Modern feature-based organization for Rust web crawler
///
/// This architecture organizes modules by features and functionality rather than
/// technical implementation details, following the 4 design principles:
/// 1. Modules by features/logic with sub-modules for sub-features
/// 2. Building blocks assembled into unified modules
/// 3. One level down organization (feature/sub-feature)
/// 4. Logical cohesion over technical separation
pub mod config; // Configuration management
pub mod core; // Core types and utilities
pub mod crawler; // Main crawling engine
pub mod logging; // Unified logging system
pub mod network; // Network layer components
pub mod processing; // Content processing and analysis
pub mod queue; // Task queue management
pub mod session; // Session management and orchestration
pub mod storage; // Data persistence

// Re-exports for convenience
// Core types and utilities
pub use core::{
    ContentProcessor, CrawlError, CrawlResult, CrawlTask, DnsResolver, DomainRateLimit,
    ErrorHandler, ErrorSeverity, ErrorUtils, HttpClientManager, LangType, QueueStats, RateLimiter,
    RetryConfig, RobotsChecker, SkipReason, TaskPriority, TaskResult, TaskStatus,
};

// Configuration
pub use config::{EnvironmentConfig, HttpClientFactory, LatinWordFilter, WebCrawlerConfig};

// Network components
pub use network::{
    ClientManager, DnsCache, DomainRequestTracker, GlobalRateLimiter, RobotsCache, RobotsHandler,
};

// Processing components - unified feature-based exports (with Level 3 enhancements)
pub use processing::{
    CategoryPriorityAdjustments,
    CharacterFilter,
    CleaningConfig,
    CleaningEngine,
    CleaningResult,
    CleaningRule,
    CleaningStats,
    // Language detection and analysis (Enhanced with Feature 3: Text cleaning)
    ContentDifficulty,
    // Content extraction and HTML processing (Enhanced with Feature 1: Keyword filtering)
    ContentExtractor,
    CrawlDepth,
    DepthPriorityAdjustments,

    DiscoveryStats,
    DomainScope,
    ExtensiveConfig,
    ExtensiveQueueManager,
    // Link discovery and URL validation (Enhanced with Feature 2: Extensive crawling)
    ExtractedLink,
    KeywordConfig,
    KeywordExtractor,
    KeywordMatchInfo,
    KeywordMatcher,
    KeywordMode,
    KeywordOptions,
    LanguageFilter,
    LengthFilter,
    LinkCategory,
    LinkExtractor,
    LinkFilter,
    LinkProcessor,
    LinkStats,
    LinkType,
    MatchResult,
    MatchStats,

    PriorityConfig,
    PriorityThresholds,
    ProcessedLink,
    QueueStatus,
    RuleType,
    TextCleaner,
    WordFilter,
    analyze_language_stats,
    detect_language,
    detect_language_type,
    estimate_content_difficulty,
    estimate_reading_time,
    extract_links_from_html,
    extract_title_from_html,
    get_language_confidence,
    is_asset_url,
    is_document_url,
    is_same_domain,
    is_valid_crawl_url,
    normalize_url,
};

// Session management - core functionality
pub use session::{CrawlResultData, CrawlSession, CrawlSessionConfig, SessionResult};

// Logging - unified system
pub use logging::{
    CrawlEvent,
    // Advanced event logging
    CrawlEventLogger,
    // Formatting
    CrawlLogFormatter,
    ErrorEvent,
    JsonLogFormatter,
    PerformanceEvent,
    configure_logging_for_environment,
    init_json_logging,
    // Basic logging setup
    init_logging,
    init_logging_with_level,
};

// Storage components
pub use storage::{CrawlMetadata, DataStorage, OutputFormat, StoredCrawlResult};

// Queue management
pub use queue::TaskQueue;

// Crawler components
pub use crawler::WebCrawler;

/// Library metadata and version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Feature-based re-exports for different use cases

/// Essential crawling functionality
pub mod essential {
    pub use crate::{ContentExtractor, CrawlSession, CrawlSessionConfig, WebCrawler};
}

/// Content processing functionality
pub mod content {
    pub use crate::processing::*;
}

/// Network management functionality  
pub mod network_management {
    pub use crate::network::*;
}

/// Logging and monitoring functionality
pub mod monitoring {
    pub use crate::logging::*;
}
