use crate::common::primitives::{DelayDuration, TimeoutDuration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    timeout: TimeoutDuration,
    retry_delay: DelayDuration,
    max_retry_delay: DelayDuration,
    keepalive: TimeoutDuration,
}

impl TimingConfig {
    pub fn new(
        timeout: TimeoutDuration,
        retry_delay: DelayDuration,
        max_retry_delay: DelayDuration,
        keepalive: TimeoutDuration,
    ) -> Self {
        Self {
            timeout,
            retry_delay,
            max_retry_delay,
            keepalive,
        }
    }
    pub fn timeout(&self) -> TimeoutDuration {
        self.timeout
    }
    pub fn retry_delay(&self) -> DelayDuration {
        self.retry_delay
    }
    pub fn max_retry_delay(&self) -> DelayDuration {
        self.max_retry_delay
    }
    pub fn keepalive(&self) -> TimeoutDuration {
        self.keepalive
    }
    pub fn set_timeout(&mut self, timeout: TimeoutDuration) {
        self.timeout = timeout;
    }
    pub fn set_retry_delay(&mut self, delay: DelayDuration) {
        self.retry_delay = delay;
    }
    pub fn set_max_retry_delay(&mut self, delay: DelayDuration) {
        self.max_retry_delay = delay;
    }
    pub fn set_keepalive(&mut self, keepalive: TimeoutDuration) {
        self.keepalive = keepalive;
    }
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            timeout: TimeoutDuration::default(),
            retry_delay: DelayDuration::default(),
            max_retry_delay: DelayDuration::from_secs(30),
            keepalive: TimeoutDuration::from_secs(30),
        }
    }
}
