use anyhow::Error;
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};

use crate::core::{DomainRateLimit, RateLimiter};

/// Tracks request timestamps for sliding window rate limiting
#[derive(Debug)]
pub struct DomainRequestTracker {
    pub request_timestamps: VecDeque<u64>, // Unix timestamps in milliseconds
    pub last_cleaned: u64,
}

impl Default for DomainRequestTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRequestTracker {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            request_timestamps: VecDeque::new(),
            last_cleaned: now,
        }
    }

    /// Clean old timestamps outside the sliding window
    pub fn clean_old_timestamps(&mut self, window_size_ms: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let cutoff = now.saturating_sub(window_size_ms);

        // Remove timestamps older than window
        while let Some(&front) = self.request_timestamps.front() {
            if front < cutoff {
                self.request_timestamps.pop_front();
            } else {
                break;
            }
        }

        self.last_cleaned = now;
    }

    /// Check if we can make a request without exceeding rate limit
    pub fn can_make_request(&mut self, rate_limit: &DomainRateLimit) -> bool {
        self.clean_old_timestamps(rate_limit.rate.window_size_ms);
        self.request_timestamps.len() < rate_limit.rate.max_requests_per_second as usize
    }

    /// Record a new request timestamp
    pub fn record_request(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.request_timestamps.push_back(now);
    }

    /// Calculate how long to wait before next request (with randomness)
    pub fn calculate_wait_time(&self, rate_limit: &DomainRateLimit) -> u64 {
        if self.request_timestamps.is_empty() {
            return 0;
        }

        // Calculate base delay between requests
        let base_delay =
            rate_limit.rate.window_size_ms / rate_limit.rate.max_requests_per_second as u64;

        // Add some randomness to avoid thundering herd
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=base_delay / 4); // Up to 25% jitter

        base_delay + jitter
    }
}

/// Global rate limiter for all domains with optimized locking
pub struct GlobalRateLimiter {
    pub domain_trackers: Arc<RwLock<HashMap<String, DomainRequestTracker>>>,
    default_rate_limit: DomainRateLimit,
    domain_specific_limits: HashMap<String, DomainRateLimit>,
    last_cleanup: Arc<Mutex<tokio::time::Instant>>, // Track last cleanup time
}

impl GlobalRateLimiter {
    pub fn new(default_rate_limit: DomainRateLimit) -> Self {
        Self {
            domain_trackers: Arc::new(RwLock::new(HashMap::new())),
            default_rate_limit,
            domain_specific_limits: HashMap::new(),
            last_cleanup: Arc::new(Mutex::new(tokio::time::Instant::now())),
        }
    }

    /// Add domain-specific rate limit
    pub fn add_domain_limit(&mut self, domain: String, rate_limit: DomainRateLimit) {
        self.domain_specific_limits.insert(domain, rate_limit);
    }

    /// Get rate limit for a specific domain
    fn get_rate_limit(&self, domain: &str) -> &DomainRateLimit {
        self.domain_specific_limits
            .get(domain)
            .unwrap_or(&self.default_rate_limit)
    }

    /// Check if request is allowed and apply rate limiting (optimized with RwLock)
    pub async fn check_and_wait(&self, domain: &str) -> Result<(), Error> {
        let rate_limit = self.get_rate_limit(domain).clone();

        // Try to get read lock first to check if we can proceed
        let can_proceed = {
            let trackers = self.domain_trackers.read().await;
            if let Some(tracker) = trackers.get(domain) {
                // Clone the necessary data to avoid holding the lock
                let request_count = tracker.request_timestamps.len();
                request_count < rate_limit.rate.max_requests_per_second as usize
            } else {
                true // New domain, can proceed
            }
        };

        if !can_proceed {
            // Need to wait, calculate delay
            let wait_time = {
                let trackers = self.domain_trackers.read().await;
                if let Some(tracker) = trackers.get(domain) {
                    tracker.calculate_wait_time(&rate_limit)
                } else {
                    0
                }
            };

            if wait_time > 0 {
                tokio::time::sleep(Duration::from_millis(wait_time)).await;
            }
        }

        // Now acquire write lock to update tracker
        {
            let mut trackers = self.domain_trackers.write().await;
            let tracker = trackers
                .entry(domain.to_string())
                .or_insert_with(DomainRequestTracker::new);

            // Clean old timestamps and check again
            tracker.clean_old_timestamps(rate_limit.rate.window_size_ms);

            if !tracker.can_make_request(&rate_limit) {
                // Still need to wait after cleanup
                let wait_time = tracker.calculate_wait_time(&rate_limit);
                drop(trackers); // Release lock before sleeping

                if wait_time > 0 {
                    tokio::time::sleep(Duration::from_millis(wait_time)).await;
                }

                // Re-acquire lock and record request
                let mut trackers = self.domain_trackers.write().await;
                let tracker = trackers
                    .entry(domain.to_string())
                    .or_insert_with(DomainRequestTracker::new);
                tracker.record_request();
            } else {
                // Can proceed, record the request
                tracker.record_request();
            }
        }

        // Periodic cleanup of old trackers (every 5 minutes)
        let should_cleanup = {
            let mut last_cleanup = self.last_cleanup.lock().await;
            let now = tokio::time::Instant::now();
            if now.duration_since(*last_cleanup) > Duration::from_secs(300) {
                *last_cleanup = now;
                true
            } else {
                false
            }
        };

        if should_cleanup {
            self.cleanup_old_trackers().await;
        }

        Ok(())
    }

    /// Clean up trackers for domains that haven't been accessed recently
    async fn cleanup_old_trackers(&self) {
        let mut trackers = self.domain_trackers.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        trackers.retain(|_, tracker| {
            // Keep trackers that have recent activity (within last hour)
            tracker.last_cleaned + 3_600_000 > now
        });
    }
}

impl RateLimiter for GlobalRateLimiter {
    async fn check_and_wait(&self, domain: &str) -> Result<(), Error> {
        self.check_and_wait(domain).await
    }

    async fn get_current_request_count(&self, domain: &str) -> usize {
        let trackers = self.domain_trackers.read().await;
        trackers
            .get(domain)
            .map(|tracker| tracker.request_timestamps.len())
            .unwrap_or(0)
    }
}
