/// Session management module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex structures  
/// Following Rule 4: Privacy first - controlled access only
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, ConfigResult, CountValue, CrawlerConfig, DelayDuration, LimitValue, NetworkResult,
    PerformanceMetrics, SessionId, SessionResult, TaskCounts, TaskError, TaskResult,
    TimeoutDuration, UrlString,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};

/// Configuration for crawl session behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    session_id: SessionId,
    max_concurrent_requests: LimitValue,
    max_depth: LimitValue,
    max_total_tasks: LimitValue,
    session_timeout: Option<TimeoutDuration>,
    storage_enabled: BooleanFlag,
    storage_path: Option<PathBuf>,
    metrics_enabled: BooleanFlag,
    auto_cleanup: BooleanFlag,
    cleanup_interval: DelayDuration,
}

impl SessionConfig {
    pub fn builder() -> SessionConfigBuilder {
        SessionConfigBuilder::new()
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn max_concurrent_requests(&self) -> u64 {
        self.max_concurrent_requests.value()
    }

    pub fn max_depth(&self) -> u64 {
        self.max_depth.value()
    }

    pub fn max_total_tasks(&self) -> u64 {
        self.max_total_tasks.value()
    }

    pub fn session_timeout(&self) -> Option<TimeoutDuration> {
        self.session_timeout
    }

    pub fn is_storage_enabled(&self) -> bool {
        self.storage_enabled.is_enabled()
    }

    pub fn storage_path(&self) -> Option<&PathBuf> {
        self.storage_path.as_ref()
    }

    pub fn is_metrics_enabled(&self) -> bool {
        self.metrics_enabled.is_enabled()
    }

    pub fn is_auto_cleanup_enabled(&self) -> bool {
        self.auto_cleanup.is_enabled()
    }

    pub fn cleanup_interval(&self) -> DelayDuration {
        self.cleanup_interval
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_id: SessionId::new(format!("session_{}", uuid::Uuid::new_v4())),
            max_concurrent_requests: LimitValue::new(5),
            max_depth: LimitValue::new(3),
            max_total_tasks: LimitValue::new(1000),
            session_timeout: Some(TimeoutDuration::from_secs(300)), // 5 minutes
            storage_enabled: BooleanFlag::enabled(),
            storage_path: Some(PathBuf::from("./crawl_data")),
            metrics_enabled: BooleanFlag::enabled(),
            auto_cleanup: BooleanFlag::enabled(),
            cleanup_interval: DelayDuration::from_secs(60),
        }
    }
}

/// Builder for session configuration
#[derive(Debug)]
pub struct SessionConfigBuilder {
    session_id: Option<SessionId>,
    max_concurrent_requests: LimitValue,
    max_depth: LimitValue,
    max_total_tasks: LimitValue,
    session_timeout: Option<TimeoutDuration>,
    storage_enabled: BooleanFlag,
    storage_path: Option<PathBuf>,
    metrics_enabled: BooleanFlag,
    auto_cleanup: BooleanFlag,
    cleanup_interval: DelayDuration,
}

impl SessionConfigBuilder {
    pub fn new() -> Self {
        let default_config = SessionConfig::default();
        Self {
            session_id: None,
            max_concurrent_requests: default_config.max_concurrent_requests,
            max_depth: default_config.max_depth,
            max_total_tasks: default_config.max_total_tasks,
            session_timeout: default_config.session_timeout,
            storage_enabled: default_config.storage_enabled,
            storage_path: default_config.storage_path,
            metrics_enabled: default_config.metrics_enabled,
            auto_cleanup: default_config.auto_cleanup,
            cleanup_interval: default_config.cleanup_interval,
        }
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_max_concurrent_requests(mut self, limit: LimitValue) -> Self {
        self.max_concurrent_requests = limit;
        self
    }

    pub fn with_max_depth(mut self, limit: LimitValue) -> Self {
        self.max_depth = limit;
        self
    }

    pub fn with_max_total_tasks(mut self, limit: LimitValue) -> Self {
        self.max_total_tasks = limit;
        self
    }

    pub fn with_session_timeout(mut self, timeout: Option<TimeoutDuration>) -> Self {
        self.session_timeout = timeout;
        self
    }

    pub fn with_storage_enabled(mut self, enabled: BooleanFlag) -> Self {
        self.storage_enabled = enabled;
        self
    }

    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    pub fn with_metrics_enabled(mut self, enabled: BooleanFlag) -> Self {
        self.metrics_enabled = enabled;
        self
    }

    pub fn with_auto_cleanup(mut self, enabled: BooleanFlag) -> Self {
        self.auto_cleanup = enabled;
        self
    }

    pub fn with_cleanup_interval(mut self, interval: DelayDuration) -> Self {
        self.cleanup_interval = interval;
        self
    }

    pub fn build(self) -> SessionConfig {
        let session_id = self
            .session_id
            .unwrap_or_else(|| SessionId::new(format!("session_{}", uuid::Uuid::new_v4())));

        SessionConfig {
            session_id,
            max_concurrent_requests: self.max_concurrent_requests,
            max_depth: self.max_depth,
            max_total_tasks: self.max_total_tasks,
            session_timeout: self.session_timeout,
            storage_enabled: self.storage_enabled,
            storage_path: self.storage_path,
            metrics_enabled: self.metrics_enabled,
            auto_cleanup: self.auto_cleanup,
            cleanup_interval: self.cleanup_interval,
        }
    }
}

impl Default for SessionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Session state tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Initialized,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState::Created
    }
}

