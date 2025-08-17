#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

/// String primitive building blocks for consistent URL and domain handling
/// Level 3 implementation - complete struct and functionality for string types
use serde::{Deserialize, Serialize};

/// Building block for URL string - ensures consistent URL handling
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct UrlString(String);

impl UrlString {
    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<String> for UrlString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UrlString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for UrlString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Building block for domain string - ensures consistent domain handling
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct DomainString(String);

impl DomainString {
    pub fn new(domain: impl Into<String>) -> Self {
        Self(domain.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_lowercase(&self) -> Self {
        Self(self.0.to_lowercase())
    }
}

impl From<String> for DomainString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DomainString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for DomainString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_string_creation() {
        let url = UrlString::new("https://example.com");
        assert_eq!(url.as_str(), "https://example.com");
        assert!(!url.is_empty());
        assert_eq!(url.len(), 19);
    }

    #[test]
    fn test_url_string_from_string() {
        let url = UrlString::from("https://test.com".to_string());
        assert_eq!(url.as_str(), "https://test.com");
    }

    #[test]
    fn test_url_string_from_str() {
        let url = UrlString::from("https://test.com");
        assert_eq!(url.as_str(), "https://test.com");
    }

    #[test]
    fn test_url_string_display() {
        let url = UrlString::new("https://example.com");
        assert_eq!(format!("{}", url), "https://example.com");
    }

    #[test]
    fn test_domain_string_creation() {
        let domain = DomainString::new("example.com");
        assert_eq!(domain.as_str(), "example.com");
        assert!(!domain.is_empty());
        assert_eq!(domain.len(), 11);
    }

    #[test]
    fn test_domain_string_lowercase() {
        let domain = DomainString::new("EXAMPLE.COM");
        let lower = domain.to_lowercase();
        assert_eq!(lower.as_str(), "example.com");
    }

    #[test]
    fn test_domain_string_from_string() {
        let domain = DomainString::from("test.com".to_string());
        assert_eq!(domain.as_str(), "test.com");
    }

    #[test]
    fn test_domain_string_from_str() {
        let domain = DomainString::from("test.com");
        assert_eq!(domain.as_str(), "test.com");
    }

    #[test]
    fn test_domain_string_display() {
        let domain = DomainString::new("example.com");
        assert_eq!(format!("{}", domain), "example.com");
    }

    #[test]
    fn test_empty_strings() {
        let empty_url = UrlString::default();
        let empty_domain = DomainString::default();

        assert!(empty_url.is_empty());
        assert!(empty_domain.is_empty());
        assert_eq!(empty_url.len(), 0);
        assert_eq!(empty_domain.len(), 0);
    }
}
