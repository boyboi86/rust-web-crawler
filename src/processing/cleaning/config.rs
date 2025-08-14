/// Configuration for text cleaning
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::core::error::CrawlError;

/// Length-based filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LengthFilter {
    /// Enable length filtering
    pub enabled: bool,
    /// Minimum word length to keep
    pub min_word_length: Option<usize>,
    /// Maximum word length to keep
    pub max_word_length: Option<usize>,
    /// Minimum sentence length to keep
    pub min_sentence_length: Option<usize>,
    /// Maximum sentence length to keep
    pub max_sentence_length: Option<usize>,
    /// Minimum paragraph length to keep
    pub min_paragraph_length: Option<usize>,
    /// Maximum paragraph length to keep
    pub max_paragraph_length: Option<usize>,
}

impl Default for LengthFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            min_word_length: None,
            max_word_length: None,
            min_sentence_length: None,
            max_sentence_length: None,
            min_paragraph_length: None,
            max_paragraph_length: None,
        }
    }
}

/// Character-based filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterFilter {
    /// Enable character filtering
    pub enabled: bool,
    /// Characters to remove
    pub remove_characters: Option<HashSet<char>>,
    /// Character ranges to remove (e.g., emoji ranges)
    pub remove_unicode_ranges: Option<Vec<(u32, u32)>>,
    /// Keep only alphanumeric characters
    pub alphanumeric_only: bool,
    /// Keep only ASCII characters
    pub ascii_only: bool,
    /// Remove extra whitespace
    pub normalize_whitespace: bool,
    /// Remove line breaks
    pub remove_line_breaks: bool,
}

impl Default for CharacterFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            remove_characters: None,
            remove_unicode_ranges: None,
            alphanumeric_only: false,
            ascii_only: false,
            normalize_whitespace: true,
            remove_line_breaks: false,
        }
    }
}

/// Word-based filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordFilter {
    /// Enable word filtering
    pub enabled: bool,
    /// Specific words to remove
    pub remove_words: Option<HashSet<String>>,
    /// Word patterns to remove (regex)
    pub remove_patterns: Option<Vec<String>>,
    /// Case-sensitive word matching
    pub case_sensitive: bool,
    /// Remove stop words for specific languages
    pub remove_stop_words: Option<Vec<String>>,
    /// Remove words containing numbers
    pub remove_numeric_words: bool,
    /// Remove words with special characters
    pub remove_special_char_words: bool,
}

impl Default for WordFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            remove_words: None,
            remove_patterns: None,
            case_sensitive: false,
            remove_stop_words: None,
            remove_numeric_words: false,
            remove_special_char_words: false,
        }
    }
}

/// Language-specific filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFilter {
    /// Enable language filtering
    pub enabled: bool,
    /// Remove Chinese characters
    pub remove_chinese: bool,
    /// Remove Japanese characters (Hiragana, Katakana, Kanji)
    pub remove_japanese: bool,
    /// Remove Korean characters (Hangul)
    pub remove_korean: bool,
    /// Remove Arabic characters
    pub remove_arabic: bool,
    /// Remove Cyrillic characters
    pub remove_cyrillic: bool,
    /// Keep only specific language scripts
    pub keep_only_scripts: Option<Vec<String>>,
    /// Custom Unicode blocks to remove
    pub remove_unicode_blocks: Option<Vec<String>>,
}

impl Default for LanguageFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            remove_chinese: false,
            remove_japanese: false,
            remove_korean: false,
            remove_arabic: false,
            remove_cyrillic: false,
            keep_only_scripts: None,
            remove_unicode_blocks: None,
        }
    }
}

/// Main text cleaning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningConfig {
    /// Enable text cleaning
    pub enabled: bool,
    /// Length-based filtering
    pub length_filter: LengthFilter,
    /// Character-based filtering
    pub character_filter: CharacterFilter,
    /// Word-based filtering
    pub word_filter: WordFilter,
    /// Language-specific filtering
    pub language_filter: LanguageFilter,
    /// Custom cleaning rules
    pub custom_rules: Option<Vec<String>>,
    /// Preserve original formatting
    pub preserve_formatting: bool,
    /// Output empty result if all content removed
    pub allow_empty_result: bool,
}

impl Default for CleaningConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            length_filter: LengthFilter::default(),
            character_filter: CharacterFilter::default(),
            word_filter: WordFilter::default(),
            language_filter: LanguageFilter::default(),
            custom_rules: None,
            preserve_formatting: true,
            allow_empty_result: false,
        }
    }
}