/// Session execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    task_counts: TaskCounts,
    performance_metrics: PerformanceMetrics,
    start_time: SystemTime,
    end_time: Option<SystemTime>,
    duration: Option<Duration>,
    urls_processed: CountValue,
    errors_encountered: CountValue,
}

impl SessionMetrics {
    pub fn new() -> Self {
        Self {
            task_counts: TaskCounts::default(),
            performance_metrics: PerformanceMetrics::default(),
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            urls_processed: CountValue::default(),
            errors_encountered: CountValue::default(),
        }
    }

    pub fn task_counts(&self) -> &TaskCounts {
        &self.task_counts
    }

    pub fn task_counts_mut(&mut self) -> &mut TaskCounts {
        &mut self.task_counts
    }

    pub fn performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    pub fn performance_metrics_mut(&mut self) -> &mut PerformanceMetrics {
        &mut self.performance_metrics
    }

    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    pub fn end_time(&self) -> Option<SystemTime> {
        self.end_time
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn urls_processed(&self) -> u64 {
        self.urls_processed.value()
    }

    pub fn errors_encountered(&self) -> u64 {
        self.errors_encountered.value()
    }

    pub fn complete(&mut self) {
        let now = SystemTime::now();
        self.end_time = Some(now);
        self.duration = now.duration_since(self.start_time).ok();
    }

    pub fn increment_urls_processed(&mut self) {
        self.urls_processed.increment();
    }

    pub fn increment_errors(&mut self) {
        self.errors_encountered.increment();
    }

    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }

    pub fn success_rate(&self) -> f64 {
        self.task_counts.success_rate().value()
    }
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Session result type for consistent error handling
pub type SessionConfigResult<T> = Result<T, TaskError>;

/// Complete crawl session manager using building blocks
/// Following Rule 4: Privacy first - all fields private with controlled access
pub struct CrawlSession {
    // Core session data (private)
    config: SessionConfig,
    state: Arc<RwLock<SessionState>>,
    metrics: Arc<RwLock<SessionMetrics>>,

    // Session data storage (private)
    results: Arc<RwLock<Vec<TaskResult<crate::common::TaskContent>>>>,
    errors: Arc<RwLock<Vec<TaskError>>>,
    metadata: Arc<RwLock<HashMap<String, String>>>,

    // Execution control (private)
    execution_start: Option<SystemTime>,
    cancellation_token: Arc<tokio_util::sync::CancellationToken>,
}

impl CrawlSession {
    /// Create a new crawl session with the given configuration
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(SessionState::Created)),
            metrics: Arc::new(RwLock::new(SessionMetrics::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            execution_start: None,
            cancellation_token: Arc::new(tokio_util::sync::CancellationToken::new()),
        }
    }

