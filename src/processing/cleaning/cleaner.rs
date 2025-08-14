/// Main text cleaner implementation
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::config::CleaningConfig;
use super::rules::{CleaningEngine, CleaningRule};
use crate::core::error::CrawlError;

/// Result of text cleaning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningResult {
    /// Original text
    pub original_text: String,
    /// Cleaned text
    pub cleaned_text: String,
    /// Cleaning statistics
    pub stats: CleaningStats,
    /// Whether any cleaning was performed
    pub was_cleaned: bool,
}

/// Statistics about text cleaning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningStats {
    /// Original character count
    pub original_chars: usize,
    /// Cleaned character count
    pub cleaned_chars: usize,
    /// Original word count
    pub original_words: usize,
    /// Cleaned word count
    pub cleaned_words: usize,
    /// Compression ratio (cleaned/original)
    pub compression_ratio: f64,
    /// Number of operations performed
    pub operations_performed: usize,
    /// Time taken for cleaning (milliseconds)
    pub processing_time_ms: u64,
}

impl Default for CleaningStats {
    fn default() -> Self {
        Self {
            original_chars: 0,
            cleaned_chars: 0,
            original_words: 0,
            cleaned_words: 0,
            compression_ratio: 1.0,
            operations_performed: 0,
            processing_time_ms: 0,
        }
    }
}

/// Main text cleaner
pub struct TextCleaner {
    config: CleaningConfig,
    custom_engine: Option<CleaningEngine>,
    word_patterns: Option<Vec<Regex>>,
}

