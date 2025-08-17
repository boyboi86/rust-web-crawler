use crate::common::primitives::{BooleanFlag, DelayDuration, LimitValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    requests_per_second: f64,
    burst_size: LimitValue,
    enabled: BooleanFlag,
    per_domain: BooleanFlag,
}

impl RateLimitConfig {
    pub fn new(
        requests_per_second: f64,
        burst_size: LimitValue,
        enabled: BooleanFlag,
        per_domain: BooleanFlag,
    ) -> Self {
        Self {
            requests_per_second,
            burst_size,
            enabled,
            per_domain,
        }
    }
    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }
    pub fn burst_size(&self) -> u64 {
        self.burst_size.value()
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.is_enabled()
    }
    pub fn is_per_domain(&self) -> bool {
        self.per_domain.is_enabled()
    }
    pub fn request_interval(&self) -> DelayDuration {
        let millis = (1000.0 / self.requests_per_second) as u64;
        DelayDuration::from_millis(millis)
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 1.0,
            burst_size: LimitValue::new(5),
            enabled: BooleanFlag::enabled(),
            per_domain: BooleanFlag::enabled(),
        }
    }
}
