/// Temporal primitive building blocks for durations and timeouts
/// Level 3 implementation - complete struct and functionality for time-related types
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Building block for timeout durations - ensures consistent timeout handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeoutDuration(Duration);

impl TimeoutDuration {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    pub fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    pub fn from_micros(micros: u64) -> Self {
        Self(Duration::from_micros(micros))
    }

    pub fn duration(&self) -> Duration {
        self.0
    }

    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }

    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    pub fn as_micros(&self) -> u128 {
        self.0.as_micros()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_expired(&self, elapsed: Duration) -> bool {
        elapsed >= self.0
    }

    pub fn remaining(&self, elapsed: Duration) -> Duration {
        self.0.saturating_sub(elapsed)
    }

    pub fn multiply(&self, factor: u32) -> Self {
        Self(self.0 * factor)
    }

    pub fn divide(&self, divisor: u32) -> Self {
        if divisor == 0 {
            Self(Duration::ZERO)
        } else {
            Self(self.0 / divisor)
        }
    }

    /// Create a short timeout (5 seconds)
    pub fn short() -> Self {
        Self::from_secs(5)
    }

    /// Create a medium timeout (30 seconds)
    pub fn medium() -> Self {
        Self::from_secs(30)
    }

    /// Create a long timeout (120 seconds)
    pub fn long() -> Self {
        Self::from_secs(120)
    }
}

impl Default for TimeoutDuration {
    fn default() -> Self {
        Self::medium()
    }
}

impl From<Duration> for TimeoutDuration {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<TimeoutDuration> for Duration {
    fn from(timeout: TimeoutDuration) -> Self {
        timeout.0
    }
}

/// Building block for delay durations - ensures consistent delay handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DelayDuration(Duration);

impl DelayDuration {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    pub fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    pub fn from_micros(micros: u64) -> Self {
        Self(Duration::from_micros(micros))
    }

    pub fn duration(&self) -> Duration {
        self.0
    }

    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }

    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    pub fn as_micros(&self) -> u128 {
        self.0.as_micros()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_ready(&self, elapsed: Duration) -> bool {
        elapsed >= self.0
    }

    pub fn remaining(&self, elapsed: Duration) -> Duration {
        self.0.saturating_sub(elapsed)
    }

    pub fn multiply(&self, factor: u32) -> Self {
        Self(self.0 * factor)
    }

    pub fn divide(&self, divisor: u32) -> Self {
        if divisor == 0 {
            Self(Duration::ZERO)
        } else {
            Self(self.0 / divisor)
        }
    }

    pub fn add(&self, other: DelayDuration) -> Self {
        Self(self.0 + other.0)
    }

    pub fn saturating_sub(&self, other: DelayDuration) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Create an immediate delay (0 duration)
    pub fn immediate() -> Self {
        Self(Duration::ZERO)
    }

    /// Create a short delay (100ms)
    pub fn short() -> Self {
        Self::from_millis(100)
    }

    /// Create a medium delay (1 second)
    pub fn medium() -> Self {
        Self::from_secs(1)
    }

    /// Create a long delay (5 seconds)
    pub fn long() -> Self {
        Self::from_secs(5)
    }

    /// Create an exponential backoff delay
    pub fn exponential_backoff(
        base_delay: DelayDuration,
        attempt: u32,
        max_delay: DelayDuration,
    ) -> Self {
        let delay = base_delay.multiply(2_u32.saturating_pow(attempt));
        if delay.0 > max_delay.0 {
            max_delay
        } else {
            delay
        }
    }
}

impl Default for DelayDuration {
    fn default() -> Self {
        Self::short()
    }
}

impl From<Duration> for DelayDuration {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<DelayDuration> for Duration {
    fn from(delay: DelayDuration) -> Self {
        delay.0
    }
}

impl std::ops::Add for DelayDuration {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for DelayDuration {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

impl std::ops::Mul<u32> for DelayDuration {
    type Output = Self;

    fn mul(self, factor: u32) -> Self {
        Self(self.0 * factor)
    }
}

impl std::ops::Div<u32> for DelayDuration {
    type Output = Self;

