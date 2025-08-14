use crate::core::types::TaskContent;
use crate::core::{CrawlTask, QueueStats, TaskPriority, TaskResult, TaskStatus};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::sleep;
use tracing::{debug, error, info};
use url::Url;

/// Serializable queue state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueState {
    pub pending_tasks: Vec<CrawlTask>,
    pub in_progress_tasks: Vec<CrawlTask>,
    pub retry_queue: Vec<CrawlTask>,
    pub stats: QueueStats,
    pub timestamp: std::time::SystemTime,
}

/// Message queue for managing crawl tasks with priority and retry logic
pub struct TaskQueue {
    // Priority queue for pending tasks (BinaryHeap is max-heap, so higher priority first)
    pending_tasks: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,

    // Tasks currently being processed
    in_progress_tasks: Arc<RwLock<HashMap<String, CrawlTask>>>,

    // Completed and failed tasks for statistics
    completed_tasks: Arc<RwLock<Vec<CrawlTask>>>,
    failed_tasks: Arc<RwLock<Vec<CrawlTask>>>,

    // Tasks waiting for retry
    retry_queue: Arc<RwLock<VecDeque<CrawlTask>>>,

    // Channel for task results
    result_sender: mpsc::UnboundedSender<TaskResult>,
    pub result_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TaskResult>>>>,

    // Concurrency control
    semaphore: Arc<Semaphore>,

    // Statistics
    stats: Arc<RwLock<QueueStats>>,

    // Configuration
    max_retries: u32,
    base_retry_delay: Duration,
    max_retry_delay: Duration,
    backoff_multiplier: f64,
}

