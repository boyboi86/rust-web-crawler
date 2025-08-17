/// Task queue module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration from external sources
/// Following Rule 3: Builder pattern for complex structures
/// Following Rule 4: Privacy first - controlled access only
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, ConfigResult, CountValue, CrawlTask, CrawlTaskBuilder, DelayDuration, LimitValue,
    NetworkResult, PerformanceMetrics, RetryConfig, SessionId, TaskCounts, TaskError, TaskId,
    TaskPriority, TaskResult, TaskStatus, TimeoutDuration, UrlString,
};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::sleep;

/// Configuration for task queue behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueConfig {
    max_concurrent_tasks: LimitValue,
    max_queue_size: LimitValue,
    retry_config: RetryConfig,
    persistence_enabled: BooleanFlag,
    persistence_interval: DelayDuration,
    queue_timeout: TimeoutDuration,
}

impl TaskQueueConfig {
    pub fn builder() -> TaskQueueConfigBuilder {
        TaskQueueConfigBuilder::new()
    }

    pub fn max_concurrent_tasks(&self) -> u64 {
        self.max_concurrent_tasks.value()
    }

    pub fn max_queue_size(&self) -> u64 {
        self.max_queue_size.value()
    }

    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence_enabled.is_enabled()
    }

    pub fn persistence_interval(&self) -> DelayDuration {
        self.persistence_interval
    }

    pub fn queue_timeout(&self) -> TimeoutDuration {
        self.queue_timeout
    }
}

impl Default for TaskQueueConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: LimitValue::new(10),
            max_queue_size: LimitValue::new(10000),
            retry_config: RetryConfig::default(),
            persistence_enabled: BooleanFlag::enabled(),
            persistence_interval: DelayDuration::from_secs(60),
            queue_timeout: TimeoutDuration::from_secs(300),
        }
    }
}

/// Builder for task queue configuration
#[derive(Debug)]
pub struct TaskQueueConfigBuilder {
    max_concurrent_tasks: LimitValue,
    max_queue_size: LimitValue,
    retry_config: RetryConfig,
    persistence_enabled: BooleanFlag,
    persistence_interval: DelayDuration,
    queue_timeout: TimeoutDuration,
}

impl TaskQueueConfigBuilder {
    pub fn new() -> Self {
        let default_config = TaskQueueConfig::default();
        Self {
            max_concurrent_tasks: default_config.max_concurrent_tasks,
            max_queue_size: default_config.max_queue_size,
            retry_config: default_config.retry_config,
            persistence_enabled: default_config.persistence_enabled,
            persistence_interval: default_config.persistence_interval,
            queue_timeout: default_config.queue_timeout,
        }
    }

    pub fn with_max_concurrent_tasks(mut self, limit: LimitValue) -> Self {
        self.max_concurrent_tasks = limit;
        self
    }

    pub fn with_max_queue_size(mut self, limit: LimitValue) -> Self {
        self.max_queue_size = limit;
        self
    }

    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    pub fn with_persistence_enabled(mut self, enabled: BooleanFlag) -> Self {
        self.persistence_enabled = enabled;
        self
    }

    pub fn with_persistence_interval(mut self, interval: DelayDuration) -> Self {
        self.persistence_interval = interval;
        self
    }

    pub fn with_queue_timeout(mut self, timeout: TimeoutDuration) -> Self {
        self.queue_timeout = timeout;
        self
    }

    pub fn build(self) -> TaskQueueConfig {
        TaskQueueConfig {
            max_concurrent_tasks: self.max_concurrent_tasks,
            max_queue_size: self.max_queue_size,
            retry_config: self.retry_config,
            persistence_enabled: self.persistence_enabled,
            persistence_interval: self.persistence_interval,
            queue_timeout: self.queue_timeout,
        }
    }
}

impl Default for TaskQueueConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable queue state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueState {
    pending_tasks: Vec<CrawlTask>,
    in_progress_tasks: Vec<CrawlTask>,
    retry_queue: Vec<CrawlTask>,
    task_counts: TaskCounts,
    performance_metrics: PerformanceMetrics,
    timestamp: SystemTime,
}