    fn div(self, divisor: u32) -> Self {
        if divisor == 0 {
            Self(Duration::ZERO)
        } else {
            Self(self.0 / divisor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_duration_creation() {
        let timeout = TimeoutDuration::from_secs(10);
        assert_eq!(timeout.as_secs(), 10);
        assert_eq!(timeout.as_millis(), 10_000);
        assert!(!timeout.is_zero());
    }

    #[test]
    fn test_timeout_duration_expiry() {
        let timeout = TimeoutDuration::from_secs(5);
        let elapsed_short = Duration::from_secs(3);
        let elapsed_long = Duration::from_secs(7);

        assert!(!timeout.is_expired(elapsed_short));
        assert!(timeout.is_expired(elapsed_long));

        assert_eq!(timeout.remaining(elapsed_short), Duration::from_secs(2));
        assert_eq!(timeout.remaining(elapsed_long), Duration::ZERO);
    }

    #[test]
    fn test_timeout_duration_operations() {
        let timeout = TimeoutDuration::from_secs(10);

        let doubled = timeout.multiply(2);
        assert_eq!(doubled.as_secs(), 20);

        let halved = timeout.divide(2);
        assert_eq!(halved.as_secs(), 5);

        let zero_div = timeout.divide(0);
        assert!(zero_div.is_zero());
    }

    #[test]
    fn test_timeout_duration_presets() {
        let short = TimeoutDuration::short();
        assert_eq!(short.as_secs(), 5);

        let medium = TimeoutDuration::medium();
        assert_eq!(medium.as_secs(), 30);

        let long = TimeoutDuration::long();
        assert_eq!(long.as_secs(), 120);

        let default = TimeoutDuration::default();
        assert_eq!(default.as_secs(), 30);
    }

    #[test]
    fn test_delay_duration_creation() {
        let delay = DelayDuration::from_millis(500);
        assert_eq!(delay.as_millis(), 500);
        assert!(!delay.is_zero());
    }

    #[test]
    fn test_delay_duration_readiness() {
        let delay = DelayDuration::from_secs(2);
        let elapsed_short = Duration::from_secs(1);
        let elapsed_long = Duration::from_secs(3);

        assert!(!delay.is_ready(elapsed_short));
        assert!(delay.is_ready(elapsed_long));

        assert_eq!(delay.remaining(elapsed_short), Duration::from_secs(1));
        assert_eq!(delay.remaining(elapsed_long), Duration::ZERO);
    }

    #[test]
    fn test_delay_duration_arithmetic() {
        let delay1 = DelayDuration::from_secs(3);
        let delay2 = DelayDuration::from_secs(2);

        let sum = delay1 + delay2;
        assert_eq!(sum.as_secs(), 5);

        let diff = delay1 - delay2;
        assert_eq!(diff.as_secs(), 1);

        let multiplied = delay1 * 3;
        assert_eq!(multiplied.as_secs(), 9);

        let divided = delay1 / 2;
        assert_eq!(divided.as_secs(), 1);
    }

    #[test]
    fn test_delay_duration_presets() {
        let immediate = DelayDuration::immediate();
        assert!(immediate.is_zero());

        let short = DelayDuration::short();
        assert_eq!(short.as_millis(), 100);

        let medium = DelayDuration::medium();
        assert_eq!(medium.as_secs(), 1);

        let long = DelayDuration::long();
        assert_eq!(long.as_secs(), 5);

        let default = DelayDuration::default();
        assert_eq!(default.as_millis(), 100);
    }

    #[test]
    fn test_exponential_backoff() {
        let base = DelayDuration::from_millis(100);
        let max = DelayDuration::from_secs(10);

        let attempt0 = DelayDuration::exponential_backoff(base, 0, max);
        assert_eq!(attempt0.as_millis(), 100);

        let attempt1 = DelayDuration::exponential_backoff(base, 1, max);
        assert_eq!(attempt1.as_millis(), 200);

        let attempt2 = DelayDuration::exponential_backoff(base, 2, max);
        assert_eq!(attempt2.as_millis(), 400);

        // Should cap at max delay
        let attempt_high = DelayDuration::exponential_backoff(base, 20, max);
        assert_eq!(attempt_high.as_secs(), 10);
    }

    #[test]
    fn test_duration_conversions() {
        let std_duration = Duration::from_secs(42);

        let timeout = TimeoutDuration::from(std_duration);
        assert_eq!(timeout.as_secs(), 42);

        let back_to_duration: Duration = timeout.into();
        assert_eq!(back_to_duration, std_duration);

        let delay = DelayDuration::from(std_duration);
        assert_eq!(delay.as_secs(), 42);

        let back_to_duration2: Duration = delay.into();
        assert_eq!(back_to_duration2, std_duration);
    }

    #[test]
    fn test_zero_durations() {
        let zero_timeout = TimeoutDuration::new(Duration::ZERO);
        assert!(zero_timeout.is_zero());

        let zero_delay = DelayDuration::immediate();
        assert!(zero_delay.is_zero());

        let any_elapsed = Duration::from_secs(1);
        assert!(zero_timeout.is_expired(any_elapsed));
        assert!(zero_delay.is_ready(any_elapsed));
    }
}
