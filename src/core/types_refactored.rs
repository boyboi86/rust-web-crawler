/// Core types module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex type configurations
/// Following Rule 4: Privacy first - controlled access to type internals
/// Following Rule 6: Feature-based organization - types grouped by functionality
/// Following Rule 8: Idiomatic Rust - Result<T,E>, Option<T>, proper error handling
use crate::common::{
    primitives::{BooleanFlag, LimitValue, SessionId, TaskId, UrlString},
    results::{ProcessingResult, TaskError},
    statistics::PerformanceMetrics,
    timing::{ExecutionTiming, TaskTiming},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use url::Url;

/// Geographic regions for content localization and proxy selection
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Region {
    NorthAmerica,
    Europe,
    AsiaPacific,
    China,
    Global,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::NorthAmerica => "north_america",
            Region::Europe => "europe",
            Region::AsiaPacific => "asia_pacific",
            Region::China => "china",
            Region::Global => "global",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "north_america" | "northamerica" | "na" | "us" => Some(Region::NorthAmerica),
            "europe" | "eu" => Some(Region::Europe),
            "asia_pacific" | "asiapacific" | "ap" | "asia" | "apac" => Some(Region::AsiaPacific),
            "china" | "cn" => Some(Region::China),
            "global" | "international" => Some(Region::Global),
            _ => None,
        }
    }

    pub fn timezone_offset(&self) -> i32 {
        match self {
            Region::NorthAmerica => -8, // PST
            Region::Europe => 1,        // CET
            Region::AsiaPacific => 9,   // JST
            Region::China => 8,         // CST
            Region::Global => 0,        // UTC
        }
    }
}

impl Default for Region {
    fn default() -> Self {
        Region::Global
    }
}

/// Enhanced language type with building blocks integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LangType {
    English,
    ChineseSimplified,
    ChineseTraditional,
    French,
    German,
    Japanese,
    Korean,
    Spanish,
    Portuguese,
    Russian,
}

impl LangType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LangType::English => "en",
            LangType::ChineseSimplified => "zh-CN",
            LangType::ChineseTraditional => "zh-TW",
            LangType::French => "fr",
            LangType::German => "de",
            LangType::Japanese => "ja",
            LangType::Korean => "ko",
            LangType::Spanish => "es",
            LangType::Portuguese => "pt",
            LangType::Russian => "ru",
        }
    }

    pub fn to_http_code(&self) -> &'static str {
        self.as_str()
    }

    pub fn to_http_variants(&self) -> Vec<&'static str> {
        match self {
            LangType::English => vec!["en-US", "en-GB", "en-CA", "en"],
            LangType::ChineseSimplified => vec!["zh-CN", "zh"],
            LangType::ChineseTraditional => vec!["zh-TW", "zh-HK", "zh"],
            LangType::French => vec!["fr-FR", "fr-CA", "fr"],
            LangType::German => vec!["de-DE", "de-AT", "de-CH", "de"],
            LangType::Japanese => vec!["ja-JP", "ja"],
            LangType::Korean => vec!["ko-KR", "ko"],
            LangType::Spanish => vec!["es-ES", "es-MX", "es-AR", "es"],
            LangType::Portuguese => vec!["pt-BR", "pt-PT", "pt"],
            LangType::Russian => vec!["ru-RU", "ru"],
        }
    }

    pub fn is_cjk(&self) -> bool {
        matches!(
            self,
            LangType::ChineseSimplified
                | LangType::ChineseTraditional
                | LangType::Japanese
                | LangType::Korean
        )
    }

    pub fn is_rtl(&self) -> bool {
        // For future Arabic/Hebrew support
        false
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(LangType::English),
            "zh-cn" | "zh_cn" | "chinese_simplified" => Some(LangType::ChineseSimplified),
            "zh-tw" | "zh_tw" | "chinese_traditional" => Some(LangType::ChineseTraditional),
            "fr" | "french" => Some(LangType::French),
            "de" | "german" => Some(LangType::German),
            "ja" | "japanese" => Some(LangType::Japanese),
            "ko" | "korean" => Some(LangType::Korean),
            "es" | "spanish" => Some(LangType::Spanish),
            "pt" | "portuguese" => Some(LangType::Portuguese),
            "ru" | "russian" => Some(LangType::Russian),
            _ => None,
        }
    }
}

