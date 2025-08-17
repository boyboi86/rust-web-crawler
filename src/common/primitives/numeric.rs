/// Numeric primitive building blocks for counts, limits, percentages, and scores
/// Level 3 implementation - complete struct and functionality for numeric types
use serde::{Deserialize, Serialize};

/// Building block for count values - ensures consistent counting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct CountValue(u64);

impl CountValue {
    pub fn new(count: u64) -> Self {
        Self(count)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn add(&mut self, amount: u64) {
        self.0 += amount;
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn saturating_add(&mut self, amount: u64) {
        self.0 = self.0.saturating_add(amount);
    }

    pub fn saturating_sub(&mut self, amount: u64) {
        self.0 = self.0.saturating_sub(amount);
    }
}

impl From<u64> for CountValue {
    fn from(count: u64) -> Self {
        Self(count)
    }
}

impl From<usize> for CountValue {
    fn from(count: usize) -> Self {
        Self(count as u64)
    }
}

impl std::ops::Add for CountValue {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::ops::Sub for CountValue {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

/// Building block for limit values - ensures consistent limits
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct LimitValue(u64);

impl LimitValue {
    pub fn new(limit: u64) -> Self {
        Self(limit)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_exceeded(&self, current: u64) -> bool {
        current >= self.0
    }

    pub fn is_exceeded_by_count(&self, count: CountValue) -> bool {
        count.value() >= self.0
    }

    pub fn remaining(&self, current: u64) -> u64 {
        self.0.saturating_sub(current)
    }

    pub fn remaining_for_count(&self, count: CountValue) -> u64 {
        self.0.saturating_sub(count.value())
    }

    pub fn is_unlimited(&self) -> bool {
        self.0 == u64::MAX
    }

    pub fn unlimited() -> Self {
        Self(u64::MAX)
    }
}

impl From<u64> for LimitValue {
    fn from(limit: u64) -> Self {
        Self(limit)
    }
}

impl From<usize> for LimitValue {
    fn from(limit: usize) -> Self {
        Self(limit as u64)
    }
}

/// Building block for percentage values - ensures consistent percentage handling
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PercentageValue(f64);

impl PercentageValue {
    pub fn new(percentage: f64) -> Self {
        Self(percentage.max(0.0).min(100.0))
    }

    pub fn from_ratio(ratio: f64) -> Self {
        Self::new(ratio * 100.0)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn as_ratio(&self) -> f64 {
        self.0 / 100.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    pub fn is_full(&self) -> bool {
        self.0 == 100.0
    }

    pub fn apply_to(&self, base: f64) -> f64 {
        base * self.as_ratio()
    }

    pub fn apply_to_count(&self, count: CountValue) -> f64 {
        count.value() as f64 * self.as_ratio()
    }
}

impl Default for PercentageValue {
    fn default() -> Self {
        Self(0.0)
    }
}

impl From<f64> for PercentageValue {
    fn from(percentage: f64) -> Self {
        Self::new(percentage)
    }
}

/// Building block for attempt counts - ensures consistent attempt tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct AttemptCount(u32);

impl AttemptCount {
    pub fn new(attempts: u32) -> Self {
        Self(attempts)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn is_exceeded(&self, max_attempts: u32) -> bool {
        self.0 >= max_attempts
    }

    pub fn remaining(&self, max_attempts: u32) -> u32 {
        max_attempts.saturating_sub(self.0)
    }
}

impl From<u32> for AttemptCount {
    fn from(attempts: u32) -> Self {
        Self(attempts)
    }
}

/// Building block for priority scores - ensures consistent priority handling
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PriorityScore(f64);

impl PriorityScore {
    pub fn new(score: f64) -> Self {
        Self(score.max(0.0))
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }

    pub fn is_medium(&self) -> bool {
        self.0 >= 0.4 && self.0 < 0.8
    }

    pub fn is_low(&self) -> bool {
        self.0 < 0.4
    }

    pub fn increase(&mut self, amount: f64) {
        self.0 += amount;
    }

    pub fn decrease(&mut self, amount: f64) {
        self.0 = (self.0 - amount).max(0.0);
    }

    pub fn adjust(&mut self, adjustment: f64) {
        if adjustment >= 0.0 {
            self.increase(adjustment);
        } else {
            self.decrease(-adjustment);
        }
    }

    pub fn high() -> Self {
        Self(0.9)
    }

    pub fn medium() -> Self {
        Self(0.5)
    }

    pub fn low() -> Self {
        Self(0.1)
    }
}

impl Default for PriorityScore {
    fn default() -> Self {
        Self::medium()
    }
}

impl From<f64> for PriorityScore {
    fn from(score: f64) -> Self {
        Self::new(score)
    }
}

impl std::cmp::Eq for PriorityScore {}

impl std::cmp::Ord for PriorityScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_value_operations() {
        let mut count = CountValue::new(5);
        assert_eq!(count.value(), 5);
        assert!(!count.is_zero());

        count.increment();
        assert_eq!(count.value(), 6);

        count.add(10);
        assert_eq!(count.value(), 16);

        count.reset();
        assert_eq!(count.value(), 0);
        assert!(count.is_zero());
    }

    #[test]
    fn test_count_value_arithmetic() {
        let count1 = CountValue::new(10);
        let count2 = CountValue::new(5);

        let sum = count1 + count2;
        assert_eq!(sum.value(), 15);

        let diff = count1 - count2;
        assert_eq!(diff.value(), 5);
    }

    #[test]
    fn test_limit_value_checks() {
        let limit = LimitValue::new(10);

        assert!(!limit.is_exceeded(5));
        assert!(limit.is_exceeded(10));
        assert!(limit.is_exceeded(15));

        assert_eq!(limit.remaining(5), 5);
        assert_eq!(limit.remaining(15), 0);
    }

    #[test]
    fn test_limit_value_unlimited() {
        let unlimited = LimitValue::unlimited();
        assert!(unlimited.is_unlimited());
        assert!(!unlimited.is_exceeded(u64::MAX - 1));
    }

    #[test]
    fn test_percentage_value_creation() {
        let percent = PercentageValue::new(75.0);
        assert_eq!(percent.value(), 75.0);
        assert_eq!(percent.as_ratio(), 0.75);

        let from_ratio = PercentageValue::from_ratio(0.5);
        assert_eq!(from_ratio.value(), 50.0);
    }

    #[test]
    fn test_percentage_value_bounds() {
        let over = PercentageValue::new(150.0);
        assert_eq!(over.value(), 100.0);

        let under = PercentageValue::new(-50.0);
        assert_eq!(under.value(), 0.0);
    }

    #[test]
    fn test_percentage_value_application() {
        let percent = PercentageValue::new(25.0);
        assert_eq!(percent.apply_to(100.0), 25.0);

        let count = CountValue::new(200);
        assert_eq!(percent.apply_to_count(count), 50.0);
    }

    #[test]
    fn test_attempt_count_operations() {
        let mut attempts = AttemptCount::new(0);
        assert!(attempts.is_zero());

        attempts.increment();
        assert_eq!(attempts.value(), 1);
        assert!(!attempts.is_zero());

        assert!(!attempts.is_exceeded(5));
        assert_eq!(attempts.remaining(5), 4);
    }

    #[test]
    fn test_priority_score_categories() {
        let high = PriorityScore::high();
        assert!(high.is_high());
        assert!(!high.is_medium());
        assert!(!high.is_low());

        let medium = PriorityScore::medium();
        assert!(!medium.is_high());
        assert!(medium.is_medium());
        assert!(!medium.is_low());

        let low = PriorityScore::low();
        assert!(!low.is_high());
        assert!(!low.is_medium());
        assert!(low.is_low());
    }

    #[test]
    fn test_priority_score_modifications() {
        let mut priority = PriorityScore::new(0.5);

        priority.increase(0.3);
        assert_eq!(priority.value(), 0.8);

        priority.decrease(0.2);
        assert_eq!(priority.value(), 0.6);

        priority.decrease(1.0);
        assert_eq!(priority.value(), 0.0);
    }

    #[test]
    fn test_type_conversions() {
        let count_from_u64 = CountValue::from(42u64);
        assert_eq!(count_from_u64.value(), 42);

        let count_from_usize = CountValue::from(100usize);
        assert_eq!(count_from_usize.value(), 100);

        let limit_from_u64 = LimitValue::from(50u64);
        assert_eq!(limit_from_u64.value(), 50);

        let percent_from_f64 = PercentageValue::from(85.0);
        assert_eq!(percent_from_f64.value(), 85.0);
    }
}