/// Wrapper for tasks in the priority queue
#[derive(Debug, Clone)]
struct PrioritizedTask {
    task: CrawlTask,
}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.task.priority == other.task.priority
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
        self.task
            .priority
            .cmp(&other.task.priority)
            .then_with(|| other.task.created_at().cmp(&self.task.created_at()))
    }
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(max_concurrent_tasks: usize, max_retries: u32) -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();

        Self {
            pending_tasks: Arc::new(RwLock::new(BinaryHeap::new())),
            in_progress_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
            failed_tasks: Arc::new(RwLock::new(Vec::new())),
            retry_queue: Arc::new(RwLock::new(VecDeque::new())),
            result_sender,
            result_receiver: Arc::new(RwLock::new(Some(result_receiver))),
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            stats: Arc::new(RwLock::new(QueueStats::default())),
            max_retries,
            base_retry_delay: Duration::from_millis(1000),
            max_retry_delay: Duration::from_millis(30000),
            backoff_multiplier: 2.0,
        }
    }

    /// Add a new task to the queue
    pub async fn enqueue_task(&self, url: Url, priority: TaskPriority) -> Result<String, Error> {
        let task = CrawlTask::new(url, priority, self.max_retries);
        let task_id = task.id.clone();

        let prioritized_task = PrioritizedTask { task };

        {
            let mut pending = self.pending_tasks.write().await;
            pending.push(prioritized_task);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.counts.total += 1;
            stats.counts.pending += 1;
        }

        Ok(task_id)
    }

    /// Add multiple tasks at once
    pub async fn enqueue_batch(
        &self,
        urls: Vec<(Url, TaskPriority)>,
    ) -> Result<Vec<String>, Error> {
        let mut task_ids = Vec::new();

        {
            let mut pending = self.pending_tasks.write().await;
            let mut stats = self.stats.write().await;

            for (url, priority) in urls {
                let task = CrawlTask::new(url, priority, self.max_retries);
                let task_id = task.id.clone();
                task_ids.push(task_id);

                let prioritized_task = PrioritizedTask { task };
                pending.push(prioritized_task);

                stats.counts.total += 1;
                stats.counts.pending += 1;
            }
        }

        Ok(task_ids)
    }

    /// Get the next task to process
    pub async fn dequeue_task(&self) -> Option<CrawlTask> {
        // First check retry queue for ready tasks
        {
            let mut retry_queue = self.retry_queue.write().await;
            if let Some(mut task) = retry_queue.pop_front() {
                if task.is_ready_for_retry() {
                    task.mark_in_progress();

                    // Move to in-progress
                    {
                        let mut in_progress = self.in_progress_tasks.write().await;
                        in_progress.insert(task.id.clone(), task.clone());
                    }

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.counts.retrying = stats.counts.retrying.saturating_sub(1);
                        stats.counts.in_progress += 1;
                    }

                    return Some(task);
                } else {
                    // Put back if not ready
                    retry_queue.push_back(task);
                }
            }
        }

        // Then check pending tasks
        {
            let mut pending = self.pending_tasks.write().await;
            if let Some(prioritized_task) = pending.pop() {
                let mut task = prioritized_task.task;
                task.mark_in_progress();

                // Move to in-progress
                {
                    let mut in_progress = self.in_progress_tasks.write().await;
                    in_progress.insert(task.id.clone(), task.clone());
                }

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.counts.pending = stats.counts.pending.saturating_sub(1);
                    stats.counts.in_progress += 1;
                }

                return Some(task);
            }
        }

        None
    }

    /// Mark a task as completed successfully
    pub async fn complete_task(
        &self,
        task_id: &str,
        content: Option<String>,
        processing_time: Duration,
    ) -> Result<(), Error> {
        let task = {
            let mut in_progress = self.in_progress_tasks.write().await;
            in_progress.remove(task_id)
        };

        if let Some(mut task) = task {
            task.mark_completed();

            // Send result
            let result = TaskResult {
                task_id: task_id.to_string(),
                url: task.url.clone(),
                success: true,
                content: content.map(|content_str| TaskContent {
                    content: content_str.clone(),
                    word_count: content_str.split_whitespace().count(),
                    detected_language: None, // Could implement language detection here
                }),
                error: None,
                processing_time,
            };

            let _ = self.result_sender.send(result);

            // Move to completed
            {
                let mut completed = self.completed_tasks.write().await;
                completed.push(task);
            }

            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.counts.in_progress = stats.counts.in_progress.saturating_sub(1);
                stats.counts.completed += 1;

                // Update average processing time
                let total_completed = stats.counts.completed as f64;
                let current_avg = stats.performance.average_processing_time_ms;
                let new_time = processing_time.as_millis() as f64;
                stats.performance.average_processing_time_ms =
                    (current_avg * (total_completed - 1.0) + new_time) / total_completed;

                // Update success rate
                let total_processed = stats.counts.completed + stats.counts.dead;
                if total_processed > 0 {
                    stats.performance.success_rate =
                        (stats.counts.completed as f64 / total_processed as f64) * 100.0;
                }
            }
        }

        Ok(())
    }

    /// Mark a task as failed and potentially retry
    pub async fn fail_task(
        &self,
        task_id: &str,
        error: String,
        processing_time: Duration,
    ) -> Result<(), Error> {
        let task = {
            let mut in_progress = self.in_progress_tasks.write().await;
            in_progress.remove(task_id)
        };

        if let Some(mut task) = task {
            // Calculate retry delay with exponential backoff
            let retry_delay = if task.can_retry() {
                let delay_ms = (self.base_retry_delay.as_millis() as f64
                    * self.backoff_multiplier.powi(task.attempt_count as i32))
                    as u64;
                let capped_delay =
                    Duration::from_millis(delay_ms.min(self.max_retry_delay.as_millis() as u64));
                Some(capped_delay)
            } else {
                None
            };

            task.mark_failed(error.clone(), retry_delay);

            // Send result
            let result = TaskResult {
                task_id: task_id.to_string(),
                url: task.url.clone(),
                success: false,
                content: None,
                error: Some(error),
                processing_time,
            };

            let _ = self.result_sender.send(result);

            if task.status == TaskStatus::Retrying {
                // Add to retry queue
                let mut retry_queue = self.retry_queue.write().await;
                retry_queue.push_back(task);

                // Update stats
                let mut stats = self.stats.write().await;
                stats.counts.in_progress = stats.counts.in_progress.saturating_sub(1);
                stats.counts.retrying += 1;
            } else {
                // Task is dead, move to failed
                let mut failed = self.failed_tasks.write().await;
                failed.push(task);

                // Update stats
                let mut stats = self.stats.write().await;
                stats.counts.in_progress = stats.counts.in_progress.saturating_sub(1);
                stats.counts.dead += 1;

                // Update success rate
                let total_processed = stats.counts.completed + stats.counts.dead;
                if total_processed > 0 {
                    stats.performance.success_rate =
                        (stats.counts.completed as f64 / total_processed as f64) * 100.0;
                }
            }
        }

        Ok(())
    }

    /// Get current queue statistics
    pub async fn get_stats(&self) -> QueueStats {
        self.stats.read().await.clone()
    }

    /// Get number of pending tasks
    pub async fn pending_count(&self) -> usize {
        self.pending_tasks.read().await.len()
    }

    /// Get number of in-progress tasks
    pub async fn in_progress_count(&self) -> usize {
        self.in_progress_tasks.read().await.len()
    }

    /// Get number of retry tasks ready to process
    pub async fn ready_retry_count(&self) -> usize {
        let retry_queue = self.retry_queue.read().await;
        retry_queue
            .iter()
            .filter(|task| task.is_ready_for_retry())
            .count()
    }

    /// Check if queue has any work available
    pub async fn has_work(&self) -> bool {
        let pending_count = self.pending_count().await;
        let ready_retries = self.ready_retry_count().await;
        pending_count > 0 || ready_retries > 0
    }

    /// Acquire a permit from the semaphore for concurrency control
    pub async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>, Error> {
        Ok(self.semaphore.acquire().await?)
    }
    /// Clean up old completed/failed tasks to prevent memory growth
    pub async fn cleanup_old_tasks(&self, max_history: usize) {
        {
            let mut completed = self.completed_tasks.write().await;
            if completed.len() > max_history {
                let len = completed.len();
                completed.drain(0..len - max_history);
            }
        }

        {
            let mut failed = self.failed_tasks.write().await;
            if failed.len() > max_history {
                let len = failed.len();
                failed.drain(0..len - max_history);
            }
        }
    }

    /// Process retry queue to move ready tasks back to pending
    pub async fn process_retry_queue(&self) {
        let mut ready_tasks = Vec::new();

        {
            let mut retry_queue = self.retry_queue.write().await;
            let mut remaining_tasks = VecDeque::new();

            while let Some(task) = retry_queue.pop_front() {
                if task.is_ready_for_retry() {
                    ready_tasks.push(task);
                } else {
                    remaining_tasks.push_back(task);
                }
            }

            *retry_queue = remaining_tasks;
        }

        if !ready_tasks.is_empty() {
            let mut pending = self.pending_tasks.write().await;
            let mut stats = self.stats.write().await;

            for task in ready_tasks {
                let prioritized_task = PrioritizedTask { task };
                pending.push(prioritized_task);

                stats.counts.retrying = stats.counts.retrying.saturating_sub(1);
                stats.counts.pending += 1;
            }
        }
    }

    /// Check for and recover zombie tasks (tasks that have been in-progress too long)
    pub async fn check_for_zombie_tasks(&self, timeout_duration: Duration) {
        let now = Instant::now();
        let mut zombie_task_ids = Vec::new();

        // Find zombie tasks
        {
            let in_progress = self.in_progress_tasks.read().await;
            for (task_id, task) in in_progress.iter() {
                if let Some(started_at) = task.started_at()
                    && now.duration_since(started_at) > timeout_duration
                {
                    zombie_task_ids.push(task_id.clone());
                }
            }
        }

        // Fail zombie tasks
        for task_id in zombie_task_ids {
            tracing::warn!(
                task_id = %task_id,
                timeout_duration = ?timeout_duration,
                "Detected zombie task (timeout exceeded)"
            );
            if let Err(e) = self
                .fail_task(
                    &task_id,
                    "Task timeout - possible network hang or infinite loop".to_string(),
                    timeout_duration,
                )
                .await
            {
                error!("Error failing zombie task {}: {}", task_id, e);
            }
        }
    }

    /// Get tasks that have been in-progress for longer than the specified duration
    pub async fn get_long_running_tasks(&self, threshold: Duration) -> Vec<(String, Duration)> {
        let now = Instant::now();
        let mut long_running = Vec::new();

        let in_progress = self.in_progress_tasks.read().await;
        for (task_id, task) in in_progress.iter() {
            if let Some(started_at) = task.started_at() {
                let duration = now.duration_since(started_at);
                if duration > threshold {
                    long_running.push((task_id.clone(), duration));
                }
            }
        }

        long_running
    }

    /// Save queue state to file for crash recovery
    pub async fn save_state<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let pending: Vec<CrawlTask> = self
            .pending_tasks
            .read()
            .await
            .iter()
            .map(|pt| pt.task.clone())
            .collect();

        let in_progress: Vec<CrawlTask> = self
            .in_progress_tasks
            .read()
            .await
            .values()
            .cloned()
            .collect();

        let retry_queue: Vec<CrawlTask> = self.retry_queue.read().await.iter().cloned().collect();

        let stats = self.stats.read().await.clone();

        let state = QueueState {
            pending_tasks: pending,
            in_progress_tasks: in_progress,
            retry_queue,
            stats,
            timestamp: std::time::SystemTime::now(),
        };

        let json = serde_json::to_string_pretty(&state)?;
        fs::write(path, json).await?;

        Ok(())
    }

    /// Load queue state from file for crash recovery
    pub async fn load_state<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let content = fs::read_to_string(path).await?;
        let state: QueueState = serde_json::from_str(&content)?;

        // Restore pending tasks
        {
            let mut pending = self.pending_tasks.write().await;
            pending.clear();
            for task in state.pending_tasks {
                pending.push(PrioritizedTask { task });
            }
        }

        // Restore retry queue
        {
            let mut retry = self.retry_queue.write().await;
            retry.clear();
            for task in state.retry_queue {
                retry.push_back(task);
            }
        }

        // Move in-progress tasks back to pending (they were interrupted)
        {
            let mut pending = self.pending_tasks.write().await;
            for mut task in state.in_progress_tasks {
                task.status = TaskStatus::Pending;
                task.attempt_count = 0; // Reset attempt count for interrupted tasks
                pending.push(PrioritizedTask { task });
            }
        }

        // Restore stats (but reset current counts as they may be stale)
        {
            let mut stats = self.stats.write().await;
            *stats = state.stats;
            let pending_count = self.pending_tasks.read().await.len() as u64;
            stats.counts.pending = pending_count;
            stats.counts.in_progress = 0; // No tasks in progress after recovery
        }

        info!("Queue state restored from checkpoint");
        Ok(())
    }

    /// Create a new TaskQueue with persistence support
    pub fn with_persistence<P: AsRef<Path>>(max_concurrent: usize, _checkpoint_path: P) -> Self {
        Self::new(max_concurrent, 3) // Default 3 retries
    }

    /// Start automatic checkpointing
    pub fn start_checkpointing<P: AsRef<Path> + Send + Sync + 'static>(
        queue: Arc<TaskQueue>,
        path: P,
        interval: Duration,
    ) {
        let checkpoint_path = path.as_ref().to_path_buf();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;

                if let Err(e) = queue.save_state(&checkpoint_path).await {
                    error!("Failed to save queue checkpoint: {}", e);
                } else {
                    debug!("Queue checkpoint saved");
                }
            }
        });
    }
}

