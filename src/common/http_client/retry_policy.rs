// Level 3: RetryPolicy
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    exponential_backoff: bool,
}

impl RetryPolicy {
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            exponential_backoff: true,
        }
    }
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            base_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            exponential_backoff: false,
        }
    }
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            exponential_backoff: true,
        }
    }
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
    pub fn base_delay(&self) -> Duration {
        self.base_delay
    }
    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }
    pub fn uses_exponential_backoff(&self) -> bool {
        self.exponential_backoff
    }
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
    pub fn with_exponential_backoff(mut self, enabled: bool) -> Self {
        self.exponential_backoff = enabled;
        self
    }
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if !self.exponential_backoff {
            return self.base_delay;
        }
        let delay_ms = self.base_delay.as_millis() * (2_u128.pow(attempt));
        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.max_delay)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}
