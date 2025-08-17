/// Flag primitive building blocks for boolean state management
/// Level 3 implementation - complete struct and functionality for flag types
use serde::{Deserialize, Serialize};

/// Building block for boolean flags - ensures consistent boolean state handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct BooleanFlag(bool);

impl BooleanFlag {
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    pub fn value(&self) -> bool {
        self.0
    }

    pub fn is_true(&self) -> bool {
        self.0
    }

    pub fn is_false(&self) -> bool {
        !self.0
    }

    pub fn is_enabled(&self) -> bool {
        self.0
    }

    pub fn is_disabled(&self) -> bool {
        !self.0
    }

    pub fn set(&mut self, value: bool) {
        self.0 = value;
    }

    pub fn enable(&mut self) {
        self.0 = true;
    }

    pub fn disable(&mut self) {
        self.0 = false;
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    pub fn toggled(&self) -> Self {
        Self(!self.0)
    }

    /// Create an enabled flag (true)
    pub fn enabled() -> Self {
        Self(true)
    }

    /// Create a disabled flag (false)
    pub fn disabled() -> Self {
        Self(false)
    }

    /// Logical AND operation
    pub fn and(&self, other: Self) -> Self {
        Self(self.0 && other.0)
    }

    /// Logical OR operation
    pub fn or(&self, other: Self) -> Self {
        Self(self.0 || other.0)
    }

    /// Logical XOR operation
    pub fn xor(&self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    /// Logical NOT operation
    pub fn not(&self) -> Self {
        Self(!self.0)
    }

    /// Check if all flags in a slice are true
    pub fn all_true(flags: &[Self]) -> bool {
        flags.iter().all(|flag| flag.is_true())
    }

    /// Check if any flag in a slice is true
    pub fn any_true(flags: &[Self]) -> bool {
        flags.iter().any(|flag| flag.is_true())
    }

    /// Count the number of true flags in a slice
    pub fn count_true(flags: &[Self]) -> usize {
        flags.iter().filter(|flag| flag.is_true()).count()
    }

    /// Create a flag from a string value (case-insensitive)
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" | "enabled" => Some(Self(true)),
            "false" | "no" | "off" | "0" | "disabled" => Some(Self(false)),
            _ => None,
        }
    }

    /// Convert to a human-readable string
    pub fn to_string(&self) -> String {
        if self.0 {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        }
    }
}

impl From<bool> for BooleanFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<BooleanFlag> for bool {
    fn from(flag: BooleanFlag) -> Self {
        flag.0
    }
}

impl std::ops::BitAnd for BooleanFlag {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        self.and(other)
    }
}

impl std::ops::BitOr for BooleanFlag {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        self.or(other)
    }
}

impl std::ops::BitXor for BooleanFlag {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        self.xor(other)
    }
}

impl std::ops::Not for BooleanFlag {
    type Output = Self;

    fn not(self) -> Self {
        self.not()
    }
}

impl std::fmt::Display for BooleanFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_flag_creation() {
        let flag_true = BooleanFlag::new(true);
        let flag_false = BooleanFlag::new(false);

