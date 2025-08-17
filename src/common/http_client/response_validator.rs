// Level 3: ResponseValidator
use std::collections::HashMap;

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
    pub fn with_status_codes(mut self, codes: Vec<u16>) -> Self {
        self.required_status_codes = codes;
        self
    }
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.required_headers = headers;
        self
    }
    pub fn with_content_patterns(mut self, patterns: Vec<String>) -> Self {
        self.required_content_patterns = patterns;
        self
    }
    pub fn validate_status(&self, status: u16) -> bool {
        self.required_status_codes.contains(&status)
    }
    pub fn validate_headers(&self, headers: &HashMap<String, String>) -> bool {
        self.required_headers
            .iter()
            .all(|header| headers.contains_key(header))
    }
    pub fn validate_content(&self, content: &str) -> bool {
        if self.required_content_patterns.is_empty() {
            return true;
        }
        self.required_content_patterns
            .iter()
            .all(|pattern| content.contains(pattern))
    }
}

impl Default for ResponseValidator {
    fn default() -> Self {
        Self::new()
    }
}