    /// Create a session with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SessionConfig::default())
    }

    /// Get the session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Get the session ID
    pub fn session_id(&self) -> &SessionId {
        self.config.session_id()
    }

    /// Get the current session state
    pub async fn state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Get the current session metrics
    pub async fn metrics(&self) -> SessionMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if the session is running
    pub async fn is_running(&self) -> bool {
        matches!(self.state().await, SessionState::Running)
    }

    /// Check if the session is completed
    pub async fn is_completed(&self) -> bool {
        matches!(
            self.state().await,
            SessionState::Completed | SessionState::Failed | SessionState::Cancelled
        )
    }

    /// Check if the session has timed out
    pub async fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.config.session_timeout() {
            if let Some(start_time) = self.execution_start {
                return start_time.elapsed().unwrap_or_default() > timeout.duration();
            }
        }
        false
    }

    /// Initialize the session
    pub async fn initialize(&mut self) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        match *state {
            SessionState::Created => {
                *state = SessionState::Initialized;
                self.execution_start = Some(SystemTime::now());
                Ok(())
            }
            _ => Err(TaskError::session("Session already initialized")),
        }
    }

    /// Start the session execution
    pub async fn start(&self) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        match *state {
            SessionState::Initialized | SessionState::Paused => {
                *state = SessionState::Running;
                Ok(())
            }
            SessionState::Created => Err(TaskError::session(
                "Session must be initialized before starting",
            )),
            _ => Err(TaskError::session(
                "Session cannot be started in current state",
            )),
        }
    }

    /// Pause the session execution
    pub async fn pause(&self) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        match *state {
            SessionState::Running => {
                *state = SessionState::Paused;
                Ok(())
            }
            _ => Err(TaskError::session("Session must be running to pause")),
        }
    }

    /// Complete the session
    pub async fn complete(&self) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        *state = SessionState::Completed;

        // Complete metrics
        let mut metrics = self.metrics.write().await;
        metrics.complete();

        Ok(())
    }

    /// Fail the session with an error
    pub async fn fail(&self, error: TaskError) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        *state = SessionState::Failed;

        // Record the error
        {
            let mut errors = self.errors.write().await;
            errors.push(error);
        }

        // Complete metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.complete();
        }

        Ok(())
    }

    /// Cancel the session
    pub async fn cancel(&self) -> SessionConfigResult<()> {
        let mut state = self.state.write().await;

        *state = SessionState::Cancelled;
        self.cancellation_token.cancel();

        // Complete metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.complete();
        }

        Ok(())
    }

    /// Add a task result to the session
    pub async fn add_result(
        &self,
        result: TaskResult<crate::common::TaskContent>,
    ) -> SessionConfigResult<()> {
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;

            if result.is_success() {
                metrics.task_counts_mut().increment_completed();
                metrics.increment_urls_processed();
            } else {
                metrics.task_counts_mut().increment_failed();
                metrics.increment_errors();

                // Record error
                if let Some(error) = result.error() {
                    let mut errors = self.errors.write().await;
                    errors.push(error.clone());
                }
            }
        }

        // Store result
        {
            let mut results = self.results.write().await;
            results.push(result);
        }

        Ok(())
    }

    /// Get all results from the session
    pub async fn results(&self) -> Vec<TaskResult<crate::common::TaskContent>> {
        self.results.read().await.clone()
    }

    /// Get all errors from the session
    pub async fn errors(&self) -> Vec<TaskError> {
        self.errors.read().await.clone()
    }

    /// Get successful results only
    pub async fn successful_results(&self) -> Vec<TaskResult<crate::common::TaskContent>> {
        self.results
            .read()
            .await
            .iter()
            .filter(|result| result.is_success())
            .cloned()
            .collect()
    }

    /// Get failed results only
    pub async fn failed_results(&self) -> Vec<TaskResult<crate::common::TaskContent>> {
        self.results
            .read()
            .await
            .iter()
            .filter(|result| result.is_failure())
            .cloned()
            .collect()
    }

    /// Add metadata to the session
    pub async fn add_metadata(&self, key: String, value: String) -> SessionConfigResult<()> {
        let mut metadata = self.metadata.write().await;
        metadata.insert(key, value);
        Ok(())
    }

    /// Get session metadata
    pub async fn metadata(&self) -> HashMap<String, String> {
        self.metadata.read().await.clone()
    }

    /// Get the cancellation token for graceful shutdown
    pub fn cancellation_token(&self) -> Arc<tokio_util::sync::CancellationToken> {
        self.cancellation_token.clone()
    }

    /// Convert session to final result
    pub async fn into_session_result(self) -> SessionResult {
        let mut session_result = SessionResult::new(self.config.session_id().clone());

        // Add all results
        let results = self.results.read().await;
        for result in results.iter() {
            session_result.add_result(result.clone());
        }

        // Complete if not already done
        session_result.complete();

        session_result
    }

    /// Get current session summary for external APIs
    pub async fn get_summary(&self) -> SessionSummary {
        let state = self.state().await;
        let metrics = self.metrics().await;
        let result_count = self.results.read().await.len();
        let error_count = self.errors.read().await.len();

        SessionSummary {
            session_id: self.config.session_id().clone(),
            state,
            total_tasks: metrics.task_counts().total(),
            completed_tasks: metrics.task_counts().completed(),
            failed_tasks: metrics.task_counts().failed(),
            success_rate: metrics.success_rate(),
            urls_processed: metrics.urls_processed(),
            errors_encountered: metrics.errors_encountered(),
            duration: metrics.duration(),
            is_completed: metrics.is_completed(),
        }
    }

    /// Check if session should be automatically cleaned up
    pub async fn should_cleanup(&self) -> bool {
        if !self.config.is_auto_cleanup_enabled() {
            return false;
        }

        self.is_completed().await
            && self
                .execution_start
                .map(|start| {
                    start.elapsed().unwrap_or_default() > self.config.cleanup_interval().duration()
                })
                .unwrap_or(false)
    }
}