        assert!(flag_true.is_true());
        assert!(!flag_true.is_false());
        assert!(!flag_false.is_true());
        assert!(flag_false.is_false());
    }

    #[test]
    fn test_boolean_flag_presets() {
        let enabled = BooleanFlag::enabled();
        let disabled = BooleanFlag::disabled();
        let default = BooleanFlag::default();

        assert!(enabled.is_true());
        assert!(disabled.is_false());
        assert!(default.is_false()); // Default is false
    }

    #[test]
    fn test_boolean_flag_mutations() {
        let mut flag = BooleanFlag::new(false);

        flag.enable();
        assert!(flag.is_true());

        flag.disable();
        assert!(flag.is_false());

        flag.set(true);
        assert!(flag.is_true());

        flag.toggle();
        assert!(flag.is_false());

        flag.toggle();
        assert!(flag.is_true());
    }

    #[test]
    fn test_boolean_flag_immutable_operations() {
        let flag = BooleanFlag::new(true);

        let toggled = flag.toggled();
        assert!(flag.is_true()); // Original unchanged
        assert!(toggled.is_false());
    }

    #[test]
    fn test_boolean_flag_logical_operations() {
        let true_flag = BooleanFlag::new(true);
        let false_flag = BooleanFlag::new(false);

        // AND operations
        assert!(true_flag.and(true_flag).is_true());
        assert!(true_flag.and(false_flag).is_false());
        assert!(false_flag.and(false_flag).is_false());

        // OR operations
        assert!(true_flag.or(true_flag).is_true());
        assert!(true_flag.or(false_flag).is_true());
        assert!(false_flag.or(false_flag).is_false());

        // XOR operations
        assert!(true_flag.xor(true_flag).is_false());
        assert!(true_flag.xor(false_flag).is_true());
        assert!(false_flag.xor(false_flag).is_false());

        // NOT operations
        assert!(true_flag.not().is_false());
        assert!(false_flag.not().is_true());
    }

    #[test]
    fn test_boolean_flag_bitwise_operators() {
        let true_flag = BooleanFlag::new(true);
        let false_flag = BooleanFlag::new(false);

        // Bitwise AND
        assert!((true_flag & true_flag).is_true());
        assert!((true_flag & false_flag).is_false());

        // Bitwise OR
        assert!((true_flag | false_flag).is_true());
        assert!((false_flag | false_flag).is_false());

        // Bitwise XOR
        assert!((true_flag ^ false_flag).is_true());
        assert!((true_flag ^ true_flag).is_false());

        // Bitwise NOT
        assert!((!true_flag).is_false());
        assert!((!false_flag).is_true());
    }

    #[test]
    fn test_boolean_flag_collections() {
        let flags = vec![
            BooleanFlag::new(true),
            BooleanFlag::new(true),
            BooleanFlag::new(false),
        ];

        assert!(!BooleanFlag::all_true(&flags));
        assert!(BooleanFlag::any_true(&flags));
        assert_eq!(BooleanFlag::count_true(&flags), 2);

        let all_true_flags = vec![BooleanFlag::new(true), BooleanFlag::new(true)];

        assert!(BooleanFlag::all_true(&all_true_flags));

        let all_false_flags = vec![BooleanFlag::new(false), BooleanFlag::new(false)];

        assert!(!BooleanFlag::any_true(&all_false_flags));
        assert_eq!(BooleanFlag::count_true(&all_false_flags), 0);
    }

    #[test]
    fn test_boolean_flag_string_parsing() {
        // Positive cases
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("true"),
            Some(BooleanFlag::new(true))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("TRUE"),
            Some(BooleanFlag::new(true))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("yes"),
            Some(BooleanFlag::new(true))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("on"),
            Some(BooleanFlag::new(true))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("1"),
            Some(BooleanFlag::new(true))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("enabled"),
            Some(BooleanFlag::new(true))
        );

        // Negative cases
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("false"),
            Some(BooleanFlag::new(false))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("FALSE"),
            Some(BooleanFlag::new(false))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("no"),
            Some(BooleanFlag::new(false))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("off"),
            Some(BooleanFlag::new(false))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("0"),
            Some(BooleanFlag::new(false))
        );
        assert_eq!(
            BooleanFlag::from_str_case_insensitive("disabled"),
            Some(BooleanFlag::new(false))
        );

        // Invalid cases
        assert_eq!(BooleanFlag::from_str_case_insensitive("maybe"), None);
        assert_eq!(BooleanFlag::from_str_case_insensitive("invalid"), None);
        assert_eq!(BooleanFlag::from_str_case_insensitive(""), None);
    }

    #[test]
    fn test_boolean_flag_display() {
        let true_flag = BooleanFlag::new(true);
        let false_flag = BooleanFlag::new(false);

        assert_eq!(true_flag.to_string(), "enabled");
        assert_eq!(false_flag.to_string(), "disabled");

        assert_eq!(format!("{}", true_flag), "enabled");
        assert_eq!(format!("{}", false_flag), "disabled");
    }

    #[test]
    fn test_boolean_flag_conversions() {
        let bool_true = true;
        let bool_false = false;

        let flag_from_true = BooleanFlag::from(bool_true);
        let flag_from_false = BooleanFlag::from(bool_false);

        assert!(flag_from_true.is_true());
        assert!(flag_from_false.is_false());

        let back_to_bool_true: bool = flag_from_true.into();
        let back_to_bool_false: bool = flag_from_false.into();

        assert_eq!(back_to_bool_true, true);
        assert_eq!(back_to_bool_false, false);
    }

    #[test]
    fn test_boolean_flag_value_access() {
        let flag = BooleanFlag::new(true);
        assert_eq!(flag.value(), true);

        let flag2 = BooleanFlag::new(false);
        assert_eq!(flag2.value(), false);
    }

    #[test]
    fn test_boolean_flag_empty_collection() {
        let empty_flags: Vec<BooleanFlag> = vec![];

        assert!(BooleanFlag::all_true(&empty_flags)); // Vacuous truth
        assert!(!BooleanFlag::any_true(&empty_flags));
        assert_eq!(BooleanFlag::count_true(&empty_flags), 0);
    }
}