impl Default for LangType {
    fn default() -> Self {
        LangType::English
    }
}

/// Task priority levels for queue management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Normal => "normal",
            TaskPriority::Medium => "medium",
            TaskPriority::High => "high",
            TaskPriority::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(TaskPriority::Low),
            "normal" => Some(TaskPriority::Normal),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "critical" => Some(TaskPriority::Critical),
            _ => None,
        }
    }

    pub fn priority_value(&self) -> u8 {
        *self as u8
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Task status in processing lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Retrying,
    Dead,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Retrying => "retrying",
            TaskStatus::Dead => "dead",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(TaskStatus::Pending),
            "in_progress" | "inprogress" => Some(TaskStatus::InProgress),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "retrying" => Some(TaskStatus::Retrying),
            "dead" => Some(TaskStatus::Dead),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Dead)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::InProgress | TaskStatus::Retrying)
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// Reasons for skipping URL processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkipReason {
    AlreadyVisited,
    RobotsBlocked,
    ContentFiltered,
    LanguageNotAccepted,
    NoContent,
    ExtensionBlocked(String),
    DomainBlocked(String),
    PolicyRestriction(String),
    TechnicalError(String),
}

impl SkipReason {
    pub fn as_str(&self) -> &str {
        match self {
            SkipReason::AlreadyVisited => "already_visited",
            SkipReason::RobotsBlocked => "robots_blocked",
            SkipReason::ContentFiltered => "content_filtered",
            SkipReason::LanguageNotAccepted => "language_not_accepted",
            SkipReason::NoContent => "no_content",
            SkipReason::ExtensionBlocked(_) => "extension_blocked",
            SkipReason::DomainBlocked(_) => "domain_blocked",
            SkipReason::PolicyRestriction(_) => "policy_restriction",
            SkipReason::TechnicalError(_) => "technical_error",
        }
    }

    pub fn details(&self) -> Option<&str> {
        match self {
            SkipReason::ExtensionBlocked(ext) => Some(ext),
            SkipReason::DomainBlocked(domain) => Some(domain),
            SkipReason::PolicyRestriction(policy) => Some(policy),
            SkipReason::TechnicalError(error) => Some(error),
            _ => None,
        }
    }
}

/// Configuration for task processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    // Basic task settings
    default_priority: TaskPriority,
    max_retries: LimitValue,
    retry_base_delay_ms: LimitValue,
    retry_max_delay_ms: LimitValue,
    retry_backoff_multiplier: f64,

    // Processing limits
    max_processing_time_secs: LimitValue,
    max_content_length: LimitValue,
    enable_depth_limiting: BooleanFlag,
    max_depth: LimitValue,

    // Language and region settings
    accepted_languages: Vec<LangType>,
    preferred_region: Region,
    enable_auto_language_detection: BooleanFlag,

    // Session tracking
    session_id: SessionId,
    track_execution_timing: BooleanFlag,
}

impl TaskConfig {
    pub fn builder() -> TaskConfigBuilder {
        TaskConfigBuilder::new()
    }

