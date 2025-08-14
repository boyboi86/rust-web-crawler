use serde::{Deserialize, Serialize};
use std::time::Instant;
use url::Url;
use whatlang::Lang;

/// URL serialization helper
mod url_serde {
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
            _ => None,
        }
    }

    /// Convert from whatlang::Lang to LangType with validation
    pub fn from_detected_lang_safe(lang: Lang) -> Option<Self> {
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
            && let Some(lang_type) = Self::from_detected_lang_safe(detected.lang())
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

/// Domain-specific rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRateLimit {
    pub max_requests_per_second: u32,
    pub window_size_ms: u64, // Default: 1000ms (1 second)
}

impl Default for DomainRateLimit {
    fn default() -> Self {
        Self {
            max_requests_per_second: 2, // Conservative default
            window_size_ms: 1000,
        }
    }
}

/// Retry configuration for fault-tolerant crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Task priority levels for the message queue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
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

/// A crawl task in the message queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlTask {
    pub id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub attempt_count: u32,
    pub max_retries: u32,
    #[serde(skip, default = "Instant::now")]
    pub created_at: Instant,
    #[serde(skip)]
    pub started_at: Option<Instant>, // When task started processing
    #[serde(skip)]
    pub last_attempt_at: Option<Instant>,
    #[serde(skip)]
    pub delay_until: Option<Instant>, // For retry delays
    pub error_message: Option<String>,
}

impl CrawlTask {
    pub fn new(url: Url, priority: TaskPriority, max_retries: u32) -> Self {
        let id = format!("task_{}", uuid::Uuid::new_v4().simple());
        Self {
            id,
            url,
            priority,
            status: TaskStatus::Pending,
            attempt_count: 0,
            max_retries,
            created_at: Instant::now(),
            started_at: None,
            last_attempt_at: None,
            delay_until: None,
            error_message: None,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.attempt_count < self.max_retries && self.status != TaskStatus::Dead
    }

    pub fn is_ready_for_retry(&self) -> bool {
        if let Some(delay_until) = self.delay_until {
            Instant::now() >= delay_until
        } else {
            true
        }
    }

    pub fn mark_failed(&mut self, error: String, retry_delay: Option<std::time::Duration>) {
        self.attempt_count += 1;
        self.last_attempt_at = Some(Instant::now());
        self.error_message = Some(error);

        if self.can_retry() {
            self.status = TaskStatus::Retrying;
            if let Some(delay) = retry_delay {
                self.delay_until = Some(Instant::now() + delay);
            }
        } else {
            self.status = TaskStatus::Dead;
        }
    }

    pub fn mark_in_progress(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Instant::now());
        self.last_attempt_at = Some(Instant::now());
    }

    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
    }
}

/// Result of task processing
#[derive(Debug)]
pub struct TaskResult {
    pub task_id: String,
    pub url: Url,
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
    pub processing_time: std::time::Duration,
}

/// Message queue statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub total_tasks: u64,
    pub pending_tasks: u64,
    pub in_progress_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub retrying_tasks: u64,
    pub dead_tasks: u64,
    pub average_processing_time_ms: f64,
    pub success_rate: f64,
}
