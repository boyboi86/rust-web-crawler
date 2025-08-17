/// Metrics collection module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all metrics configuration external
/// Following Rule 3: Builder pattern for complex metrics configurations
/// Following Rule 4: Privacy first - controlled access to metrics data
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
/// Following Rule 9: Fearless concurrency - thread-safe atomic operations
use crate::common::{
    BooleanFlag, ConfigResult, LimitValue, PerformanceMetrics, ProcessingResult, SessionId,
    TaskError, UrlString,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    // Collection settings
    collect_domain_stats: BooleanFlag,
    collect_timing_stats: BooleanFlag,
    collect_error_stats: BooleanFlag,
    collect_queue_stats: BooleanFlag,

    // Retention settings
    max_domain_entries: LimitValue,
    max_error_entries: LimitValue,
    retention_hours: LimitValue,

    // Reporting settings
    snapshot_interval_secs: LimitValue,
    top_domains_count: LimitValue,
    detailed_reporting: BooleanFlag,

    // Session tracking
    session_id: SessionId,
    enable_persistence: BooleanFlag,
}

impl MetricsConfig {
    pub fn builder() -> MetricsConfigBuilder {
        MetricsConfigBuilder::new()
    }

    // Getters for all configuration fields
    pub fn should_collect_domain_stats(&self) -> bool {
        self.collect_domain_stats.is_enabled()
    }
    pub fn should_collect_timing_stats(&self) -> bool {
        self.collect_timing_stats.is_enabled()
    }
    pub fn should_collect_error_stats(&self) -> bool {
        self.collect_error_stats.is_enabled()
    }
    pub fn should_collect_queue_stats(&self) -> bool {
        self.collect_queue_stats.is_enabled()
    }
    pub fn max_domain_entries(&self) -> u64 {
        self.max_domain_entries.value()
    }
    pub fn max_error_entries(&self) -> u64 {
        self.max_error_entries.value()
    }
    pub fn retention_hours(&self) -> u64 {
        self.retention_hours.value()
    }
    pub fn snapshot_interval_secs(&self) -> u64 {
        self.snapshot_interval_secs.value()
    }
    pub fn top_domains_count(&self) -> u64 {
        self.top_domains_count.value()
    }
    pub fn should_use_detailed_reporting(&self) -> bool {
        self.detailed_reporting.is_enabled()
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn should_enable_persistence(&self) -> bool {
        self.enable_persistence.is_enabled()
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collect_domain_stats: BooleanFlag::enabled(),
            collect_timing_stats: BooleanFlag::enabled(),
            collect_error_stats: BooleanFlag::enabled(),
            collect_queue_stats: BooleanFlag::enabled(),
            max_domain_entries: LimitValue::new(1000),
            max_error_entries: LimitValue::new(500),
            retention_hours: LimitValue::new(24),
            snapshot_interval_secs: LimitValue::new(60),
            top_domains_count: LimitValue::new(10),
            detailed_reporting: BooleanFlag::enabled(),
            session_id: SessionId::new(format!("metrics_{}", uuid::Uuid::new_v4())),
            enable_persistence: BooleanFlag::disabled(),
        }
    }
}

/// Builder for metrics configuration
#[derive(Debug)]
pub struct MetricsConfigBuilder {
    collect_domain_stats: BooleanFlag,
    collect_timing_stats: BooleanFlag,
    collect_error_stats: BooleanFlag,
    collect_queue_stats: BooleanFlag,
    max_domain_entries: LimitValue,
    max_error_entries: LimitValue,
    retention_hours: LimitValue,
    snapshot_interval_secs: LimitValue,
    top_domains_count: LimitValue,
    detailed_reporting: BooleanFlag,
    session_id: SessionId,
    enable_persistence: BooleanFlag,
}