/// Background task processor that continuously processes the queue
pub async fn run_queue_processor<F, Fut>(
    queue: Arc<TaskQueue>,
    processor_fn: F,
    cleanup_interval: Duration,
) -> Result<(), Error>
where
    F: Fn(CrawlTask) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<Option<String>, Error>> + Send + 'static,
{
    let queue_clone = Arc::clone(&queue);

    // Spawn cleanup task
    tokio::spawn(async move {
        let mut cleanup_interval = tokio::time::interval(cleanup_interval);
        loop {
            cleanup_interval.tick().await;
            queue_clone.cleanup_old_tasks(1000).await;
            queue_clone.process_retry_queue().await;

            // Check for zombie tasks (tasks stuck in progress for too long)
            let zombie_timeout = Duration::from_secs(300); // 5 minutes timeout
            queue_clone.check_for_zombie_tasks(zombie_timeout).await;
        }
    });

    // Main processing loop
    loop {
        if let Some(task) = queue.dequeue_task().await {
            let _permit = queue.acquire_permit().await?;
            let queue_clone = Arc::clone(&queue);
            let task_id = task.id.clone();
            let processor_fn_clone = processor_fn.clone();

            tokio::spawn(async move {
                let start_time = Instant::now();

                match processor_fn_clone(task.clone()).await {
                    Ok(content) => {
                        let processing_time = start_time.elapsed();
                        if let Err(e) = queue_clone
                            .complete_task(&task_id, content, processing_time)
                            .await
                        {
                            tracing::error!(
                                task_id = %task_id,
                                error = %e,
                                "Error completing task"
                            );
                        }
                    }
                    Err(error) => {
                        let processing_time = start_time.elapsed();
                        if let Err(e) = queue_clone
                            .fail_task(&task_id, error.to_string(), processing_time)
                            .await
                        {
                            tracing::error!(
                                task_id = %task_id,
                                error = %e,
                                "Error failing task"
                            );
                        }
                    }
                }
            });
        } else {
            // No work available, sleep briefly
            sleep(Duration::from_millis(100)).await;
        }
    }
}
