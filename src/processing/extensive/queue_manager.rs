/// Queue management for extensive crawling
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use super::config::ExtensiveConfig;
use super::link_processor::ProcessedLink;
use crate::core::error::CrawlError;
use crate::core::types::CrawlTask;

/// Status of the extensive crawling queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// Total URLs in queue
    pub total_queued: usize,
    /// URLs processed
    pub processed: usize,
    /// URLs failed
    pub failed: usize,
    /// Current queue depth distribution
    pub depth_distribution: HashMap<usize, usize>,
    /// Queue capacity utilization
    pub capacity_used: f64,
}

/// Statistics about link discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryStats {
    /// Total links discovered
    pub total_discovered: usize,
    /// Links added to queue
    pub queued: usize,
    /// Links filtered out
    pub filtered: usize,
    /// Links by category
    pub category_distribution: HashMap<String, usize>,
    /// Average priority score
    pub average_priority: f64,
    /// Discovery rate (links per page)
    pub discovery_rate: f64,
}

impl Default for DiscoveryStats {
    fn default() -> Self {
        Self {
            total_discovered: 0,
            queued: 0,
            filtered: 0,
            category_distribution: HashMap::new(),
            average_priority: 0.0,
            discovery_rate: 0.0,
        }
    }
}

/// Extensive queue manager
pub struct ExtensiveQueueManager {
    config: ExtensiveConfig,
    queue: VecDeque<CrawlTask>,
    processed_urls: HashMap<String, Instant>,
    stats: DiscoveryStats,
    pages_processed: usize,
}

impl ExtensiveQueueManager {
    /// Create a new extensive queue manager
    pub fn new(config: ExtensiveConfig) -> Result<Self, CrawlError> {
        Ok(Self {
            config,
            queue: VecDeque::new(),
            processed_urls: HashMap::new(),
            stats: DiscoveryStats::default(),
            pages_processed: 0,
        })
    }

    /// Add discovered links to the queue
    pub async fn add_discovered_links(
        &mut self,
        processed_links: Vec<ProcessedLink>,
    ) -> Result<usize, CrawlError> {
        if !self.config.should_crawl_extensively() {
            return Ok(0);
        }

        let mut added_count = 0;
        let start_time = Instant::now();

        for processed_link in processed_links {
            // Update discovery statistics
            self.update_discovery_stats(&processed_link);

            if !processed_link.should_crawl {
                self.stats.filtered += 1;
                continue;
            }

            // Check if URL was already processed recently
            if let Some(last_processed) = self.processed_urls.get(&processed_link.normalized_url) {
                let elapsed = start_time.duration_since(*last_processed);
                if elapsed < Duration::from_secs(3600) {
                    // Don't re-crawl within 1 hour
                    self.stats.filtered += 1;
                    continue;
                }
            }

            // Check queue capacity
            if let Some(max_size) = self.config.max_queue_size {
                if self.queue.len() >= max_size {
                    break;
                }
            }

            // Create crawl task using configurable priority thresholds
            let thresholds = &self.config.priority_thresholds;
            let task_priority = if processed_link.priority <= thresholds.low_threshold {
                crate::core::types::TaskPriority::Low
            } else if processed_link.priority <= thresholds.normal_threshold {
                crate::core::types::TaskPriority::Normal
            } else if processed_link.priority <= thresholds.high_threshold {
                crate::core::types::TaskPriority::High
            } else {
                crate::core::types::TaskPriority::Critical
            };

            let crawl_task = CrawlTask::new(
                url::Url::parse(&processed_link.normalized_url)
                    .map_err(|_| CrawlError::InvalidUrl(processed_link.normalized_url.clone()))?,
                task_priority,
                3, // max_retries - this could also be configurable if needed
            );

            self.queue.push_back(crawl_task);
            self.processed_urls
                .insert(processed_link.normalized_url, start_time);
            self.stats.queued += 1;
            added_count += 1;

            // Apply delay if configured
            if let Some(delay_ms) = self.config.queue_delay_ms {
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        self.pages_processed += 1;
        self.update_discovery_rate();

        Ok(added_count)
    }

    /// Get the next URL to crawl
    pub fn get_next_url(&mut self) -> Option<CrawlTask> {
        self.queue.pop_front()
    }

    /// Get current queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        let mut depth_distribution = HashMap::new();

        for task in &self.queue {
            let depth = task.depth; // CrawlTask now has depth as usize, not Option<usize>
            *depth_distribution.entry(depth).or_insert(0) += 1;
        }

        let capacity_used = if let Some(max_size) = self.config.max_queue_size {
            self.queue.len() as f64 / max_size as f64
        } else {
            0.0
        };

        QueueStatus {
            total_queued: self.queue.len(),
            processed: self.stats.queued,
            failed: self.stats.filtered,
            depth_distribution,
            capacity_used,
        }
    }

    /// Get discovery statistics
    pub fn get_discovery_stats(&self) -> &DiscoveryStats {
        &self.stats
    }

    /// Clear old processed URLs to prevent memory growth
    pub fn cleanup_processed_urls(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.processed_urls
            .retain(|_, &mut timestamp| now.duration_since(timestamp) < max_age);
    }

    /// Check if queue has capacity for more URLs
    pub fn has_capacity(&self) -> bool {
        if let Some(max_size) = self.config.max_queue_size {
            self.queue.len() < max_size
        } else {
            true
        }
    }

    /// Get remaining queue capacity
    pub fn remaining_capacity(&self) -> Option<usize> {
        self.config
            .max_queue_size
            .map(|max| max.saturating_sub(self.queue.len()))
    }

    /// Prioritize queue by moving high-priority items to front
    pub fn prioritize_queue(&mut self) {
        let mut queue_vec: Vec<CrawlTask> = self.queue.drain(..).collect();
        queue_vec.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.queue.extend(queue_vec);
    }

    /// Update discovery statistics for a processed link
    fn update_discovery_stats(&mut self, processed_link: &ProcessedLink) {
        self.stats.total_discovered += 1;

        // Update category distribution
        let category = format!("{:?}", processed_link.category);
        *self
            .stats
            .category_distribution
            .entry(category)
            .or_insert(0) += 1;

        // Update average priority
        let total_priority: u64 = self.stats.average_priority as u64
            * (self.stats.total_discovered - 1) as u64
            + processed_link.priority as u64;
        self.stats.average_priority = total_priority as f64 / self.stats.total_discovered as f64;
    }

    /// Update discovery rate
    fn update_discovery_rate(&mut self) {
        if self.pages_processed > 0 {
            self.stats.discovery_rate =
                self.stats.total_discovered as f64 / self.pages_processed as f64;
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DiscoveryStats::default();
        self.pages_processed = 0;
    }

    /// Export queue state for persistence
    pub fn export_queue(&self) -> Vec<CrawlTask> {
        self.queue.iter().cloned().collect()
    }

    /// Import queue state from persistence
    pub fn import_queue(&mut self, tasks: Vec<CrawlTask>) -> Result<(), CrawlError> {
        self.queue.clear();

        // Validate and add tasks
        for task in tasks {
            if let Some(max_size) = self.config.max_queue_size {
                if self.queue.len() >= max_size {
                    break;
                }
            }
            self.queue.push_back(task);
        }

        Ok(())
    }
}