impl MetricsConfigBuilder {
    pub fn new() -> Self {
        let default_config = MetricsConfig::default();
        Self {
            collect_domain_stats: default_config.collect_domain_stats,
            collect_timing_stats: default_config.collect_timing_stats,
            collect_error_stats: default_config.collect_error_stats,
            collect_queue_stats: default_config.collect_queue_stats,
            max_domain_entries: default_config.max_domain_entries,
            max_error_entries: default_config.max_error_entries,
            retention_hours: default_config.retention_hours,
            snapshot_interval_secs: default_config.snapshot_interval_secs,
            top_domains_count: default_config.top_domains_count,
            detailed_reporting: default_config.detailed_reporting,
            session_id: default_config.session_id,
            enable_persistence: default_config.enable_persistence,
        }
    }

    pub fn with_collection_settings(
        mut self,
        domain_stats: BooleanFlag,
        timing_stats: BooleanFlag,
        error_stats: BooleanFlag,
        queue_stats: BooleanFlag,
    ) -> Self {
        self.collect_domain_stats = domain_stats;
        self.collect_timing_stats = timing_stats;
        self.collect_error_stats = error_stats;
        self.collect_queue_stats = queue_stats;
        self
    }

    pub fn with_retention_settings(
        mut self,
        max_domains: LimitValue,
        max_errors: LimitValue,
        retention_hours: LimitValue,
    ) -> Self {
        self.max_domain_entries = max_domains;
        self.max_error_entries = max_errors;
        self.retention_hours = retention_hours;
        self
    }

    pub fn with_reporting_settings(
        mut self,
        snapshot_interval: LimitValue,
        top_domains_count: LimitValue,
        detailed: BooleanFlag,
    ) -> Self {
        self.snapshot_interval_secs = snapshot_interval;
        self.top_domains_count = top_domains_count;
        self.detailed_reporting = detailed;
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_persistence(mut self, enabled: BooleanFlag) -> Self {
        self.enable_persistence = enabled;
        self
    }

    pub fn build(self) -> MetricsConfig {
        MetricsConfig {
            collect_domain_stats: self.collect_domain_stats,
            collect_timing_stats: self.collect_timing_stats,
            collect_error_stats: self.collect_error_stats,
            collect_queue_stats: self.collect_queue_stats,
            max_domain_entries: self.max_domain_entries,
            max_error_entries: self.max_error_entries,
            retention_hours: self.retention_hours,
            snapshot_interval_secs: self.snapshot_interval_secs,
            top_domains_count: self.top_domains_count,
            detailed_reporting: self.detailed_reporting,
            session_id: self.session_id,
            enable_persistence: self.enable_persistence,
        }
    }
}

impl Default for MetricsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Domain-specific metrics with privacy control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainMetrics {
    // Request counters
    requests_count: u64,
    success_count: u64,
    failure_count: u64,

    // Timing statistics
    timing_stats: PerformanceMetrics,

    // Data transfer
    total_bytes: u64,

    // Timestamps
    first_request_time: SystemTime,
    last_request_time: SystemTime,

    // Error tracking
    error_categories: HashMap<String, u64>,
}

impl DomainMetrics {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            requests_count: 0,
            success_count: 0,
            failure_count: 0,
            timing_stats: PerformanceMetrics::new(),
            total_bytes: 0,
            first_request_time: now,
            last_request_time: now,
            error_categories: HashMap::new(),
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, duration: Duration, bytes: u64) {
        self.requests_count += 1;
        self.success_count += 1;
        self.timing_stats.record_duration(duration);
        self.total_bytes += bytes;
        self.last_request_time = SystemTime::now();
    }

    /// Record a failed request
    pub fn record_failure(&mut self, duration: Duration, error_category: Option<&str>) {
        self.requests_count += 1;
        self.failure_count += 1;
        self.timing_stats.record_duration(duration);
        self.last_request_time = SystemTime::now();

        if let Some(category) = error_category {
            *self
                .error_categories
                .entry(category.to_string())
                .or_insert(0) += 1;
        }
    }