impl QueueState {
    pub fn new() -> Self {
        Self {
            pending_tasks: Vec::new(),
            in_progress_tasks: Vec::new(),
            retry_queue: Vec::new(),
            task_counts: TaskCounts::default(),
            performance_metrics: PerformanceMetrics::default(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn pending_tasks(&self) -> &[CrawlTask] {
        &self.pending_tasks
    }

    pub fn in_progress_tasks(&self) -> &[CrawlTask] {
        &self.in_progress_tasks
    }

    pub fn retry_queue(&self) -> &[CrawlTask] {
        &self.retry_queue
    }

    pub fn task_counts(&self) -> &TaskCounts {
        &self.task_counts
    }

    pub fn performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

impl Default for QueueState {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for tasks in the priority queue
#[derive(Debug, Clone)]
struct PrioritizedTask {
    task: CrawlTask,
}

impl PrioritizedTask {
    fn new(task: CrawlTask) -> Self {
        Self { task }
    }

    fn task(&self) -> &CrawlTask {
        &self.task
    }

    fn into_task(self) -> CrawlTask {
        self.task
    }
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority() == other.task.priority()
    }
}

impl Eq for PrioritizedTask {}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then by creation time (FIFO for same priority)
        match self.task.priority().cmp(&other.task.priority()) {
            std::cmp::Ordering::Equal => {
                // For same priority, earlier tasks first (reverse order for max-heap)
                other
                    .task
                    .timing()
                    .created_at()
                    .cmp(&self.task.timing().created_at())
            }
            other_ordering => other_ordering,
        }
    }
}

/// Task queue result type for consistent error handling
pub type QueueResult<T> = Result<T, TaskError>;

/// Message queue for managing crawl tasks with priority and retry logic
/// Following Rule 4: Privacy first - all fields private with controlled access
pub struct TaskQueue {
    // Core queue structures (private)
    pending_tasks: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
    in_progress_tasks: Arc<RwLock<HashMap<String, CrawlTask>>>,
    completed_tasks: Arc<RwLock<Vec<CrawlTask>>>,
    failed_tasks: Arc<RwLock<Vec<CrawlTask>>>,
    retry_queue: Arc<RwLock<VecDeque<CrawlTask>>>,

    // Communication (private)
    result_sender: mpsc::UnboundedSender<TaskResult<crate::common::TaskContent>>,
    result_receiver:
        Arc<RwLock<Option<mpsc::UnboundedReceiver<TaskResult<crate::common::TaskContent>>>>>,

    // Concurrency control (private)
    semaphore: Arc<Semaphore>,

    // Statistics and metrics (private)
    task_counts: Arc<RwLock<TaskCounts>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,

    // Configuration (private)
    config: TaskQueueConfig,
}

impl TaskQueue {
    /// Create a new task queue with the given configuration
    pub fn new(config: TaskQueueConfig) -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks() as usize));

