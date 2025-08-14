use std::time::{Duration, Instant};

/// Session-level statistics and metrics
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub total_urls: usize,
    pub processed_urls: usize,
    pub successful_urls: usize,
    pub failed_urls: usize,
    pub total_processing_time: Duration,
    pub average_processing_time: Duration,
    pub throughput_urls_per_second: f64,
}

impl SessionStatistics {
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            total_urls: 0,
            processed_urls: 0,
            successful_urls: 0,
            failed_urls: 0,
            total_processing_time: Duration::from_millis(0),
            average_processing_time: Duration::from_millis(0),
            throughput_urls_per_second: 0.0,
        }
    }

    /// Mark session as started
    pub fn session_started(&mut self, total_urls: usize) {
        self.start_time = Some(Instant::now());
        self.total_urls = total_urls;
    }

    /// Mark session as completed
    pub fn session_completed(&mut self, total_duration: Duration) {
        self.end_time = Some(Instant::now());

        // Calculate throughput
        if total_duration.as_secs() > 0 {
            self.throughput_urls_per_second =
                self.processed_urls as f64 / total_duration.as_secs_f64();
        }
    }

    /// Record a completed URL
    pub fn url_completed(&mut self, success: bool, processing_time: Duration) {
        self.processed_urls += 1;
        self.total_processing_time += processing_time;

        if success {
            self.successful_urls += 1;
        } else {
            self.failed_urls += 1;
        }

        // Update average processing time
        if self.processed_urls > 0 {
            self.average_processing_time = self.total_processing_time / self.processed_urls as u32;
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.processed_urls == 0 {
            0.0
        } else {
            (self.successful_urls as f64 / self.processed_urls as f64) * 100.0
        }
    }

    /// Get completion rate as percentage
    pub fn completion_rate(&self) -> f64 {
        if self.total_urls == 0 {
            0.0
        } else {
            (self.processed_urls as f64 / self.total_urls as f64) * 100.0
        }
    }

    /// Get session duration
    pub fn session_duration(&self) -> Option<Duration> {
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            Some(end.duration_since(start))
        } else if let Some(start) = self.start_time {
            Some(start.elapsed())
        } else {
            None
        }
    }
}

/// Real-time statistics for monitoring
#[derive(Debug, Clone)]
pub struct RealTimeStats {
    pub current_throughput: f64,
    pub estimated_completion_time: Option<Duration>,
    pub urls_remaining: usize,
    pub current_processing_time: Duration,
}

impl RealTimeStats {
    pub fn calculate_from_session(stats: &SessionStatistics) -> Self {
        let urls_remaining = stats.total_urls.saturating_sub(stats.processed_urls);

        let estimated_completion_time =
            if stats.throughput_urls_per_second > 0.0 && urls_remaining > 0 {
                let remaining_seconds = urls_remaining as f64 / stats.throughput_urls_per_second;
                Some(Duration::from_secs_f64(remaining_seconds))
            } else {
                None
            };

        Self {
            current_throughput: stats.throughput_urls_per_second,
            estimated_completion_time,
            urls_remaining,
            current_processing_time: stats.average_processing_time,
        }
    }
}