    // Getters for all fields
    pub fn requests_count(&self) -> u64 {
        self.requests_count
    }
    pub fn success_count(&self) -> u64 {
        self.success_count
    }
    pub fn failure_count(&self) -> u64 {
        self.failure_count
    }
    pub fn success_rate(&self) -> f64 {
        if self.requests_count > 0 {
            (self.success_count as f64 / self.requests_count as f64) * 100.0
        } else {
            0.0
        }
    }
    pub fn timing_stats(&self) -> &PerformanceMetrics {
        &self.timing_stats
    }
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
    pub fn first_request_time(&self) -> SystemTime {
        self.first_request_time
    }
    pub fn last_request_time(&self) -> SystemTime {
        self.last_request_time
    }
    pub fn error_categories(&self) -> &HashMap<String, u64> {
        &self.error_categories
    }
}

impl Default for DomainMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Error statistics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    error_counts: HashMap<String, u64>,
    total_errors: u64,
    first_error_time: Option<SystemTime>,
    last_error_time: Option<SystemTime>,
}

impl ErrorStatistics {
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            total_errors: 0,
            first_error_time: None,
            last_error_time: None,
        }
    }

    /// Record an error occurrence
    pub fn record_error(&mut self, error_type: &str) {
        let now = SystemTime::now();

        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        self.total_errors += 1;

        if self.first_error_time.is_none() {
            self.first_error_time = Some(now);
        }
        self.last_error_time = Some(now);
    }

    // Getters for all fields
    pub fn error_counts(&self) -> &HashMap<String, u64> {
        &self.error_counts
    }
    pub fn total_errors(&self) -> u64 {
        self.total_errors
    }
    pub fn first_error_time(&self) -> Option<SystemTime> {
        self.first_error_time
    }
    pub fn last_error_time(&self) -> Option<SystemTime> {
        self.last_error_time
    }

    /// Get the most common error types
    pub fn top_errors(&self, count: usize) -> Vec<(String, u64)> {
        let mut sorted: Vec<_> = self
            .error_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(count);
        sorted
    }
}

impl Default for ErrorStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue metrics tracking
#[derive(Debug, Serialize, Deserialize)]
pub struct QueueMetrics {
    tasks_enqueued: AtomicU64,
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,
    tasks_retried: AtomicU64,
    queue_timing: Arc<RwLock<PerformanceMetrics>>,
}

impl QueueMetrics {
    pub fn new() -> Self {
        Self {
            tasks_enqueued: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            tasks_retried: AtomicU64::new(0),
            queue_timing: Arc::new(RwLock::new(PerformanceMetrics::new())),
        }
    }