impl TextCleaner {
    /// Create a new text cleaner
    pub fn new(config: CleaningConfig) -> Result<Self, CrawlError> {
        config.validate()?;

        // Compile custom rules if provided
        let custom_engine = if let Some(ref custom_rules) = config.custom_rules {
            let rules: Result<Vec<CleaningRule>, _> = custom_rules
                .iter()
                .enumerate()
                .map(|(i, rule_str)| {
                    // Parse custom rule string (simplified format: "remove:pattern" or "replace:pattern:replacement")
                    let parts: Vec<&str> = rule_str.split(':').collect();
                    match parts.len() {
                        2 if parts[0] == "remove" => Ok(CleaningRule::remove(&format!("custom_{}", i), parts[1])),
                        3 if parts[0] == "replace" => Ok(CleaningRule::replace(&format!("custom_{}", i), parts[1], parts[2])),
                        _ => Err(CrawlError::CleaningConfigError(
                            format!("Invalid custom rule format: '{}'. Use 'remove:pattern' or 'replace:pattern:replacement'", rule_str)
                        ))
                    }
                })
                .collect();

            Some(CleaningEngine::new(rules?)?)
        } else {
            None
        };

        // Compile word patterns
        let word_patterns = if config.word_filter.enabled {
            if let Some(ref patterns) = config.word_filter.remove_patterns {
                let compiled: Result<Vec<Regex>, _> =
                    patterns.iter().map(|pattern| Regex::new(pattern)).collect();
                Some(compiled.map_err(|e| {
                    CrawlError::CleaningConfigError(format!(
                        "Failed to compile word pattern: {}",
                        e
                    ))
                })?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            config,
            custom_engine,
            word_patterns,
        })
    }

    /// Clean text according to configuration
    pub fn clean_text(&self, text: &str) -> Result<CleaningResult, CrawlError> {
        if !self.config.should_clean() {
            return Ok(CleaningResult {
                original_text: text.to_string(),
                cleaned_text: text.to_string(),
                stats: CleaningStats::default(),
                was_cleaned: false,
            });
        }

        let start_time = std::time::Instant::now();
        let mut cleaned_text = text.to_string();
        let mut operations_count = 0;

        let original_chars = text.len();
        let original_words = text.split_whitespace().count();

        // Apply character filtering
        if self.config.character_filter.enabled {
            cleaned_text = self.apply_character_filter(&cleaned_text)?;
            operations_count += 1;
        }

        // Apply word filtering
        if self.config.word_filter.enabled {
            cleaned_text = self.apply_word_filter(&cleaned_text)?;
            operations_count += 1;
        }

        // Apply language filtering
        if self.config.language_filter.enabled {
            cleaned_text = self.apply_language_filter(&cleaned_text)?;
            operations_count += 1;
        }

        // Apply length filtering
        if self.config.length_filter.enabled {
            cleaned_text = self.apply_length_filter(&cleaned_text)?;
            operations_count += 1;
        }

        // Apply custom rules
        if let Some(ref engine) = self.custom_engine {
            cleaned_text = engine.apply_rules(&cleaned_text);
            operations_count += 1;
        }

        // Final whitespace normalization
        if self.config.character_filter.normalize_whitespace {
            cleaned_text = Regex::new(r"\s+")
                .unwrap()
                .replace_all(&cleaned_text, " ")
                .trim()
                .to_string();
        }

        // Check if result is empty and not allowed
        if cleaned_text.trim().is_empty() && !self.config.allow_empty_result {
            return Err(CrawlError::CleaningRuleError(
                "Text cleaning resulted in empty content".to_string(),
            ));
        }

        let cleaned_chars = cleaned_text.len();
        let cleaned_words = cleaned_text.split_whitespace().count();
        let compression_ratio = if original_chars > 0 {
            cleaned_chars as f64 / original_chars as f64
        } else {
            1.0
        };

        let processing_time = start_time.elapsed().as_millis() as u64;

        Ok(CleaningResult {
            original_text: text.to_string(),
            cleaned_text,
            stats: CleaningStats {
                original_chars,
                cleaned_chars,
                original_words,
                cleaned_words,
                compression_ratio,
                operations_performed: operations_count,
                processing_time_ms: processing_time,
            },
            was_cleaned: true,
        })
    }

    /// Apply character-based filtering
    fn apply_character_filter(&self, text: &str) -> Result<String, CrawlError> {
        let filter = &self.config.character_filter;
        let mut result = text.to_string();

        // Remove specific characters
        if let Some(ref chars_to_remove) = filter.remove_characters {
            result = result
                .chars()
                .filter(|c| !chars_to_remove.contains(c))
                .collect();
        }

        // Remove Unicode ranges
        if let Some(ref ranges) = filter.remove_unicode_ranges {
            result = result
                .chars()
                .filter(|c| {
                    let code_point = *c as u32;
                    !ranges
                        .iter()
                        .any(|(start, end)| code_point >= *start && code_point <= *end)
                })
                .collect();
        }

        // ASCII only
        if filter.ascii_only {
            result = result.chars().filter(|c| c.is_ascii()).collect();
        }

        // Alphanumeric only
        if filter.alphanumeric_only {
            result = result
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
        }

        // Remove line breaks
        if filter.remove_line_breaks {
            result = result.replace('\n', " ").replace('\r', " ");
        }

        Ok(result)
    }

    /// Apply word-based filtering
    fn apply_word_filter(&self, text: &str) -> Result<String, CrawlError> {
        let filter = &self.config.word_filter;
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut filtered_words = Vec::new();

        for word in words {
            let mut keep_word = true;
            let word_to_check = if filter.case_sensitive {
                word.to_string()
            } else {
                word.to_lowercase()
            };

            // Check against remove words list
            if let Some(ref remove_words) = filter.remove_words {
                let words_set: HashSet<String> = if filter.case_sensitive {
                    remove_words.clone()
                } else {
                    remove_words.iter().map(|w| w.to_lowercase()).collect()
                };

                if words_set.contains(&word_to_check) {
                    keep_word = false;
                }
            }

            // Check against patterns
            if keep_word && let Some(ref patterns) = self.word_patterns {
                if patterns.iter().any(|pattern| pattern.is_match(word)) {
                    keep_word = false;
                }
            }

            // Remove numeric words
            if keep_word && filter.remove_numeric_words {
                if word.chars().any(|c| c.is_ascii_digit()) {
                    keep_word = false;
                }
            }

            // Remove words with special characters
            if keep_word && filter.remove_special_char_words {
                if word.chars().any(|c| !c.is_alphanumeric()) {
                    keep_word = false;
                }
            }

            if keep_word {
                filtered_words.push(word);
            }
        }

        Ok(filtered_words.join(" "))
    }

    /// Apply language-specific filtering
    fn apply_language_filter(&self, text: &str) -> Result<String, CrawlError> {
        let filter = &self.config.language_filter;
        let mut result = text.to_string();

        if filter.remove_chinese {
            result = Regex::new(r"[\u4e00-\u9fff]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        if filter.remove_japanese {
            // Remove Hiragana
            result = Regex::new(r"[\u3040-\u309f]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
            // Remove Katakana
            result = Regex::new(r"[\u30a0-\u30ff]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        if filter.remove_korean {
            result = Regex::new(r"[\uac00-\ud7af]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        if filter.remove_arabic {
            result = Regex::new(r"[\u0600-\u06ff]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        if filter.remove_cyrillic {
            result = Regex::new(r"[\u0400-\u04ff]+")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        Ok(result)
    }

    /// Apply length-based filtering
    fn apply_length_filter(&self, text: &str) -> Result<String, CrawlError> {
        let filter = &self.config.length_filter;

        // Filter words by length
        let words: Vec<&str> = text.split_whitespace().collect();
        let filtered_words: Vec<&str> = words
            .into_iter()
            .filter(|word| {
                let len = word.len();

                if let Some(min_len) = filter.min_word_length {
                    if len < min_len {
                        return false;
                    }
                }

                if let Some(max_len) = filter.max_word_length {
                    if len > max_len {
                        return false;
                    }
                }

                true
            })
            .collect();

        let result = filtered_words.join(" ");

        // Check sentence length (simplified: split by periods)
        if filter.min_sentence_length.is_some() || filter.max_sentence_length.is_some() {
            let sentences: Vec<&str> = result.split('.').collect();
            let filtered_sentences: Vec<&str> = sentences
                .into_iter()
                .filter(|sentence| {
                    let len = sentence.trim().len();

                    if let Some(min_len) = filter.min_sentence_length {
                        if len < min_len {
                            return false;
                        }
                    }

                    if let Some(max_len) = filter.max_sentence_length {
                        if len > max_len {
                            return false;
                        }
                    }

                    true
                })
                .collect();

            return Ok(filtered_sentences.join("."));
        }

        Ok(result)
    }
}
