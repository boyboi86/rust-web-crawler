use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTiming {
    request_times: Vec<Duration>,
    average_time: Duration,
    min_time: Option<Duration>,
    max_time: Option<Duration>,
}

impl PerformanceTiming {
    pub fn new() -> Self {
        Self {
            request_times: Vec::new(),
            average_time: Duration::default(),
            min_time: None,
            max_time: None,
        }
    }
    pub fn record_time(&mut self, duration: Duration) {
        self.request_times.push(duration);
        self.update_stats();
    }
    pub fn average_time(&self) -> Duration {
        self.average_time
    }
    pub fn min_time(&self) -> Option<Duration> {
        self.min_time
    }
    pub fn max_time(&self) -> Option<Duration> {
        self.max_time
    }
    pub fn sample_count(&self) -> usize {
        self.request_times.len()
    }
    pub fn reset(&mut self) {
        self.request_times.clear();
        self.average_time = Duration::default();
        self.min_time = None;
        self.max_time = None;
    }
    fn update_stats(&mut self) {
        if self.request_times.is_empty() {
            return;
        }
        let total: Duration = self.request_times.iter().sum();
        self.average_time = total / self.request_times.len() as u32;
        self.min_time = self.request_times.iter().min().copied();
        self.max_time = self.request_times.iter().max().copied();
    }
}

impl Default for PerformanceTiming {
    fn default() -> Self {
        Self::new()
    }
}