    /// Record task operations
    pub fn record_enqueued(&self) {
        self.tasks_enqueued.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_completed(&self) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retried(&self) {
        self.tasks_retried.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn record_processing_time(&self, duration: Duration) {
        let mut timing = self.queue_timing.write().await;
        timing.record_duration(duration);
    }

    // Getters for metrics
    pub fn tasks_enqueued(&self) -> u64 {
        self.tasks_enqueued.load(Ordering::Relaxed)
    }
    pub fn tasks_completed(&self) -> u64 {
        self.tasks_completed.load(Ordering::Relaxed)
    }
    pub fn tasks_failed(&self) -> u64 {
        self.tasks_failed.load(Ordering::Relaxed)
    }
    pub fn tasks_retried(&self) -> u64 {
        self.tasks_retried.load(Ordering::Relaxed)
    }

    pub fn completion_rate(&self) -> f64 {
        let enqueued = self.tasks_enqueued();
        if enqueued > 0 {
            (self.tasks_completed() as f64 / enqueued as f64) * 100.0
        } else {
            0.0
        }
    }

    pub async fn timing_statistics(&self) -> PerformanceMetrics {
        self.queue_timing.read().await.clone()
    }
}

impl Default for QueueMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive metrics collector following building blocks pattern
/// Following Rule 4: Privacy first - metrics access controlled
pub struct CrawlerMetrics {
    // Configuration
    config: MetricsConfig,

    // Start time for uptime calculation
    start_time: Instant,

    // Core metrics (atomic for thread safety)
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    total_bytes: AtomicU64,

    // Complex metrics (protected by RwLock)
    domain_metrics: Arc<RwLock<HashMap<String, DomainMetrics>>>,
    error_statistics: Arc<RwLock<ErrorStatistics>>,
    queue_metrics: QueueMetrics,
    timing_statistics: Arc<RwLock<PerformanceMetrics>>,
}

impl CrawlerMetrics {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            domain_metrics: Arc::new(RwLock::new(HashMap::new())),
            error_statistics: Arc::new(RwLock::new(ErrorStatistics::new())),
            queue_metrics: QueueMetrics::new(),
            timing_statistics: Arc::new(RwLock::new(PerformanceMetrics::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(MetricsConfig::default())
    }

    /// Record a successful crawl operation
    pub async fn record_success(
        &self,
        url: &UrlString,
        duration: Duration,
        bytes: u64,
    ) -> ProcessingResult<()> {
        // Update core metrics
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);

        // Update timing statistics
        if self.config.should_collect_timing_stats() {
            let mut timing = self.timing_statistics.write().await;
            timing.record_duration(duration);
        }

        // Update domain metrics
        if self.config.should_collect_domain_stats() {
            self.update_domain_metrics(url, |metrics| {
                metrics.record_success(duration, bytes);
            })
            .await?;
        }

        Ok(())
    }

    /// Record a failed crawl operation
    pub async fn record_failure(
        &self,
        url: &UrlString,
        duration: Duration,
        error: &TaskError,
    ) -> ProcessingResult<()> {
        // Update core metrics
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);

        // Update timing statistics
        if self.config.should_collect_timing_stats() {
            let mut timing = self.timing_statistics.write().await;
            timing.record_duration(duration);
        }

        // Update error statistics
        if self.config.should_collect_error_stats() {
            let mut error_stats = self.error_statistics.write().await;
            error_stats.record_error(&error.error_type());
        }

        // Update domain metrics
        if self.config.should_collect_domain_stats() {
            self.update_domain_metrics(url, |metrics| {
                metrics.record_failure(duration, Some(&error.error_type()));
            })
            .await?;
        }

        Ok(())
    }

    /// Update domain-specific metrics
    async fn update_domain_metrics<F>(&self, url: &UrlString, update_fn: F) -> ProcessingResult<()>
    where
        F: FnOnce(&mut DomainMetrics),
    {
        let domain = self.extract_domain(url)?;
        let mut domain_stats = self.domain_metrics.write().await;

        // Check if we need to cleanup old entries
        if domain_stats.len() >= self.config.max_domain_entries() as usize {
            self.cleanup_old_domains(&mut domain_stats).await;
        }

        let metrics = domain_stats
            .entry(domain)
            .or_insert_with(DomainMetrics::new);
        update_fn(metrics);

        Ok(())
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &UrlString) -> ProcessingResult<String> {
        url::Url::parse(url.as_str())
            .map_err(|e| TaskError::content_validation(format!("Invalid URL: {}", e)))?
            .host_str()
            .map(|h| h.to_string())
            .ok_or_else(|| TaskError::content_validation("No host in URL".to_string()))
    }

    /// Cleanup old domain entries to stay within limits
    async fn cleanup_old_domains(&self, domain_stats: &mut HashMap<String, DomainMetrics>) {
        let target_size = (self.config.max_domain_entries() as f64 * 0.8) as usize;
        if domain_stats.len() <= target_size {
            return;
        }

        // Sort by last request time and keep the most recent
        let mut entries: Vec<_> = domain_stats
            .iter()
            .map(|(domain, metrics)| (domain.clone(), metrics.last_request_time()))
            .collect();

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(target_size);

        let keep_domains: std::collections::HashSet<_> =
            entries.into_iter().map(|(domain, _)| domain).collect();

        domain_stats.retain(|domain, _| keep_domains.contains(domain));
    }

    /// Record queue operations
    pub fn record_task_enqueued(&self) {
        if self.config.should_collect_queue_stats() {
            self.queue_metrics.record_enqueued();
        }
    }

    pub fn record_task_completed(&self) {
        if self.config.should_collect_queue_stats() {
            self.queue_metrics.record_completed();
        }
    }

    pub fn record_task_failed(&self) {
        if self.config.should_collect_queue_stats() {
            self.queue_metrics.record_failed();
        }
    }

    pub fn record_task_retried(&self) {
        if self.config.should_collect_queue_stats() {
            self.queue_metrics.record_retried();
        }
    }

    pub async fn record_queue_processing_time(&self, duration: Duration) {
        if self.config.should_collect_queue_stats() {
            self.queue_metrics.record_processing_time(duration).await;
        }
    }

    /// Get comprehensive metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let uptime = self.start_time.elapsed();
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);