    // Getters for all configuration fields
    pub fn default_priority(&self) -> TaskPriority {
        self.default_priority
    }
    pub fn max_retries(&self) -> u64 {
        self.max_retries.value()
    }
    pub fn retry_base_delay_ms(&self) -> u64 {
        self.retry_base_delay_ms.value()
    }
    pub fn retry_max_delay_ms(&self) -> u64 {
        self.retry_max_delay_ms.value()
    }
    pub fn retry_backoff_multiplier(&self) -> f64 {
        self.retry_backoff_multiplier
    }
    pub fn max_processing_time_secs(&self) -> u64 {
        self.max_processing_time_secs.value()
    }
    pub fn max_content_length(&self) -> u64 {
        self.max_content_length.value()
    }
    pub fn should_enable_depth_limiting(&self) -> bool {
        self.enable_depth_limiting.is_enabled()
    }
    pub fn max_depth(&self) -> u64 {
        self.max_depth.value()
    }
    pub fn accepted_languages(&self) -> &[LangType] {
        &self.accepted_languages
    }
    pub fn preferred_region(&self) -> Region {
        self.preferred_region
    }
    pub fn should_enable_auto_language_detection(&self) -> bool {
        self.enable_auto_language_detection.is_enabled()
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn should_track_execution_timing(&self) -> bool {
        self.track_execution_timing.is_enabled()
    }

    /// Calculate retry delay with exponential backoff
    pub fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.retry_base_delay_ms() as f64;
        let multiplier = self.retry_backoff_multiplier();
        let max_delay = self.retry_max_delay_ms() as f64;

        let delay = base_delay * multiplier.powi(attempt as i32);
        let delay = delay.min(max_delay);

        Duration::from_millis(delay as u64)
    }
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            default_priority: TaskPriority::Normal,
            max_retries: LimitValue::new(3),
            retry_base_delay_ms: LimitValue::new(1000),
            retry_max_delay_ms: LimitValue::new(30000),
            retry_backoff_multiplier: 2.0,
            max_processing_time_secs: LimitValue::new(60),
            max_content_length: LimitValue::new(10 * 1024 * 1024), // 10MB
            enable_depth_limiting: BooleanFlag::enabled(),
            max_depth: LimitValue::new(5),
            accepted_languages: vec![LangType::English],
            preferred_region: Region::Global,
            enable_auto_language_detection: BooleanFlag::enabled(),
            session_id: SessionId::new(format!("task_{}", uuid::Uuid::new_v4())),
            track_execution_timing: BooleanFlag::enabled(),
        }
    }
}

/// Builder for task configuration
#[derive(Debug)]
pub struct TaskConfigBuilder {
    default_priority: TaskPriority,
    max_retries: LimitValue,
    retry_base_delay_ms: LimitValue,
    retry_max_delay_ms: LimitValue,
    retry_backoff_multiplier: f64,
    max_processing_time_secs: LimitValue,
    max_content_length: LimitValue,
    enable_depth_limiting: BooleanFlag,
    max_depth: LimitValue,
    accepted_languages: Vec<LangType>,
    preferred_region: Region,
    enable_auto_language_detection: BooleanFlag,
    session_id: SessionId,
    track_execution_timing: BooleanFlag,
}

impl TaskConfigBuilder {
    pub fn new() -> Self {
        let default_config = TaskConfig::default();
        Self {
            default_priority: default_config.default_priority,
            max_retries: default_config.max_retries,
            retry_base_delay_ms: default_config.retry_base_delay_ms,
            retry_max_delay_ms: default_config.retry_max_delay_ms,
            retry_backoff_multiplier: default_config.retry_backoff_multiplier,
            max_processing_time_secs: default_config.max_processing_time_secs,
            max_content_length: default_config.max_content_length,
            enable_depth_limiting: default_config.enable_depth_limiting,
            max_depth: default_config.max_depth,
            accepted_languages: default_config.accepted_languages,
            preferred_region: default_config.preferred_region,
            enable_auto_language_detection: default_config.enable_auto_language_detection,
            session_id: default_config.session_id,
            track_execution_timing: default_config.track_execution_timing,
        }
    }

    pub fn with_priority_settings(
        mut self,
        default_priority: TaskPriority,
        max_retries: LimitValue,
    ) -> Self {
        self.default_priority = default_priority;
        self.max_retries = max_retries;
        self
    }

    pub fn with_retry_settings(
        mut self,
        base_delay: LimitValue,
        max_delay: LimitValue,
        backoff_multiplier: f64,
    ) -> Self {
        self.retry_base_delay_ms = base_delay;
        self.retry_max_delay_ms = max_delay;
        self.retry_backoff_multiplier = backoff_multiplier;
        self
    }

    pub fn with_processing_limits(mut self, max_time: LimitValue, max_content: LimitValue) -> Self {
        self.max_processing_time_secs = max_time;
        self.max_content_length = max_content;
        self
    }

    pub fn with_depth_limiting(mut self, enabled: BooleanFlag, max_depth: LimitValue) -> Self {
        self.enable_depth_limiting = enabled;
        self.max_depth = max_depth;
        self
    }

    pub fn with_language_settings(
        mut self,
        accepted_languages: Vec<LangType>,
        auto_detection: BooleanFlag,
    ) -> Self {
        self.accepted_languages = accepted_languages;
        self.enable_auto_language_detection = auto_detection;
        self
    }