impl Default for CrawlSession {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Summary of session status for external consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    session_id: SessionId,
    state: SessionState,
    total_tasks: u64,
    completed_tasks: u64,
    failed_tasks: u64,
    success_rate: f64,
    urls_processed: u64,
    errors_encountered: u64,
    duration: Option<Duration>,
    is_completed: bool,
}

impl SessionSummary {
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn total_tasks(&self) -> u64 {
        self.total_tasks
    }

    pub fn completed_tasks(&self) -> u64 {
        self.completed_tasks
    }

    pub fn failed_tasks(&self) -> u64 {
        self.failed_tasks
    }

    pub fn success_rate(&self) -> f64 {
        self.success_rate
    }

    pub fn urls_processed(&self) -> u64 {
        self.urls_processed
    }

    pub fn errors_encountered(&self) -> u64 {
        self.errors_encountered
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn is_completed(&self) -> bool {
        self.is_completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig::builder()
            .with_session_id(SessionId::new("test-session"))
            .with_max_concurrent_requests(LimitValue::new(10))
            .build();

        let session = CrawlSession::new(config);

        assert_eq!(session.session_id().as_str(), "test-session");
        assert_eq!(session.config().max_concurrent_requests(), 10);
        assert_eq!(session.state().await, SessionState::Created);
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let mut session = CrawlSession::with_defaults();

        // Initialize
        session.initialize().await.expect("Failed to initialize");
        assert_eq!(session.state().await, SessionState::Initialized);

        // Start
        session.start().await.expect("Failed to start");
        assert_eq!(session.state().await, SessionState::Running);
        assert!(session.is_running().await);

        // Pause
        session.pause().await.expect("Failed to pause");
        assert_eq!(session.state().await, SessionState::Paused);

        // Resume
        session.start().await.expect("Failed to resume");
        assert_eq!(session.state().await, SessionState::Running);

        // Complete
        session.complete().await.expect("Failed to complete");
        assert_eq!(session.state().await, SessionState::Completed);
        assert!(session.is_completed().await);
    }

    #[tokio::test]
    async fn test_session_metrics() {
        let session = CrawlSession::with_defaults();

        // Create a mock result
        let task_result = TaskResult::success(
            crate::common::TaskId::new("test-task"),
            crate::common::UrlString::new("https://example.com"),
            crate::common::TaskContent::default(),
            crate::common::ExecutionTiming::default(),
        );

        session
            .add_result(task_result)
            .await
            .expect("Failed to add result");

        let metrics = session.metrics().await;
        assert_eq!(metrics.task_counts().completed(), 1);
        assert_eq!(metrics.urls_processed(), 1);

        let summary = session.get_summary().await;
        assert_eq!(summary.completed_tasks(), 1);
        assert!(summary.success_rate() > 0.0);
    }

    #[tokio::test]
    async fn test_session_timeout() {
        let config = SessionConfig::builder()
            .with_session_timeout(Some(TimeoutDuration::from_millis(100)))
            .build();

        let mut session = CrawlSession::new(config);
        session.initialize().await.expect("Failed to initialize");

        // Wait for timeout
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        assert!(session.is_timed_out().await);
    }
}
