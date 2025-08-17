use crate::common::primitives::{BooleanFlag, LimitValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilterConfig {
    enabled: BooleanFlag,
    min_content_length: LimitValue,
    max_content_length: LimitValue,
    allowed_content_types: Vec<String>,
    blocked_extensions: Vec<String>,
    keyword_filtering: BooleanFlag,
}

impl ContentFilterConfig {
    pub fn new(
        enabled: BooleanFlag,
        min_content_length: LimitValue,
        max_content_length: LimitValue,
        allowed_content_types: Vec<String>,
        blocked_extensions: Vec<String>,
        keyword_filtering: BooleanFlag,
    ) -> Self {
        Self {
            enabled,
            min_content_length,
            max_content_length,
            allowed_content_types,
            blocked_extensions,
            keyword_filtering,
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.is_enabled()
    }
    pub fn min_content_length(&self) -> u64 {
        self.min_content_length.value()
    }
    pub fn max_content_length(&self) -> u64 {
        self.max_content_length.value()
    }
    pub fn allowed_content_types(&self) -> &[String] {
        &self.allowed_content_types
    }
    pub fn blocked_extensions(&self) -> &[String] {
        &self.blocked_extensions
    }
    pub fn is_keyword_filtering_enabled(&self) -> bool {
        self.keyword_filtering.is_enabled()
    }
    pub fn is_content_type_allowed(&self, content_type: &str) -> bool {
        if self.allowed_content_types.is_empty() {
            true
        } else {
            self.allowed_content_types
                .iter()
                .any(|allowed| content_type.contains(allowed))
        }
    }
    pub fn is_extension_blocked(&self, extension: &str) -> bool {
        self.blocked_extensions
            .iter()
            .any(|blocked| extension.eq_ignore_ascii_case(blocked))
    }
}

impl Default for ContentFilterConfig {
    fn default() -> Self {
        Self {
            enabled: BooleanFlag::enabled(),
            min_content_length: LimitValue::new(100),
            max_content_length: LimitValue::new(10_000_000),
            allowed_content_types: vec![
                "text/html".to_string(),
                "text/plain".to_string(),
                "application/json".to_string(),
            ],
            blocked_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
            ],
            keyword_filtering: BooleanFlag::disabled(),
        }
    }
}
