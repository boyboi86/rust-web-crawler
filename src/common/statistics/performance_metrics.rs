use crate::common::primitives::{CountValue, PercentageValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    requests_per_second: f64,
    average_response_time: f64,
    success_rate: PercentageValue,
    throughput: CountValue,
    error_count: CountValue,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            requests_per_second: 0.0,
            average_response_time: 0.0,
            success_rate: PercentageValue::default(),
            throughput: CountValue::default(),
            error_count: CountValue::default(),
        }
    }
    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }
    pub fn average_response_time(&self) -> f64 {
        self.average_response_time
    }
    pub fn success_rate(&self) -> PercentageValue {
        self.success_rate
    }
    pub fn throughput(&self) -> u64 {
        self.throughput.value()
    }
    pub fn error_count(&self) -> u64 {
        self.error_count.value()
    }
    pub fn update_requests_per_second(&mut self, rps: f64) {
        self.requests_per_second = rps;
    }
    pub fn update_average_response_time(&mut self, time_ms: f64) {
        self.average_response_time = time_ms;
    }
    pub fn update_success_rate(&mut self, rate: PercentageValue) {
        self.success_rate = rate;
    }
    pub fn increment_throughput(&mut self) {
        self.throughput.increment();
    }
    pub fn increment_error_count(&mut self) {
        self.error_count.increment();
    }
    pub fn reset(&mut self) {
        self.requests_per_second = 0.0;
        self.average_response_time = 0.0;
        self.success_rate = PercentageValue::default();
        self.throughput.reset();
        self.error_count.reset();
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}