        let success_rate = if total_requests > 0 {
            (successful as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let requests_per_second = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        let bytes_per_second = if uptime.as_secs() > 0 {
            total_bytes as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        // Get timing statistics
        let timing = self.timing_statistics.read().await.clone();

        // Get domain metrics
        let domain_stats = self.domain_metrics.read().await;
        let mut top_domains: Vec<_> = domain_stats
            .iter()
            .map(|(domain, metrics)| (domain.clone(), metrics.clone()))
            .collect();
        top_domains.sort_by(|a, b| b.1.requests_count().cmp(&a.1.requests_count()));
        top_domains.truncate(self.config.top_domains_count() as usize);

        // Get error statistics
        let error_stats = self.error_statistics.read().await;
        let top_errors = error_stats.top_errors(10);

        // Get queue metrics
        let queue_timing = self.queue_metrics.timing_statistics().await;

        MetricsSnapshot {
            session_id: self.config.session_id().clone(),
            uptime_secs: uptime.as_secs(),
            total_requests,
            successful_requests: successful,
            failed_requests: failed,
            success_rate,
            requests_per_second,
            total_bytes,
            bytes_per_second,
            timing_statistics: timing,
            top_domains,
            top_errors,
            queue_metrics: QueueMetricsSnapshot {
                tasks_enqueued: self.queue_metrics.tasks_enqueued(),
                tasks_completed: self.queue_metrics.tasks_completed(),
                tasks_failed: self.queue_metrics.tasks_failed(),
                tasks_retried: self.queue_metrics.tasks_retried(),
                completion_rate: self.queue_metrics.completion_rate(),
                timing_statistics: queue_timing,
            },
            config: self.config.clone(),
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) -> ProcessingResult<()> {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.total_bytes.store(0, Ordering::Relaxed);

        {
            let mut domain_stats = self.domain_metrics.write().await;
            domain_stats.clear();
        }

        {
            let mut error_stats = self.error_statistics.write().await;
            *error_stats = ErrorStatistics::new();
        }

        {
            let mut timing = self.timing_statistics.write().await;
            *timing = PerformanceMetrics::new();
        }

        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }

    /// Get basic statistics without full snapshot
    pub fn basic_stats(&self) -> BasicMetricsStats {
        BasicMetricsStats {
            uptime_secs: self.start_time.elapsed().as_secs(),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
        }
    }
}

impl Default for CrawlerMetrics {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Comprehensive metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    session_id: SessionId,
    uptime_secs: u64,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    success_rate: f64,
    requests_per_second: f64,
    total_bytes: u64,
    bytes_per_second: f64,
    timing_statistics: PerformanceMetrics,
    top_domains: Vec<(String, DomainMetrics)>,
    top_errors: Vec<(String, u64)>,
    queue_metrics: QueueMetricsSnapshot,
    config: MetricsConfig,
}

impl MetricsSnapshot {
    // Getters for all fields
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn uptime_secs(&self) -> u64 {
        self.uptime_secs
    }
    pub fn total_requests(&self) -> u64 {
        self.total_requests
    }
    pub fn successful_requests(&self) -> u64 {
        self.successful_requests
    }
    pub fn failed_requests(&self) -> u64 {
        self.failed_requests
    }
    pub fn success_rate(&self) -> f64 {
        self.success_rate
    }
    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
    pub fn bytes_per_second(&self) -> f64 {
        self.bytes_per_second
    }
    pub fn timing_statistics(&self) -> &PerformanceMetrics {
        &self.timing_statistics
    }
    pub fn top_domains(&self) -> &[(String, DomainMetrics)] {
        &self.top_domains
    }
    pub fn top_errors(&self) -> &[(String, u64)] {
        &self.top_errors
    }
    pub fn queue_metrics(&self) -> &QueueMetricsSnapshot {
        &self.queue_metrics
    }
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }
}

/// Queue metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetricsSnapshot {
    tasks_enqueued: u64,
    tasks_completed: u64,
    tasks_failed: u64,
    tasks_retried: u64,
    completion_rate: f64,
    timing_statistics: PerformanceMetrics,
}

impl QueueMetricsSnapshot {
    pub fn tasks_enqueued(&self) -> u64 {
        self.tasks_enqueued
    }
    pub fn tasks_completed(&self) -> u64 {
        self.tasks_completed
    }
    pub fn tasks_failed(&self) -> u64 {
        self.tasks_failed
    }
    pub fn tasks_retried(&self) -> u64 {
        self.tasks_retried
    }
    pub fn completion_rate(&self) -> f64 {
        self.completion_rate
    }
    pub fn timing_statistics(&self) -> &PerformanceMetrics {
        &self.timing_statistics
    }
}

/// Basic metrics for lightweight access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMetricsStats {
    uptime_secs: u64,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_bytes: u64,
}