    pub fn with_region(mut self, region: Region) -> Self {
        self.preferred_region = region;
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_timing_tracking(mut self, enabled: BooleanFlag) -> Self {
        self.track_execution_timing = enabled;
        self
    }

    pub fn build(self) -> TaskConfig {
        TaskConfig {
            default_priority: self.default_priority,
            max_retries: self.max_retries,
            retry_base_delay_ms: self.retry_base_delay_ms,
            retry_max_delay_ms: self.retry_max_delay_ms,
            retry_backoff_multiplier: self.retry_backoff_multiplier,
            max_processing_time_secs: self.max_processing_time_secs,
            max_content_length: self.max_content_length,
            enable_depth_limiting: self.enable_depth_limiting,
            max_depth: self.max_depth,
            accepted_languages: self.accepted_languages,
            preferred_region: self.preferred_region,
            enable_auto_language_detection: self.enable_auto_language_detection,
            session_id: self.session_id,
            track_execution_timing: self.track_execution_timing,
        }
    }
}

impl Default for TaskConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced task content with privacy-first design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContent {
    // Content data
    text: String,
    content_type: String,
    content_length: u64,

    // Language and processing
    detected_language: Option<LangType>,
    word_count: u64,

    // Extracted data
    links: Vec<UrlString>,
    metadata: HashMap<String, String>,

    // Processing information
    processing_time: Duration,
    timestamp: SystemTime,
}

impl TaskContent {
    pub fn new(text: String, content_type: String) -> Self {
        let content_length = text.len() as u64;
        let word_count = text.split_whitespace().count() as u64;

        Self {
            text,
            content_type,
            content_length,
            detected_language: None,
            word_count,
            links: Vec::new(),
            metadata: HashMap::new(),
            processing_time: Duration::from_secs(0),
            timestamp: SystemTime::now(),
        }
    }

    pub fn with_language_detection(mut self, language: Option<LangType>) -> Self {
        self.detected_language = language;
        self
    }

    pub fn with_links(mut self, links: Vec<UrlString>) -> Self {
        self.links = links;
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_processing_time(mut self, duration: Duration) -> Self {
        self.processing_time = duration;
        self
    }

    // Getters for all fields
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn content_type(&self) -> &str {
        &self.content_type
    }
    pub fn content_length(&self) -> u64 {
        self.content_length
    }
    pub fn detected_language(&self) -> Option<LangType> {
        self.detected_language
    }
    pub fn word_count(&self) -> u64 {
        self.word_count
    }
    pub fn links(&self) -> &[UrlString] {
        &self.links
    }
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    pub fn processing_time(&self) -> Duration {
        self.processing_time
    }
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    /// Check if content meets quality thresholds
    pub fn meets_quality_threshold(&self, min_words: u64, min_length: u64) -> bool {
        self.word_count >= min_words && self.content_length >= min_length
    }

    /// Get content summary for logging
    pub fn summary(&self) -> String {
        format!(
            "Content: {} chars, {} words, type: {}, language: {:?}",
            self.content_length, self.word_count, self.content_type, self.detected_language
        )
    }
}

/// Task execution timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionTiming {
    created_at: SystemTime,
    started_at: Option<SystemTime>,
    completed_at: Option<SystemTime>,
    last_attempt_at: Option<SystemTime>,
    retry_delay_until: Option<SystemTime>,
    total_duration: Duration,
}

impl TaskExecutionTiming {
    pub fn new() -> Self {
        Self {
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            last_attempt_at: None,
            retry_delay_until: None,
            total_duration: Duration::from_secs(0),
        }
    }

    pub fn mark_started(&mut self) {
        let now = SystemTime::now();
        self.started_at = Some(now);
        self.last_attempt_at = Some(now);
    }

    pub fn mark_completed(&mut self) {
        let now = SystemTime::now();
        self.completed_at = Some(now);

        if let Some(started) = self.started_at {
            if let Ok(duration) = now.duration_since(started) {
                self.total_duration = duration;
            }
        }
    }

    pub fn mark_attempt(&mut self) {
        self.last_attempt_at = Some(SystemTime::now());
    }

    pub fn set_retry_delay(&mut self, delay: Duration) {
        let now = SystemTime::now();
        self.retry_delay_until = Some(now + delay);
    }

