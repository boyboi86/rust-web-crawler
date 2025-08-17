//! Retry policy building blocks for reliable operations
//! Level 3 implementation: RetryPolicy

use std::time::Duration;

/// RetryPolicy - standardized retry logic building block with privacy-first design
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

    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            exponential_backoff: true,
        }
    }

    // Controlled access methods (privacy-first)
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

    // Builder pattern methods
    pub fn with_max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts.max(1); // Ensure at least 1 attempt
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
        if !self.exponential_backoff || attempt == 0 {
            return self.base_delay;
        }

        let delay_ms = self.base_delay.as_millis() * (2_u128.pow(attempt.saturating_sub(1)));
        let delay = Duration::from_millis(delay_ms.min(u64::MAX as u128) as u64);

        std::cmp::min(delay, self.max_delay)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    pub fn next_delay(&self, current_attempt: u32) -> Option<Duration> {
        if self.should_retry(current_attempt + 1) {
            Some(self.calculate_delay(current_attempt))
        } else {
            None
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::new();
        assert_eq!(policy.max_attempts(), 3);
        assert_eq!(policy.base_delay(), Duration::from_millis(500));
        assert_eq!(policy.max_delay(), Duration::from_secs(30));
        assert!(policy.uses_exponential_backoff());
    }

    #[test]
    fn test_retry_policy_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_attempts(), 1);
        assert_eq!(policy.base_delay(), Duration::from_millis(0));
        assert!(!policy.should_retry(1));
    }

    #[test]
    fn test_retry_policy_aggressive() {
        let policy = RetryPolicy::aggressive();
        assert_eq!(policy.max_attempts(), 5);
        assert_eq!(policy.base_delay(), Duration::from_millis(200));
        assert_eq!(policy.max_delay(), Duration::from_secs(10));
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::new()
            .with_max_attempts(10)
            .with_base_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(5))
            .with_exponential_backoff(false);

        assert_eq!(policy.max_attempts(), 10);
        assert_eq!(policy.base_delay(), Duration::from_millis(100));
        assert_eq!(policy.max_delay(), Duration::from_secs(5));
        assert!(!policy.uses_exponential_backoff());
    }

    #[test]
    fn test_retry_policy_calculate_delay() {
        let policy = RetryPolicy::new();
        
        let delay1 = policy.calculate_delay(0);
        let delay2 = policy.calculate_delay(1);
        let delay3 = policy.calculate_delay(2);
        
        // With exponential backoff: 500ms, 1000ms, 2000ms
        assert_eq!(delay1, Duration::from_millis(500));
        assert_eq!(delay2, Duration::from_millis(1000));
        assert_eq!(delay3, Duration::from_millis(2000));
        
        // Test without exponential backoff
        let linear_policy = RetryPolicy::new().with_exponential_backoff(false);
        assert_eq!(linear_policy.calculate_delay(0), Duration::from_millis(500));
        assert_eq!(linear_policy.calculate_delay(1), Duration::from_millis(500));
        assert_eq!(linear_policy.calculate_delay(2), Duration::from_millis(500));
    }

    #[test]
    fn test_retry_policy_max_delay_cap() {
        let policy = RetryPolicy::new()
            .with_base_delay(Duration::from_millis(500))
            .with_max_delay(Duration::from_millis(1200));

        // Should cap at max_delay
        let delay = policy.calculate_delay(3); // Would be 4000ms, but capped
        assert_eq!(delay, Duration::from_millis(1200));
    }

    #[test]
    fn test_retry_policy_should_retry() {
        let policy = RetryPolicy::new(); // max_attempts = 3
        
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
        assert!(!policy.should_retry(4));
    }

    #[test]
    fn test_retry_policy_next_delay() {
        let policy = RetryPolicy::new(); // max_attempts = 3
        
        assert_eq!(policy.next_delay(0), Some(Duration::from_millis(500)));
        assert_eq!(policy.next_delay(1), Some(Duration::from_millis(1000)));
        assert_eq!(policy.next_delay(2), None); // No more retries
    }

    #[test]
    fn test_retry_policy_min_attempts() {
        let policy = RetryPolicy::new().with_max_attempts(0);
        assert_eq!(policy.max_attempts(), 1); // Should be capped at minimum 1
    }
}
