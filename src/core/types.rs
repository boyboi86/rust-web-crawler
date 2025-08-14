use serde::{Deserialize, Serialize};
use std::time::Instant;
use url::Url;
use whatlang::Lang;

/// Type aliases for optional types - building blocks for common patterns
pub type OptionInstant = Option<Instant>;
pub type OptionString = Option<String>;
pub type OptionLangType = Option<LangType>;
pub type OptionUrl = Option<Url>;

/// Building block for task timing information
#[derive(Debug, Clone)]
pub struct TaskTiming {
    #[allow(dead_code)] // Will be used in CrawlTask refactor
    pub created_at: Instant,
    pub started_at: OptionInstant,
    pub last_attempt_at: OptionInstant,
    pub delay_until: OptionInstant,
}

impl TaskTiming {
    pub fn new() -> Self {
        Self {
            created_at: Instant::now(),
            started_at: None,
            last_attempt_at: None,
            delay_until: None,
        }
    }

    pub fn mark_started(&mut self) {
        let now = Instant::now(); // Scoped variable for DRY
        self.started_at = Some(now);
        self.last_attempt_at = Some(now);
    }

    pub fn mark_attempt(&mut self) {
        let now = Instant::now(); // Scoped variable for DRY
        self.last_attempt_at = Some(now);
    }

    pub fn set_retry_delay(&mut self, delay: std::time::Duration) {
        let now = Instant::now(); // Scoped variable for DRY
        self.delay_until = Some(now + delay);
    }

    pub fn is_ready_for_retry(&self) -> bool {
        self.delay_until
            .map_or(true, |delay_until| Instant::now() >= delay_until)
    }
}

impl Default for TaskTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// URL serialization helper
pub mod url_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use url::Url;

    pub fn serialize<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        url.as_str().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Url::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Enhanced language type with additional utility methods
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum LangType {
    Eng,
    Cmn,
    Fra,
    Deu,
    Jpn,
    Kor,
}

impl LangType {
    /// Convert LangType to HTTP Accept-Language code
    pub fn to_http_code(&self) -> &'static str {
        match self {
            LangType::Eng => "en",
            LangType::Cmn => "zh-CN",
            LangType::Fra => "fr",
            LangType::Deu => "de",
            LangType::Jpn => "ja",
            LangType::Kor => "ko",
        }
    }

    /// Get regional variants for better content negotiation
    pub fn to_http_variants(&self) -> Vec<&'static str> {
        match self {
            LangType::Eng => vec!["en-US", "en-GB", "en"],
            LangType::Cmn => vec!["zh-CN", "zh-TW", "zh"],
            LangType::Fra => vec!["fr-FR", "fr-CA", "fr"],
            LangType::Deu => vec!["de-DE", "de-AT", "de"],
            LangType::Jpn => vec!["ja-JP", "ja"],
            LangType::Kor => vec!["ko-KR", "ko"],
        }
    }

    /// Convert from whatlang::Lang to LangType
    pub fn from_detected_lang(lang: Lang) -> Option<Self> {
        match lang {
            Lang::Eng => Some(LangType::Eng),
            Lang::Cmn => Some(LangType::Cmn),
            Lang::Fra => Some(LangType::Fra),
            Lang::Deu => Some(LangType::Deu),
            Lang::Jpn => Some(LangType::Jpn),
            Lang::Kor => Some(LangType::Kor),
            _ => None, // Unsupported language
        }
    }

    /// Convert from whatlang detection result with accepted language validation
    pub fn from_detection_with_validation(
        detection_result: Option<whatlang::Info>,
        accepted_languages: &[LangType],
    ) -> Option<Self> {
        if let Some(detected) = detection_result
            && let Some(lang_type) = Self::from_detected_lang(detected.lang())
            && accepted_languages.contains(&lang_type)
        {
            return Some(lang_type);
        }
        None
    }

    /// Check if this language uses CJK (Chinese, Japanese, Korean) word segmentation
    pub fn is_cjk(&self) -> bool {
        matches!(self, LangType::Cmn | LangType::Jpn | LangType::Kor)
    }
}

/// Building block for task result content
#[derive(Debug, Clone)]
pub struct TaskContent {
    pub content: String,
    pub word_count: usize,
    pub detected_language: OptionLangType,
}

/// Building block for task result error information  
#[derive(Debug, Clone)]
pub struct TaskError {
    pub error: crate::core::error::CrawlError,
    pub error_message: String,
    pub retry_count: u32,
}

/// Building block for task execution timing
#[derive(Debug, Clone)]
pub struct TaskExecutionTiming {
    pub started_at: OptionInstant,
    pub completed_at: OptionInstant,
    pub processing_time: std::time::Duration,
}

/// Building block for task counts in queue statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskCounts {
    pub total: u64,
    pub pending: u64,
    pub in_progress: u64,
    pub completed: u64,
    pub failed: u64,
    pub retrying: u64,
    pub dead: u64,
}

/// Building block for performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_processing_time_ms: f64,
    pub success_rate: f64,
}

/// Building block for timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

/// Building block for rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateConfig {
    pub max_requests_per_second: u32,
    pub window_size_ms: u64,
}