impl CleaningConfig {
    /// Create a new cleaning configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable basic text cleaning (whitespace normalization)
    pub fn basic() -> Self {
        Self {
            enabled: true,
            character_filter: CharacterFilter {
                enabled: true,
                normalize_whitespace: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Enable aggressive cleaning (remove all non-ASCII)
    pub fn aggressive() -> Self {
        Self {
            enabled: true,
            character_filter: CharacterFilter {
                enabled: true,
                ascii_only: true,
                normalize_whitespace: true,
                remove_line_breaks: true,
                ..Default::default()
            },
            word_filter: WordFilter {
                enabled: true,
                remove_numeric_words: true,
                remove_special_char_words: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Enable CJK (Chinese, Japanese, Korean) removal
    pub fn remove_cjk() -> Self {
        Self {
            enabled: true,
            language_filter: LanguageFilter {
                enabled: true,
                remove_chinese: true,
                remove_japanese: true,
                remove_korean: true,
                ..Default::default()
            },
            character_filter: CharacterFilter {
                enabled: true,
                normalize_whitespace: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), CrawlError> {
        if !self.enabled {
            return Ok(());
        }

        // Validate length filters
        if self.length_filter.enabled {
            if let (Some(min), Some(max)) = (
                self.length_filter.min_word_length,
                self.length_filter.max_word_length,
            ) {
                if min > max {
                    return Err(CrawlError::CleaningConfigError(
                        "Minimum word length cannot be greater than maximum word length"
                            .to_string(),
                    ));
                }
            }

            if let (Some(min), Some(max)) = (
                self.length_filter.min_sentence_length,
                self.length_filter.max_sentence_length,
            ) {
                if min > max {
                    return Err(CrawlError::CleaningConfigError(
                        "Minimum sentence length cannot be greater than maximum sentence length"
                            .to_string(),
                    ));
                }
            }

            if let (Some(min), Some(max)) = (
                self.length_filter.min_paragraph_length,
                self.length_filter.max_paragraph_length,
            ) {
                if min > max {
                    return Err(CrawlError::CleaningConfigError(
                        "Minimum paragraph length cannot be greater than maximum paragraph length"
                            .to_string(),
                    ));
                }
            }
        }

        // Validate word filter patterns
        if self.word_filter.enabled {
            if let Some(ref patterns) = self.word_filter.remove_patterns {
                for pattern in patterns {
                    if let Err(e) = regex::Regex::new(pattern) {
                        return Err(CrawlError::CleaningConfigError(format!(
                            "Invalid word pattern '{}': {}",
                            pattern, e
                        )));
                    }
                }
            }
        }

        // Validate Unicode ranges
        if self.character_filter.enabled {
            if let Some(ref ranges) = self.character_filter.remove_unicode_ranges {
                for (start, end) in ranges {
                    if start > end {
                        return Err(CrawlError::CleaningConfigError(format!(
                            "Invalid Unicode range: {} > {}",
                            start, end
                        )));
                    }
                }
            }
        }

        // Validate custom rules
        if let Some(ref rules) = self.custom_rules {
            for rule in rules {
                if rule.trim().is_empty() {
                    return Err(CrawlError::CleaningConfigError(
                        "Custom rules cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if any cleaning should be performed
    pub fn should_clean(&self) -> bool {
        self.enabled
            && (self.length_filter.enabled
                || self.character_filter.enabled
                || self.word_filter.enabled
                || self.language_filter.enabled
                || self.custom_rules.is_some())
    }

    /// Enable length filtering with specific constraints
    pub fn with_length_filter(mut self, min_word: Option<usize>, max_word: Option<usize>) -> Self {
        self.length_filter.enabled = true;
        self.length_filter.min_word_length = min_word;
        self.length_filter.max_word_length = max_word;
        self
    }

    /// Enable character filtering with specific options
    pub fn with_character_filter(mut self, ascii_only: bool, normalize_whitespace: bool) -> Self {
        self.character_filter.enabled = true;
        self.character_filter.ascii_only = ascii_only;
        self.character_filter.normalize_whitespace = normalize_whitespace;
        self
    }

    /// Enable word filtering with specific word list
    pub fn with_word_filter(mut self, remove_words: HashSet<String>, case_sensitive: bool) -> Self {
        self.word_filter.enabled = true;
        self.word_filter.remove_words = Some(remove_words);
        self.word_filter.case_sensitive = case_sensitive;
        self
    }

    /// Enable language filtering for specific scripts
    pub fn with_language_filter(mut self, remove_cjk: bool, remove_arabic: bool) -> Self {
        self.language_filter.enabled = true;
        if remove_cjk {
            self.language_filter.remove_chinese = true;
            self.language_filter.remove_japanese = true;
            self.language_filter.remove_korean = true;
        }
        self.language_filter.remove_arabic = remove_arabic;
        self
    }
}