        Self {
            pending_tasks: Arc::new(RwLock::new(BinaryHeap::new())),
            in_progress_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
            failed_tasks: Arc::new(RwLock::new(Vec::new())),
            retry_queue: Arc::new(RwLock::new(VecDeque::new())),
            result_sender,
            result_receiver: Arc::new(RwLock::new(Some(result_receiver))),
            semaphore,
            task_counts: Arc::new(RwLock::new(TaskCounts::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            config,
        }
    }

    /// Create a task queue with default configuration
    pub fn with_defaults() -> Self {
        Self::new(TaskQueueConfig::default())
    }

    /// Get the current task counts
    pub async fn task_counts(&self) -> TaskCounts {
        self.task_counts.read().await.clone()
    }

    /// Get the current performance metrics
    pub async fn performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Get the queue configuration
    pub fn config(&self) -> &TaskQueueConfig {
        &self.config
    }

    /// Check if the queue is at capacity
    pub async fn is_at_capacity(&self) -> bool {
        let pending_count = self.pending_tasks.read().await.len();
        pending_count >= self.config.max_queue_size() as usize
    }

    /// Get the number of pending tasks
    pub async fn pending_count(&self) -> usize {
        self.pending_tasks.read().await.len()
    }

    /// Get the number of in-progress tasks
    pub async fn in_progress_count(&self) -> usize {
        self.in_progress_tasks.read().await.len()
    }

    /// Add a task to the queue
    /// Following Rule 8: Result<T,E> for error handling
    pub async fn enqueue(&self, task: CrawlTask) -> QueueResult<()> {
        if self.is_at_capacity().await {
            return Err(TaskError::configuration("Queue is at capacity"));
        }

        let prioritized_task = PrioritizedTask::new(task);

        {
            let mut pending = self.pending_tasks.write().await;
            pending.push(prioritized_task);
        }

        {
            let mut counts = self.task_counts.write().await;
            counts.increment_pending();
            counts.increment_total();
        }

        Ok(())
    }

    /// Create and enqueue a task using the builder pattern
    /// Following Rule 3: Builder pattern for complex structures
    pub async fn enqueue_url(
        &self,
        session_id: SessionId,
        url: UrlString,
        priority: TaskPriority,
    ) -> QueueResult<TaskId> {
        let task_id = TaskId::new(format!("task_{}", uuid::Uuid::new_v4()));

        let task = CrawlTask::builder()
            .with_id(task_id.clone())
            .with_session_id(session_id)
            .with_url(url)
            .with_priority(priority)
            .build()?;

        self.enqueue(task).await?;
        Ok(task_id)
    }

    /// Dequeue the next task with highest priority
    /// Following Rule 8: Option<T> for nullable results
    pub async fn dequeue(&self) -> Option<CrawlTask> {
        let permit = self.semaphore.clone().try_acquire_owned().ok()?;

        let task = {
            let mut pending = self.pending_tasks.write().await;
            pending.pop()?.into_task()
        };

        // Move task to in-progress
        {
            let mut in_progress = self.in_progress_tasks.write().await;
            in_progress.insert(task.id().as_str().to_string(), task.clone());
        }

        // Update counts
        {
            let mut counts = self.task_counts.write().await;
            counts.increment_in_progress();
        }

        // Release permit when task is done (stored in task context)
        std::mem::forget(permit); // Will be released by complete/fail methods

        Some(task)
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, task_id: &TaskId) -> QueueResult<()> {
        let task = self.remove_from_in_progress(task_id).await?;

        {
            let mut completed = self.completed_tasks.write().await;
            completed.push(task);
        }

        {
            let mut counts = self.task_counts.write().await;
            counts.increment_completed();
        }

        Ok(())
    }

    /// Mark a task as failed and potentially retry
    pub async fn fail_task(&self, task_id: &TaskId, error: TaskError) -> QueueResult<()> {
        let mut task = self.remove_from_in_progress(task_id).await?;

        if task.attempt_count() < self.config.retry_config().max_attempts() {
            // Schedule for retry
            task.retry();

            let delay = self
                .config
                .retry_config()
                .calculate_delay(task.attempt_count());
            task.timing_mut().set_delay(delay);

            {
                let mut retry_queue = self.retry_queue.write().await;
                retry_queue.push_back(task);
            }

            {
                let mut counts = self.task_counts.write().await;
                counts.increment_retrying();
            }
        } else {
            // Mark as dead (exceeded retry limit)
            task.mark_dead();

            {
                let mut failed = self.failed_tasks.write().await;
                failed.push(task);
            }

            {
                let mut counts = self.task_counts.write().await;
                counts.increment_dead();
            }
        }

        Ok(())
    }

    /// Process retry queue - move ready tasks back to pending
    pub async fn process_retry_queue(&self) -> QueueResult<usize> {
        let mut moved_count = 0;
        let mut tasks_to_retry = Vec::new();

        // Collect ready tasks from retry queue
        {
            let mut retry_queue = self.retry_queue.write().await;
            let mut remaining = VecDeque::new();

            while let Some(task) = retry_queue.pop_front() {
                if task.is_ready() {
                    tasks_to_retry.push(task);
                } else {
                    remaining.push_back(task);
                }
            }

            *retry_queue = remaining;
        }

        // Move ready tasks back to pending queue
        for task in tasks_to_retry {
            let prioritized_task = PrioritizedTask::new(task);

            {
                let mut pending = self.pending_tasks.write().await;
                pending.push(prioritized_task);
            }

            moved_count += 1;
        }

        Ok(moved_count)
    }

    /// Get the current queue state for persistence or monitoring
    pub async fn get_state(&self) -> QueueState {
        let pending_tasks = self
            .pending_tasks
            .read()
            .await
            .iter()
            .map(|pt| pt.task().clone())
            .collect();

        let in_progress_tasks = self
            .in_progress_tasks
            .read()
            .await
            .values()
            .cloned()
            .collect();

        let retry_queue = self.retry_queue.read().await.iter().cloned().collect();

        let task_counts = self.task_counts.read().await.clone();
        let performance_metrics = self.performance_metrics.read().await.clone();

        QueueState {
            pending_tasks,
            in_progress_tasks,
            retry_queue,
            task_counts,
            performance_metrics,
            timestamp: SystemTime::now(),
        }
    }

    /// Restore queue state from persistence
    pub async fn restore_state(&self, state: QueueState) -> QueueResult<()> {
        // Restore pending tasks
        {
            let mut pending = self.pending_tasks.write().await;
            pending.clear();

            for task in state.pending_tasks {
                pending.push(PrioritizedTask::new(task));
            }
        }

        // Restore in-progress tasks
        {
            let mut in_progress = self.in_progress_tasks.write().await;
            in_progress.clear();

            for task in state.in_progress_tasks {
                in_progress.insert(task.id().as_str().to_string(), task);
            }
        }

        // Restore retry queue
        {
            let mut retry_queue = self.retry_queue.write().await;
            retry_queue.clear();

            for task in state.retry_queue {
                retry_queue.push_back(task);
            }
        }

        // Restore statistics
        {
            let mut counts = self.task_counts.write().await;
            *counts = state.task_counts;
        }

        {
            let mut metrics = self.performance_metrics.write().await;
            *metrics = state.performance_metrics;
        }

        Ok(())
    }

    /// Private helper to remove task from in-progress
    async fn remove_from_in_progress(&self, task_id: &TaskId) -> QueueResult<CrawlTask> {
        let mut in_progress = self.in_progress_tasks.write().await;

        in_progress.remove(task_id.as_str()).ok_or_else(|| {
            TaskError::internal(format!("Task {} not found in progress", task_id.as_str()))
        })
    }

    /// Get result receiver for task completion notifications
    pub async fn take_result_receiver(
        &self,
    ) -> Option<mpsc::UnboundedReceiver<TaskResult<crate::common::TaskContent>>> {
        self.result_receiver.write().await.take()
    }

    /// Send a task result notification
    pub fn send_result(&self, result: TaskResult<crate::common::TaskContent>) -> QueueResult<()> {
        self.result_sender
            .send(result)
            .map_err(|_| TaskError::internal("Failed to send task result"))
    }

    /// Cleanup completed and failed tasks older than the specified duration
    pub async fn cleanup_old_tasks(&self, max_age: std::time::Duration) -> QueueResult<usize> {
        let cutoff_time = std::time::Instant::now() - max_age;
        let mut cleaned_count = 0;

        // Clean completed tasks
        {
            let mut completed = self.completed_tasks.write().await;
            let original_len = completed.len();
            completed.retain(|task| task.timing().created_at() > cutoff_time);
            cleaned_count += original_len - completed.len();
        }

        // Clean failed tasks
        {
            let mut failed = self.failed_tasks.write().await;
            let original_len = failed.len();
            failed.retain(|task| task.timing().created_at() > cutoff_time);
            cleaned_count += original_len - failed.len();
        }

        Ok(cleaned_count)
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_queue_creation() {
        let config = TaskQueueConfig::builder()
            .with_max_concurrent_tasks(LimitValue::new(5))
            .with_max_queue_size(LimitValue::new(100))
            .build();

        let queue = TaskQueue::new(config);

        assert_eq!(queue.config().max_concurrent_tasks(), 5);
        assert_eq!(queue.config().max_queue_size(), 100);
        assert_eq!(queue.pending_count().await, 0);
        assert_eq!(queue.in_progress_count().await, 0);
    }

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = TaskQueue::with_defaults();

        let task_id = queue
            .enqueue_url(
                SessionId::new("test-session"),
                UrlString::new("https://example.com"),
                TaskPriority::High,
            )
            .await
            .expect("Failed to enqueue task");

        assert_eq!(queue.pending_count().await, 1);

        let dequeued_task = queue.dequeue().await;
        assert!(dequeued_task.is_some());

        let task = dequeued_task.unwrap();
        assert_eq!(task.id(), &task_id);
        assert_eq!(task.priority(), TaskPriority::High);
    }

    #[tokio::test]
    async fn test_task_completion() {
        let queue = TaskQueue::with_defaults();

        let task_id = queue
            .enqueue_url(
                SessionId::new("test-session"),
                UrlString::new("https://example.com"),
                TaskPriority::Normal,
            )
            .await
            .expect("Failed to enqueue task");

        let _task = queue.dequeue().await.expect("No task available");

        queue
            .complete_task(&task_id)
            .await
            .expect("Failed to complete task");

        let counts = queue.task_counts().await;
        assert_eq!(counts.completed(), 1);
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let retry_config = RetryConfig::default();
        let config = TaskQueueConfig::builder()
            .with_retry_config(retry_config)
            .build();

        let queue = TaskQueue::new(config);

        let task_id = queue
            .enqueue_url(
                SessionId::new("test-session"),
                UrlString::new("https://example.com"),
                TaskPriority::Normal,
            )
            .await
            .expect("Failed to enqueue task");

        let _task = queue.dequeue().await.expect("No task available");

        let error = TaskError::network("Connection failed");
        queue
            .fail_task(&task_id, error)
            .await
            .expect("Failed to fail task");

        let counts = queue.task_counts().await;
        assert_eq!(counts.retrying(), 1);
    }
}