    pub fn is_ready_for_retry(&self) -> bool {
        self.retry_delay_until
            .map_or(true, |delay_until| SystemTime::now() >= delay_until)
    }

    // Getters for all fields
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
    pub fn started_at(&self) -> Option<SystemTime> {
        self.started_at
    }
    pub fn completed_at(&self) -> Option<SystemTime> {
        self.completed_at
    }
    pub fn last_attempt_at(&self) -> Option<SystemTime> {
        self.last_attempt_at
    }
    pub fn retry_delay_until(&self) -> Option<SystemTime> {
        self.retry_delay_until
    }
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }
}

impl Default for TaskExecutionTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced crawl task using building blocks
/// Following Rule 4: Privacy first - controlled access to task data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlTask {
    // Task identification
    task_id: TaskId,
    url: UrlString,

    // Task state
    priority: TaskPriority,
    status: TaskStatus,
    depth: u64,

    // Execution tracking
    attempt_count: u64,
    retry_count: u64,
    execution_timing: TaskExecutionTiming,

    // Error information
    last_error: Option<String>,
    error_history: Vec<String>,

    // Configuration
    config: TaskConfig,

    // Metadata
    user_agent: String,
    session_id: SessionId,
    created_by: String,
}

impl CrawlTask {
    pub fn new(url: UrlString, config: TaskConfig) -> Self {
        let task_id = TaskId::new(format!("task_{}", uuid::Uuid::new_v4()));
        let session_id = config.session_id().clone();

        Self {
            task_id,
            url,
            priority: config.default_priority(),
            status: TaskStatus::Pending,
            depth: 0,
            attempt_count: 0,
            retry_count: 0,
            execution_timing: TaskExecutionTiming::new(),
            last_error: None,
            error_history: Vec::new(),
            config,
            user_agent: "rust-web-crawler/2.0".to_string(),
            session_id,
            created_by: "system".to_string(),
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_depth(mut self, depth: u64) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn with_created_by(mut self, created_by: String) -> Self {
        self.created_by = created_by;
        self
    }

    /// Task state management
    pub fn mark_started(&mut self) {
        self.status = TaskStatus::InProgress;
        self.execution_timing.mark_started();
        self.attempt_count += 1;
    }

    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.execution_timing.mark_completed();
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.execution_timing.mark_attempt();
        self.last_error = Some(error.clone());
        self.error_history.push(error);

        if self.can_retry() {
            self.status = TaskStatus::Retrying;
            self.retry_count += 1;
            let delay = self.config.calculate_retry_delay(self.retry_count as u32);
            self.execution_timing.set_retry_delay(delay);
        } else {
            self.status = TaskStatus::Dead;
        }
    }

    /// Retry logic
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.config.max_retries() && !self.status.is_terminal()
    }

    pub fn is_ready_for_retry(&self) -> bool {
        self.execution_timing.is_ready_for_retry()
    }

    /// Validation
    pub fn validate(&self) -> ProcessingResult<()> {
        if self.url.as_str().is_empty() {
            return Err(TaskError::content_validation("Empty URL".to_string()));
        }

        if url::Url::parse(self.url.as_str()).is_err() {
            return Err(TaskError::content_validation(
                "Invalid URL format".to_string(),
            ));
        }

        if self.config.should_enable_depth_limiting() && self.depth > self.config.max_depth() {
            return Err(TaskError::content_validation(
                "Depth exceeds maximum limit".to_string(),
            ));
        }

        Ok(())
    }

    // Getters for all fields
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }
    pub fn url(&self) -> &UrlString {
        &self.url
    }
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
    pub fn status(&self) -> TaskStatus {
        self.status
    }
    pub fn depth(&self) -> u64 {
        self.depth
    }
    pub fn attempt_count(&self) -> u64 {
        self.attempt_count
    }
    pub fn retry_count(&self) -> u64 {
        self.retry_count
    }
    pub fn execution_timing(&self) -> &TaskExecutionTiming {
        &self.execution_timing
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
    pub fn error_history(&self) -> &[String] {
        &self.error_history
    }
    pub fn config(&self) -> &TaskConfig {
        &self.config
    }
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    /// Get task summary for logging
    pub fn summary(&self) -> String {
        format!(
            "Task {}: {} [{}] depth={} attempts={} retries={}",
            self.task_id.as_str(),
            self.url.as_str(),
            self.status.as_str(),
            self.depth,
            self.attempt_count,
            self.retry_count
        )
    }
}

