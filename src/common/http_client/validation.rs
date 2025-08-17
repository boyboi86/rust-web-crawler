//! Response validation building blocks for HTTP operations
//! Level 3 implementation: ResponseValidator

use std::collections::HashMap;

/// ResponseValidator - building block for validating API responses
#[derive(Debug, Clone)]
pub struct ResponseValidator {
    required_status_codes: Vec<u16>,
    required_headers: Vec<String>,
    required_content_patterns: Vec<String>,
}

impl ResponseValidator {
    pub fn new() -> Self {
        Self {
            required_status_codes: vec![200],
            required_headers: Vec::new(),
            required_content_patterns: Vec::new(),
        }
    }

    pub fn success_only() -> Self {
        Self {
            required_status_codes: vec![200],
            required_headers: Vec::new(),
            required_content_patterns: Vec::new(),
        }
    }

    pub fn any_success() -> Self {
        Self {
            required_status_codes: vec![200, 201, 202, 204],
            required_headers: Vec::new(),
            required_content_patterns: Vec::new(),
        }
    }

    pub fn permissive() -> Self {
        Self {
            required_status_codes: (200..300).collect(), // All 2xx codes
            required_headers: Vec::new(),
            required_content_patterns: Vec::new(),
        }
    }

    pub fn with_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.required_status_codes = codes;
        self
    }

    pub fn with_status_code(mut self, code: u16) -> Self {
        self.required_status_codes = vec![code];
        self
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.required_headers = headers;
        self
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.required_headers.push(header.into());
        self
    }

    pub fn with_content_patterns(mut self, patterns: Vec<String>) -> Self {
        self.required_content_patterns = patterns;
        self
    }

    pub fn with_content_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.required_content_patterns.push(pattern.into());
        self
    }

    // Validation methods
    pub fn validate_status(&self, status: u16) -> bool {
        if self.required_status_codes.is_empty() {
            return true; // No requirements means any status is valid
        }
        self.required_status_codes.contains(&status)
    }

    pub fn validate_headers(&self, headers: &HashMap<String, String>) -> bool {
        if self.required_headers.is_empty() {
            return true; // No requirements means any headers are valid
        }
        
        self.required_headers
            .iter()
            .all(|header| headers.contains_key(header))
    }

    pub fn validate_content(&self, content: &str) -> bool {
        if self.required_content_patterns.is_empty() {
            return true; // No requirements means any content is valid
        }

        self.required_content_patterns
            .iter()
            .all(|pattern| content.contains(pattern))
    }

    pub fn validate_all(&self, status: u16, headers: &HashMap<String, String>, content: &str) -> bool {
        self.validate_status(status) 
            && self.validate_headers(headers) 
            && self.validate_content(content)
    }

    // Getters for inspection
    pub fn required_status_codes(&self) -> &[u16] {
        &self.required_status_codes
    }

    pub fn required_headers(&self) -> &[String] {
        &self.required_headers
    }

    pub fn required_content_patterns(&self) -> &[String] {
        &self.required_content_patterns
    }

    // Helper methods
    pub fn is_permissive(&self) -> bool {
        self.required_status_codes.is_empty() 
            && self.required_headers.is_empty() 
            && self.required_content_patterns.is_empty()
    }

    pub fn clear_requirements(&mut self) {
        self.required_status_codes.clear();
        self.required_headers.clear();
        self.required_content_patterns.clear();
    }
}

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_validator_defaults() {
        let validator = ResponseValidator::new();
        assert_eq!(validator.required_status_codes(), &[200]);
        assert!(validator.required_headers().is_empty());
        assert!(validator.required_content_patterns().is_empty());
    }

    #[test]
    fn test_response_validator_presets() {
        let success_only = ResponseValidator::success_only();
        assert_eq!(success_only.required_status_codes(), &[200]);

        let any_success = ResponseValidator::any_success();
        assert_eq!(any_success.required_status_codes(), &[200, 201, 202, 204]);

        let permissive = ResponseValidator::permissive();
        assert_eq!(permissive.required_status_codes().len(), 100); // 200-299
    }

    #[test]
    fn test_response_validator_builder() {
        let validator = ResponseValidator::new()
            .with_status_codes(vec![200, 201])
            .with_header("Content-Type")
            .with_content_pattern("success");

        assert_eq!(validator.required_status_codes(), &[200, 201]);
        assert_eq!(validator.required_headers(), &["Content-Type"]);
        assert_eq!(validator.required_content_patterns(), &["success"]);
    }

    #[test]
    fn test_validate_status() {
        let validator = ResponseValidator::new().with_status_codes(vec![200, 201]);
        
        assert!(validator.validate_status(200));
        assert!(validator.validate_status(201));
        assert!(!validator.validate_status(404));
        assert!(!validator.validate_status(500));
    }

    #[test]
    fn test_validate_headers() {
        let validator = ResponseValidator::new()
            .with_headers(vec!["Content-Type".to_string(), "Authorization".to_string()]);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        headers.insert("Extra-Header".to_string(), "extra".to_string());

        assert!(validator.validate_headers(&headers));

        // Missing required header
        headers.remove("Authorization");
        assert!(!validator.validate_headers(&headers));
    }

    #[test]
    fn test_validate_content() {
        let validator = ResponseValidator::new()
            .with_content_patterns(vec!["success".to_string(), "data".to_string()]);

        let content = r#"{"status": "success", "data": {"id": 1}}"#;
        assert!(validator.validate_content(content));

        let missing_pattern = r#"{"status": "error", "data": {"id": 1}}"#;
        assert!(!validator.validate_content(missing_pattern));
    }

    #[test]
    fn test_validate_all() {
        let validator = ResponseValidator::new()
            .with_status_code(200)
            .with_header("Content-Type")
            .with_content_pattern("success");

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let content = r#"{"status": "success"}"#;

        assert!(validator.validate_all(200, &headers, content));
        assert!(!validator.validate_all(404, &headers, content)); // Wrong status
        assert!(!validator.validate_all(200, &HashMap::new(), content)); // Missing header
        assert!(!validator.validate_all(200, &headers, "error")); // Wrong content
    }

    #[test]
    fn test_empty_requirements() {
        let validator = ResponseValidator::new()
            .with_status_codes(vec![])
            .with_headers(vec![])
            .with_content_patterns(vec![]);

        // Empty requirements should validate everything
        assert!(validator.validate_status(404));
        assert!(validator.validate_headers(&HashMap::new()));
        assert!(validator.validate_content("any content"));
        assert!(validator.is_permissive());
    }

    #[test]
    fn test_clear_requirements() {
        let mut validator = ResponseValidator::new()
            .with_status_code(200)
            .with_header("Content-Type")
            .with_content_pattern("success");

        assert!(!validator.is_permissive());
        
        validator.clear_requirements();
        assert!(validator.is_permissive());
        assert!(validator.required_status_codes().is_empty());
        assert!(validator.required_headers().is_empty());
        assert!(validator.required_content_patterns().is_empty());
    }
}