/// Enhanced crawl result with more detailed information
#[derive(Debug, Clone)]
pub enum CrawlResult {
    Success {
        content: String,
        word_count: usize,
        detected_language: Option<LangType>,
        processing_time_ms: u64,
    },
    Failed {
        error: crate::core::error::CrawlError,
        retry_count: u32,
        final_attempt: bool,
    },
    Skipped {
        reason: SkipReason,
        url: Url,
    },
}

/// Reasons why a URL might be skipped
#[derive(Debug, Clone)]
pub enum SkipReason {
    AlreadyVisited,
    RobotsBlocked,
    ContentFiltered,
    LanguageNotAccepted,
    NoContent,
    ExtensionBlocked(String),
    DomainBlocked(String),
}

/// Error severity levels for better error handling
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Domain-specific rate limiting configuration - composed of building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRateLimit {
    #[serde(flatten)]
    pub rate: RateConfig,
}

impl Default for DomainRateLimit {
    fn default() -> Self {
        Self {
            rate: RateConfig {
                max_requests_per_second: 2, // Conservative default
                window_size_ms: 1000,
            },
        }
    }
}

/// Retry configuration for fault-tolerant crawling - composed of building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    #[serde(flatten)]
    pub timing: TimingConfig,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timing: TimingConfig {
                base_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff_multiplier: 2.0,
            },
            jitter_factor: 0.1,
        }
    }
}

/// Task priority levels for the message queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Status of a crawl task in the queue
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Retrying,
    Dead, // Exceeded max retries
}

/// Building block for task identity and core properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIdentity {
    pub id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
}

/// Building block for task execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub attempt_count: u32,
    pub max_retries: u32,
    pub depth: usize,
    pub error_message: Option<String>,
}

/// A crawl task in the message queue - composed of building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlTask {
    pub id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub attempt_count: u32,
    pub max_retries: u32,
    pub retry_count: u32, // Added for backward compatibility
    pub depth: usize,
    pub error_message: OptionString,
    pub user_agent: String, // Added for backward compatibility

    // Building blocks for composition - timing is handled by TaskTiming
    #[serde(skip)]
    pub timing: TaskTiming,
}

// Remove the Deref implementation and building block sub-structs for now
// We'll implement this gradually to avoid breaking changes

impl CrawlTask {
    pub fn new(url: Url, priority: TaskPriority, max_retries: u32) -> Self {
        let task_id = format!("task_{}", uuid::Uuid::new_v4().simple()); // Scoped variable
        Self {
            id: task_id,
            url,
            priority,
            status: TaskStatus::Pending,
            attempt_count: 0,
            max_retries,
            retry_count: 0, // Initialize to 0
            depth: 0,       // Default depth
            error_message: None,
            user_agent: "rust-web-crawler/1.0".to_string(), // Default user agent
            timing: TaskTiming::new(),                      // Use building block
        }
    }

    /// Create a new task with specified depth for extension crawling
    pub fn new_with_depth(
        url: Url,
        priority: TaskPriority,
        max_retries: u32,
        depth: usize,
    ) -> Self {
        let task_id = format!("task_{}", uuid::Uuid::new_v4().simple()); // Scoped variable
        Self {
            id: task_id,
            url,
            priority,
            status: TaskStatus::Pending,
            attempt_count: 0,
            max_retries,
            retry_count: 0, // Initialize to 0
            depth,
            error_message: None,
            user_agent: "rust-web-crawler/1.0".to_string(), // Default user agent
            timing: TaskTiming::new(),                      // Use building block
        }
    }

    pub fn can_retry(&self) -> bool {
        self.attempt_count < self.max_retries && self.status != TaskStatus::Dead
    }

    pub fn is_ready_for_retry(&self) -> bool {
        self.timing.is_ready_for_retry()
    }

    pub fn mark_failed(&mut self, error: String, retry_delay: Option<std::time::Duration>) {
        self.attempt_count += 1;
        self.timing.mark_attempt();
        self.error_message = Some(error);

        if self.can_retry() {
            self.status = TaskStatus::Retrying;
            if let Some(delay) = retry_delay {
                self.timing.set_retry_delay(delay);
            }
        } else {
            self.status = TaskStatus::Dead;
        }
    }

    pub fn mark_in_progress(&mut self) {
        self.status = TaskStatus::InProgress;
        self.timing.mark_started();
    }

    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
    }

    // Convenience accessor methods for compatibility
    pub fn created_at(&self) -> Instant {
        self.timing.created_at
    }

    pub fn started_at(&self) -> OptionInstant {
        self.timing.started_at
    }

    pub fn last_attempt_at(&self) -> OptionInstant {
        self.timing.last_attempt_at
    }

    pub fn delay_until(&self) -> OptionInstant {
        self.timing.delay_until
    }
}

/// Result of task processing - composed of building blocks
#[derive(Debug)]
pub struct TaskResult {
    pub task_id: String,
    pub url: Url,
    pub success: bool,
    pub content: Option<TaskContent>,
    pub error: OptionString,
    pub processing_time: std::time::Duration,
}

/// Message queue statistics - composed of building blocks
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    #[serde(flatten)]
    pub counts: TaskCounts,
    #[serde(flatten)]
    pub performance: PerformanceMetrics,
}