/// Task result with comprehensive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult<T> {
    // Task identification
    task_id: TaskId,
    url: UrlString,

    // Result data
    success: bool,
    content: Option<T>,
    error: Option<TaskError>,

    // Timing information
    execution_timing: TaskExecutionTiming,

    // Metadata
    session_id: SessionId,
    timestamp: SystemTime,
}

impl<T> TaskResult<T> {
    pub fn success(
        task_id: TaskId,
        url: UrlString,
        content: T,
        execution_timing: TaskExecutionTiming,
        session_id: SessionId,
    ) -> Self {
        Self {
            task_id,
            url,
            success: true,
            content: Some(content),
            error: None,
            execution_timing,
            session_id,
            timestamp: SystemTime::now(),
        }
    }

    pub fn failure(
        task_id: TaskId,
        url: UrlString,
        error: TaskError,
        execution_timing: TaskExecutionTiming,
        session_id: SessionId,
    ) -> Self {
        Self {
            task_id,
            url,
            success: false,
            content: None,
            error: Some(error),
            execution_timing,
            session_id,
            timestamp: SystemTime::now(),
        }
    }

    // Getters for all fields
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }
    pub fn url(&self) -> &UrlString {
        &self.url
    }
    pub fn is_success(&self) -> bool {
        self.success
    }
    pub fn content(&self) -> Option<&T> {
        self.content.as_ref()
    }
    pub fn error(&self) -> Option<&TaskError> {
        self.error.as_ref()
    }
    pub fn execution_timing(&self) -> &TaskExecutionTiming {
        &self.execution_timing
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

/// Queue statistics using building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatistics {
    // Task counts
    total_tasks: u64,
    pending_tasks: u64,
    in_progress_tasks: u64,
    completed_tasks: u64,
    failed_tasks: u64,
    retrying_tasks: u64,
    dead_tasks: u64,

    // Performance metrics
    completion_rate: f64,
    average_processing_time: Duration,
    throughput_per_second: f64,

    // Timing information
    timing_stats: PerformanceMetrics,

    // Session tracking
    session_id: SessionId,
    last_updated: SystemTime,
}

