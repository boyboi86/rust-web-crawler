/// Level 2 Assembly Module for Primitives
/// Following Rule 7: Level 2 modules should ONLY assemble Level 3 components
/// This module assembles all primitive building blocks without implementing business logic
// Level 3 sub-modules - complete implementations
pub mod collections;
pub mod flags;
pub mod identifiers;
pub mod numeric;
pub mod strings;
pub mod temporal;

// Re-export all primitive types for external use
// String primitives
pub use strings::{DomainString, UrlString};

// Identifier primitives
pub use identifiers::{SessionId, TaskId};

// Numeric primitives
pub use numeric::{AttemptCount, CountValue, LimitValue, PercentageValue, PriorityScore};

// Temporal primitives
pub use temporal::{DelayDuration, TimeoutDuration};

// Flag primitives
pub use flags::BooleanFlag;

// Collection primitives
pub use collections::OptionalVec;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_primitive_type_integration() {
        // Test that all primitive types work together
        let url = UrlString::new("https://example.com");
        let domain = DomainString::new("example.com");
        let session_id = SessionId::generate();
        let task_id = TaskId::generate();
        let count = CountValue::new(5);
        let limit = LimitValue::new(10);
        let percentage = PercentageValue::new(75.0);
        let attempts = AttemptCount::new(3);
        let priority = PriorityScore::high();
        let timeout = TimeoutDuration::medium();
        let delay = DelayDuration::short();
        let flag = BooleanFlag::enabled();
        let mut optional_vec = OptionalVec::new();
        optional_vec.push("test");

        // Verify all types are properly created
        assert!(!url.is_empty());
        assert!(!domain.is_empty());
        assert!(!session_id.is_empty());
        assert!(!task_id.is_empty());
        assert!(!count.is_zero());
        assert!(!limit.is_exceeded(5));
        assert!(!percentage.is_zero());
        assert!(!attempts.is_zero());
        assert!(priority.is_high());
        assert!(!timeout.is_zero());
        assert!(!delay.is_zero());
        assert!(flag.is_true());
        assert!(!optional_vec.is_empty());
    }

    #[test]
    fn test_primitive_type_interactions() {
        // Test how primitive types interact with each other
        let count = CountValue::new(8);
        let limit = LimitValue::new(10);
        let percentage = PercentageValue::new(80.0);

        // Check limit interactions
        assert!(!limit.is_exceeded_by_count(count));
        assert_eq!(limit.remaining_for_count(count), 2);

        // Check percentage interactions
        let percentage_of_count = percentage.apply_to_count(count);
        assert_eq!(percentage_of_count, 6.4);

        // Check temporal interactions
        let base_delay = DelayDuration::short();
        let attempts = AttemptCount::new(2);
        let max_delay = DelayDuration::long();
        let backoff_delay =
            DelayDuration::exponential_backoff(base_delay, attempts.value(), max_delay);
        assert!(backoff_delay.as_millis() > base_delay.as_millis());
    }

    #[test]
    fn test_primitive_type_serialization() {
        // Test that all types are serializable (basic smoke test)
        let url = UrlString::new("https://test.com");
        let serialized = serde_json::to_string(&url).expect("Should serialize");
        let deserialized: UrlString =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(url, deserialized);

        let count = CountValue::new(42);
        let serialized = serde_json::to_string(&count).expect("Should serialize");
        let deserialized: CountValue =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(count, deserialized);
    }

    #[test]
    fn test_primitive_type_defaults() {
        // Test that all types have sensible defaults
        let url = UrlString::default();
        let domain = DomainString::default();
        let session_id = SessionId::default();
        let task_id = TaskId::default();
        let count = CountValue::default();
        let limit = LimitValue::default();
        let percentage = PercentageValue::default();
        let attempts = AttemptCount::default();
        let priority = PriorityScore::default();
        let timeout = TimeoutDuration::default();
        let delay = DelayDuration::default();
        let flag = BooleanFlag::default();
        let optional_vec = OptionalVec::<String>::default();

        // Verify defaults are in expected states
        assert!(url.is_empty());
        assert!(domain.is_empty());
        assert!(session_id.is_empty());
        assert!(task_id.is_empty());
        assert!(count.is_zero());
        assert_eq!(limit.value(), 0);
        assert!(percentage.is_zero());
        assert!(attempts.is_zero());
        assert!(priority.is_medium());
        assert_eq!(timeout.as_secs(), 30); // medium timeout
        assert_eq!(delay.as_millis(), 100); // short delay
        assert!(flag.is_false());
        assert!(optional_vec.is_none());
    }
}
