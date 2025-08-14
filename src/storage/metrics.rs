use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Comprehensive metrics collection for the crawler
#[derive(Debug)]
pub struct CrawlerMetrics {
    // Request metrics
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    retried_requests: AtomicU64,

    // Performance metrics
    total_processing_time: AtomicU64, // milliseconds
    total_bytes_downloaded: AtomicU64,

    // Queue metrics
    tasks_enqueued: AtomicU64,
    tasks_completed: AtomicU64,
    tasks_failed: AtomicU64,

    // Domain-specific metrics
    domain_stats: Arc<RwLock<HashMap<String, DomainMetrics>>>,

    // System metrics
    start_time: Instant,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainMetrics {
    pub requests_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_response_time_ms: f64,
    pub total_bytes: u64,
    #[serde(skip)]
    pub last_request_time: Option<Instant>,
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub requests_per_second: f64,
    pub bytes_per_second: f64,
    pub queue_metrics: QueueMetricsSnapshot,
    pub top_domains: Vec<(String, DomainMetrics)>,
}

#[derive(Debug, Serialize)]
pub struct QueueMetricsSnapshot {
    pub tasks_enqueued: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub completion_rate: f64,
}

impl CrawlerMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            retried_requests: AtomicU64::new(0),
            total_processing_time: AtomicU64::new(0),
            total_bytes_downloaded: AtomicU64::new(0),
            tasks_enqueued: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            domain_stats: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a successful request
    pub async fn record_success(&self, domain: &str, response_time: Duration, bytes: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time
            .fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);
        self.total_bytes_downloaded
            .fetch_add(bytes, Ordering::Relaxed);

        self.update_domain_stats(domain, true, response_time, bytes)
            .await;
    }

    /// Record a failed request
    pub async fn record_failure(&self, domain: &str, response_time: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time
            .fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);

        self.update_domain_stats(domain, false, response_time, 0)
            .await;
    }

    /// Record a retry
    pub fn record_retry(&self) {
        self.retried_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record queue operations
    pub fn record_task_enqueued(&self) {
        self.tasks_enqueued.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_task_completed(&self) {
        self.tasks_completed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_task_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
    }

    /// Update domain-specific statistics
    async fn update_domain_stats(
        &self,
        domain: &str,
        success: bool,
        response_time: Duration,
        bytes: u64,
    ) {
        let mut stats = self.domain_stats.write().await;
        let domain_metrics = stats.entry(domain.to_string()).or_insert(DomainMetrics {
            requests_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_response_time_ms: 0.0,
            total_bytes: 0,
            last_request_time: None,
        });

        domain_metrics.requests_count += 1;
        if success {
            domain_metrics.success_count += 1;
        } else {
            domain_metrics.failure_count += 1;
        }

        // Update average response time
        let new_time_ms = response_time.as_millis() as f64;
        domain_metrics.avg_response_time_ms = (domain_metrics.avg_response_time_ms
            * (domain_metrics.requests_count - 1) as f64
            + new_time_ms)
            / domain_metrics.requests_count as f64;

        domain_metrics.total_bytes += bytes;
        domain_metrics.last_request_time = Some(Instant::now());
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let uptime = self.start_time.elapsed();
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let total_time_ms = self.total_processing_time.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes_downloaded.load(Ordering::Relaxed);

        let success_rate = if total_requests > 0 {
            (successful as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let avg_response_time = if total_requests > 0 {
            total_time_ms as f64 / total_requests as f64
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

        // Get top domains by request count
        let domain_stats = self.domain_stats.read().await;
        let mut top_domains: Vec<_> = domain_stats
            .iter()
            .map(|(domain, metrics)| (domain.clone(), metrics.clone()))
            .collect();
        top_domains.sort_by(|a, b| b.1.requests_count.cmp(&a.1.requests_count));
        top_domains.truncate(10); // Top 10 domains

        MetricsSnapshot {
            uptime_secs: uptime.as_secs(),
            total_requests,
            success_rate,
            avg_response_time_ms: avg_response_time,
            requests_per_second,
            bytes_per_second,
            queue_metrics: QueueMetricsSnapshot {
                tasks_enqueued: self.tasks_enqueued.load(Ordering::Relaxed),
                tasks_completed: self.tasks_completed.load(Ordering::Relaxed),
                tasks_failed: self.tasks_failed.load(Ordering::Relaxed),
                completion_rate: {
                    let enqueued = self.tasks_enqueued.load(Ordering::Relaxed);
                    let completed = self.tasks_completed.load(Ordering::Relaxed);
                    if enqueued > 0 {
                        (completed as f64 / enqueued as f64) * 100.0
                    } else {
                        0.0
                    }
                },
            },
            top_domains,
        }
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.retried_requests.store(0, Ordering::Relaxed);
        self.total_processing_time.store(0, Ordering::Relaxed);
        self.total_bytes_downloaded.store(0, Ordering::Relaxed);
        self.tasks_enqueued.store(0, Ordering::Relaxed);
        self.tasks_completed.store(0, Ordering::Relaxed);
        self.tasks_failed.store(0, Ordering::Relaxed);

        let mut domain_stats = self.domain_stats.write().await;
        domain_stats.clear();
    }
}

impl Default for CrawlerMetrics {
    fn default() -> Self {
        Self::new()
    }
}