impl BasicMetricsStats {
    pub fn uptime_secs(&self) -> u64 {
        self.uptime_secs
    }
    pub fn total_requests(&self) -> u64 {
        self.total_requests
    }
    pub fn successful_requests(&self) -> u64 {
        self.successful_requests
    }
    pub fn failed_requests(&self) -> u64 {
        self.failed_requests
    }
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_config_builder() {
        let config = MetricsConfig::builder()
            .with_collection_settings(
                BooleanFlag::enabled(),
                BooleanFlag::enabled(),
                BooleanFlag::disabled(),
                BooleanFlag::enabled(),
            )
            .with_retention_settings(
                LimitValue::new(500),
                LimitValue::new(100),
                LimitValue::new(12),
            )
            .build();

        assert!(config.should_collect_domain_stats());
        assert!(config.should_collect_timing_stats());
        assert!(!config.should_collect_error_stats());
        assert!(config.should_collect_queue_stats());
        assert_eq!(config.max_domain_entries(), 500);
    }

    #[tokio::test]
    async fn test_domain_metrics() {
        let mut metrics = DomainMetrics::new();

        metrics.record_success(Duration::from_millis(100), 1024);
        metrics.record_failure(Duration::from_millis(200), Some("timeout"));

        assert_eq!(metrics.requests_count(), 2);
        assert_eq!(metrics.success_count(), 1);
        assert_eq!(metrics.failure_count(), 1);
        assert_eq!(metrics.total_bytes(), 1024);
        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[tokio::test]
    async fn test_crawler_metrics() {
        let config = MetricsConfig::default();
        let metrics = CrawlerMetrics::new(config);

        let url = UrlString::new("https://example.com".to_string());
        let duration = Duration::from_millis(100);

        metrics.record_success(&url, duration, 1024).await.unwrap();

        let stats = metrics.basic_stats();
        assert_eq!(stats.total_requests(), 1);
        assert_eq!(stats.successful_requests(), 1);
        assert_eq!(stats.total_bytes(), 1024);
    }

    #[tokio::test]
    async fn test_queue_metrics() {
        let queue_metrics = QueueMetrics::new();

        queue_metrics.record_enqueued();
        queue_metrics.record_completed();
        queue_metrics
            .record_processing_time(Duration::from_millis(50))
            .await;

        assert_eq!(queue_metrics.tasks_enqueued(), 1);
        assert_eq!(queue_metrics.tasks_completed(), 1);
        assert_eq!(queue_metrics.completion_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_error_statistics() {
        let mut error_stats = ErrorStatistics::new();

        error_stats.record_error("timeout");
        error_stats.record_error("connection_refused");
        error_stats.record_error("timeout");

        assert_eq!(error_stats.total_errors(), 3);

        let top_errors = error_stats.top_errors(5);
        assert_eq!(top_errors.len(), 2);
        assert_eq!(top_errors[0], ("timeout".to_string(), 2));
    }
}