impl QueueStatistics {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            total_tasks: 0,
            pending_tasks: 0,
            in_progress_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            retrying_tasks: 0,
            dead_tasks: 0,
            completion_rate: 0.0,
            average_processing_time: Duration::from_secs(0),
            throughput_per_second: 0.0,
            timing_stats: PerformanceMetrics::new(),
            session_id,
            last_updated: SystemTime::now(),
        }
    }

    pub fn update_counts(
        &mut self,
        pending: u64,
        in_progress: u64,
        completed: u64,
        failed: u64,
        retrying: u64,
        dead: u64,
    ) {
        self.pending_tasks = pending;
        self.in_progress_tasks = in_progress;
        self.completed_tasks = completed;
        self.failed_tasks = failed;
        self.retrying_tasks = retrying;
        self.dead_tasks = dead;
        self.total_tasks = pending + in_progress + completed + failed + retrying + dead;

        // Calculate completion rate
        if self.total_tasks > 0 {
            self.completion_rate = (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0;
        }

        self.last_updated = SystemTime::now();
    }

    pub fn update_timing(&mut self, timing_stats: PerformanceMetrics) {
        self.timing_stats = timing_stats;
        self.average_processing_time =
            Duration::from_millis(timing_stats.average_duration().as_millis() as u64);
    }

    pub fn update_throughput(&mut self, throughput: f64) {
        self.throughput_per_second = throughput;
    }

    // Getters for all fields
    pub fn total_tasks(&self) -> u64 {
        self.total_tasks
    }
    pub fn pending_tasks(&self) -> u64 {
        self.pending_tasks
    }
    pub fn in_progress_tasks(&self) -> u64 {
        self.in_progress_tasks
    }
    pub fn completed_tasks(&self) -> u64 {
        self.completed_tasks
    }
    pub fn failed_tasks(&self) -> u64 {
        self.failed_tasks
    }
    pub fn retrying_tasks(&self) -> u64 {
        self.retrying_tasks
    }
    pub fn dead_tasks(&self) -> u64 {
        self.dead_tasks
    }
    pub fn completion_rate(&self) -> f64 {
        self.completion_rate
    }
    pub fn average_processing_time(&self) -> Duration {
        self.average_processing_time
    }
    pub fn throughput_per_second(&self) -> f64 {
        self.throughput_per_second
    }
    pub fn timing_stats(&self) -> &PerformanceMetrics {
        &self.timing_stats
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn last_updated(&self) -> SystemTime {
        self.last_updated
    }
}

impl Default for QueueStatistics {
    fn default() -> Self {
        Self::new(SessionId::new("default_queue".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_conversion() {
        assert_eq!(Region::NorthAmerica.as_str(), "north_america");
        assert_eq!(Region::from_str("europe"), Some(Region::Europe));
        assert_eq!(Region::from_str("invalid"), None);
    }

    #[test]
    fn test_lang_type_http_variants() {
        let variants = LangType::English.to_http_variants();
        assert!(variants.contains(&"en-US"));
        assert!(variants.contains(&"en"));
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Critical > TaskPriority::High);
    }

    #[test]
    fn test_task_status_terminal() {
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Dead.is_terminal());
        assert!(!TaskStatus::Pending.is_terminal());
    }

    #[test]
    fn test_task_config_builder() {
        let config = TaskConfig::builder()
            .with_priority_settings(TaskPriority::High, LimitValue::new(5))
            .with_language_settings(
                vec![LangType::English, LangType::French],
                BooleanFlag::enabled(),
            )
            .build();

        assert_eq!(config.default_priority(), TaskPriority::High);
        assert_eq!(config.max_retries(), 5);
        assert_eq!(config.accepted_languages().len(), 2);
    }

    #[test]
    fn test_task_content_creation() {
        let content = TaskContent::new("Hello world".to_string(), "text/plain".to_string())
            .with_language_detection(Some(LangType::English));

        assert_eq!(content.text(), "Hello world");
        assert_eq!(content.word_count(), 2);
        assert_eq!(content.detected_language(), Some(LangType::English));
    }

    #[test]
    fn test_crawl_task_creation() {
        let config = TaskConfig::default();
        let url = UrlString::new("https://example.com".to_string());
        let task = CrawlTask::new(url.clone(), config)
            .with_priority(TaskPriority::High)
            .with_depth(2);

        assert_eq!(task.url(), &url);
        assert_eq!(task.priority(), TaskPriority::High);
        assert_eq!(task.depth(), 2);
        assert_eq!(task.status(), TaskStatus::Pending);
    }

    #[test]
    fn test_task_retry_logic() {
        let config = TaskConfig::builder()
            .with_priority_settings(TaskPriority::Normal, LimitValue::new(2))
            .build();
        let url = UrlString::new("https://example.com".to_string());
        let mut task = CrawlTask::new(url, config);

        assert!(task.can_retry());

        task.mark_failed("Test error 1".to_string());
        assert_eq!(task.status(), TaskStatus::Retrying);
        assert_eq!(task.retry_count(), 1);
        assert!(task.can_retry());

        task.mark_failed("Test error 2".to_string());
        assert_eq!(task.status(), TaskStatus::Dead);
        assert_eq!(task.retry_count(), 2);
        assert!(!task.can_retry());
    }

    #[test]
    fn test_task_result_creation() {
        let task_id = TaskId::new("test_task".to_string());
        let url = UrlString::new("https://example.com".to_string());
        let session_id = SessionId::new("test_session".to_string());
        let timing = TaskExecutionTiming::new();

        let result = TaskResult::success(
            task_id.clone(),
            url.clone(),
            "test content",
            timing,
            session_id.clone(),
        );

        assert!(result.is_success());
        assert_eq!(result.task_id(), &task_id);
        assert_eq!(result.url(), &url);
        assert_eq!(result.content(), Some(&"test content"));
        assert!(result.error().is_none());
    }

    #[test]
    fn test_queue_statistics() {
        let session_id = SessionId::new("test_queue".to_string());
        let mut stats = QueueStatistics::new(session_id.clone());

        stats.update_counts(10, 2, 5, 1, 1, 1);
        assert_eq!(stats.total_tasks(), 20);
        assert_eq!(stats.completion_rate(), 25.0);
    }
}
