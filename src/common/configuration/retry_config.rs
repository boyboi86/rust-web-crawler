use crate::common::primitives::{AttemptCount, BooleanFlag, DelayDuration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    max_attempts: AttemptCount,
    base_delay: DelayDuration,
    max_delay: DelayDuration,
    exponential_backoff: BooleanFlag,
    backoff_multiplier: f64,
}

impl RetryConfig {
    pub fn new(
        max_attempts: AttemptCount,
        base_delay: DelayDuration,
        max_delay: DelayDuration,
        exponential_backoff: BooleanFlag,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            exponential_backoff,
            backoff_multiplier,
        }
    }
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts.value()
    }
    pub fn base_delay(&self) -> DelayDuration {
        self.base_delay
    }
    pub fn max_delay(&self) -> DelayDuration {
        self.max_delay
    }
    pub fn is_exponential_backoff(&self) -> bool {
        self.exponential_backoff.is_enabled()
    }
    pub fn backoff_multiplier(&self) -> f64 {
        self.backoff_multiplier
    }
    pub fn calculate_delay(&self, attempt: u32) -> DelayDuration {
        if self.exponential_backoff.is_enabled() {
            let multiplier = self.backoff_multiplier.powi(attempt as i32);
            let delay_ms = (self.base_delay.as_millis() as f64 * multiplier) as u64;
            let capped_delay = delay_ms.min(self.max_delay.as_millis() as u64);
            DelayDuration::from_millis(capped_delay)
        } else {
            self.base_delay
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: AttemptCount::new(3),
            base_delay: DelayDuration::from_millis(1000),
            max_delay: DelayDuration::from_secs(30),
            exponential_backoff: BooleanFlag::enabled(),
            backoff_multiplier: 2.0,
        }
    }
}
