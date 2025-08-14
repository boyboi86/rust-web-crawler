use crate::core::error::CrawlError;
/// Configuration for keyword-based content filtering
use serde::{Deserialize, Serialize};

/// Keyword matching mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeywordMode {
    /// Match any of the keywords (OR operation)
    Any,
    /// Match all keywords (AND operation)  
    All,
    /// Match exact phrases
    Exact,
    /// Case-insensitive matching
    CaseInsensitive,
    /// Regular expression matching
    Regex,
}

impl Default for KeywordMode {
    fn default() -> Self {
        KeywordMode::Any
    }
}

/// Additional options for keyword processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordOptions {
    /// Minimum number of keyword matches required
    pub min_matches: Option<usize>,
    /// Maximum distance between keywords for proximity matching
    pub proximity_distance: Option<usize>,
    /// Include surrounding context around matches
    pub include_context: bool,
    /// Context window size (characters before and after match)
    pub context_window: usize,
    /// Highlight matched keywords in results
    pub highlight_matches: bool,
}

impl Default for KeywordOptions {
    fn default() -> Self {
        Self {
            min_matches: None,
            proximity_distance: None,
            include_context: false,
            context_window: 100,
            highlight_matches: false,
        }
    }
}

/// Configuration for keyword-based content filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordConfig {
    /// Enable keyword filtering
    pub enabled: bool,
    /// Target keywords to search for
    pub keywords: Vec<String>,
    /// Keyword matching mode
    pub mode: KeywordMode,
    /// Additional processing options
    pub options: KeywordOptions,
}

impl Default for KeywordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            keywords: Vec::new(),
            mode: KeywordMode::default(),
            options: KeywordOptions::default(),
        }
    }
}

impl KeywordConfig {
    /// Create a new keyword configuration
    pub fn new(keywords: Vec<String>, mode: KeywordMode) -> Self {
        Self {
            enabled: true,
            keywords,
            mode,
            options: KeywordOptions::default(),
        }
    }

    /// Create configuration with options
    pub fn with_options(keywords: Vec<String>, mode: KeywordMode, options: KeywordOptions) -> Self {
        Self {
            enabled: true,
            keywords,
            mode,
            options,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CrawlError> {
        if self.enabled && self.keywords.is_empty() {
            return Err(CrawlError::KeywordConfigError(
                "Keywords cannot be empty when keyword filtering is enabled".to_string(),
            ));
        }

        if self.enabled && self.keywords.iter().any(|k| k.trim().is_empty()) {
            return Err(CrawlError::KeywordConfigError(
                "Keywords cannot contain empty strings".to_string(),
            ));
        }

        // Validate regex patterns if using regex mode
        if self.mode == KeywordMode::Regex {
            for keyword in &self.keywords {
                if let Err(e) = regex::Regex::new(keyword) {
                    return Err(CrawlError::KeywordConfigError(format!(
                        "Invalid regex pattern '{}': {}",
                        keyword, e
                    )));
                }
            }
        }

        // Validate proximity distance
        if let Some(distance) = self.options.proximity_distance {
            if distance == 0 {
                return Err(CrawlError::KeywordConfigError(
                    "Proximity distance must be greater than 0".to_string(),
                ));
            }
        }

        // Validate context window
        if self.options.include_context && self.options.context_window == 0 {
            return Err(CrawlError::KeywordConfigError(
                "Context window must be greater than 0 when including context".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if keyword filtering should be performed
    pub fn should_filter(&self) -> bool {
        self.enabled && !self.keywords.is_empty()
    }

    /// Get normalized keywords based on mode
    pub fn normalized_keywords(&self) -> Vec<String> {
        match self.mode {
            KeywordMode::CaseInsensitive => {
                self.keywords.iter().map(|k| k.to_lowercase()).collect()
            }
            _ => self.keywords.clone(),
        }
    }
}
