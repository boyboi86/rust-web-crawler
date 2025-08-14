/// Keyword matching functionality
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::config::{KeywordConfig, KeywordMode};
use crate::core::error::CrawlError;

/// Information about a keyword match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchInfo {
    /// The matched keyword
    pub keyword: String,
    /// Position in the text where match was found
    pub position: usize,
    /// Length of the match
    pub length: usize,
    /// Surrounding context (if enabled)
    pub context: Option<String>,
}

/// Result of keyword matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// Whether any keywords were found
    pub found: bool,
    /// Total number of matches
    pub match_count: usize,
    /// Details of each match
    pub matches: Vec<MatchInfo>,
    /// Unique keywords that were matched
    pub matched_keywords: Vec<String>,
    /// Match statistics
    pub stats: MatchStats,
}

impl Default for MatchResult {
    fn default() -> Self {
        Self {
            found: false,
            match_count: 0,
            matches: Vec::new(),
            matched_keywords: Vec::new(),
            stats: MatchStats::default(),
        }
    }
}

/// Statistics about keyword matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStats {
    /// Total characters processed
    pub total_chars: usize,
    /// Percentage of keywords found
    pub match_percentage: f64,
    /// Most frequent matched keyword
    pub most_frequent_keyword: Option<String>,
    /// Average distance between matches
    pub average_distance: Option<f64>,
}

impl Default for MatchStats {
    fn default() -> Self {
        Self {
            total_chars: 0,
            match_percentage: 0.0,
            most_frequent_keyword: None,
            average_distance: None,
        }
    }
}

/// Keyword matcher implementation
pub struct KeywordMatcher {
    config: KeywordConfig,
    regex_patterns: Option<Vec<Regex>>,
}

impl KeywordMatcher {
    /// Create a new keyword matcher
    pub fn new(config: KeywordConfig) -> Result<Self, CrawlError> {
        config.validate()?;

        let regex_patterns = if config.mode == KeywordMode::Regex {
            let patterns: Result<Vec<Regex>, _> = config
                .keywords
                .iter()
                .map(|pattern| Regex::new(pattern))
                .collect();

            Some(patterns.map_err(|e| {
                CrawlError::KeywordConfigError(format!("Failed to compile regex: {}", e))
            })?)
        } else {
            None
        };

        Ok(Self {
            config,
            regex_patterns,
        })
    }

    /// Match keywords in the given text
    pub fn match_keywords(&self, text: &str) -> Result<MatchResult, CrawlError> {
        if !self.config.should_filter() {
            return Ok(MatchResult::default());
        }

        let mut matches = Vec::new();
        let mut keyword_counts = HashMap::new();

        match self.config.mode {
            KeywordMode::Any | KeywordMode::CaseInsensitive => {
                self.match_any_keywords(text, &mut matches, &mut keyword_counts)?;
            }
            KeywordMode::All => {
                self.match_all_keywords(text, &mut matches, &mut keyword_counts)?;
            }
            KeywordMode::Exact => {
                self.match_exact_keywords(text, &mut matches, &mut keyword_counts)?;
            }
            KeywordMode::Regex => {
                self.match_regex_keywords(text, &mut matches, &mut keyword_counts)?;
            }
        }

        // Check minimum matches requirement
        if let Some(min_matches) = self.config.options.min_matches {
            if matches.len() < min_matches {
                return Ok(MatchResult::default());
            }
        }

        // Check proximity requirement
        if let Some(proximity_distance) = self.config.options.proximity_distance {
            if !self.check_proximity(&matches, proximity_distance) {
                return Ok(MatchResult::default());
            }
        }

        let matched_keywords: Vec<String> = keyword_counts.keys().cloned().collect();
        let stats = self.calculate_stats(text, &matches, &keyword_counts);

        Ok(MatchResult {
            found: !matches.is_empty(),
            match_count: matches.len(),
            matches,
            matched_keywords,
            stats,
        })
    }

    /// Match any of the keywords (OR operation)
    fn match_any_keywords(
        &self,
        text: &str,
        matches: &mut Vec<MatchInfo>,
        keyword_counts: &mut HashMap<String, usize>,
    ) -> Result<(), CrawlError> {
        let search_text = if self.config.mode == KeywordMode::CaseInsensitive {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        let keywords = self.config.normalized_keywords();

        for keyword in &keywords {
            let mut start = 0;
            while let Some(pos) = search_text[start..].find(keyword) {
                let absolute_pos = start + pos;
                let context = if self.config.options.include_context {
                    Some(self.extract_context(text, absolute_pos, keyword.len()))
                } else {
                    None
                };

                matches.push(MatchInfo {
                    keyword: keyword.clone(),
                    position: absolute_pos,
                    length: keyword.len(),
                    context,
                });

                *keyword_counts.entry(keyword.clone()).or_insert(0) += 1;
                start = absolute_pos + keyword.len();
            }
        }

        Ok(())
    }

    /// Match all keywords (AND operation)
    fn match_all_keywords(
        &self,
        text: &str,
        matches: &mut Vec<MatchInfo>,
        keyword_counts: &mut HashMap<String, usize>,
    ) -> Result<(), CrawlError> {
        let keywords = self.config.normalized_keywords();
        let search_text = if self.config.mode == KeywordMode::CaseInsensitive {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        // First check if all keywords exist
        let all_found = keywords.iter().all(|keyword| search_text.contains(keyword));

        if all_found {
            // If all keywords found, collect all matches
            self.match_any_keywords(text, matches, keyword_counts)?;
        }

        Ok(())
    }

    /// Match exact phrases
    fn match_exact_keywords(
        &self,
        text: &str,
        matches: &mut Vec<MatchInfo>,
        keyword_counts: &mut HashMap<String, usize>,
    ) -> Result<(), CrawlError> {
        for keyword in &self.config.keywords {
            let mut start = 0;
            while let Some(pos) = text[start..].find(keyword) {
                let absolute_pos = start + pos;

                // Check word boundaries for exact matching
                let is_word_start = absolute_pos == 0
                    || !text
                        .chars()
                        .nth(absolute_pos - 1)
                        .unwrap()
                        .is_alphanumeric();
                let is_word_end = absolute_pos + keyword.len() >= text.len()
                    || !text
                        .chars()
                        .nth(absolute_pos + keyword.len())
                        .unwrap()
                        .is_alphanumeric();

                if is_word_start && is_word_end {
                    let context = if self.config.options.include_context {
                        Some(self.extract_context(text, absolute_pos, keyword.len()))
                    } else {
                        None
                    };

                    matches.push(MatchInfo {
                        keyword: keyword.clone(),
                        position: absolute_pos,
                        length: keyword.len(),
                        context,
                    });

                    *keyword_counts.entry(keyword.clone()).or_insert(0) += 1;
                }

                start = absolute_pos + 1;
            }
        }

        Ok(())
    }

    /// Match using regular expressions
    fn match_regex_keywords(
        &self,
        text: &str,
        matches: &mut Vec<MatchInfo>,
        keyword_counts: &mut HashMap<String, usize>,
    ) -> Result<(), CrawlError> {
        if let Some(ref patterns) = self.regex_patterns {
            for (i, pattern) in patterns.iter().enumerate() {
                let keyword = &self.config.keywords[i];

                for mat in pattern.find_iter(text) {
                    let context = if self.config.options.include_context {
                        Some(self.extract_context(text, mat.start(), mat.len()))
                    } else {
                        None
                    };

                    matches.push(MatchInfo {
                        keyword: keyword.clone(),
                        position: mat.start(),
                        length: mat.len(),
                        context,
                    });

                    *keyword_counts.entry(keyword.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(())
    }

    /// Extract context around a match
    fn extract_context(&self, text: &str, position: usize, match_length: usize) -> String {
        let window = self.config.options.context_window;
        let start = position.saturating_sub(window);
        let end = std::cmp::min(text.len(), position + match_length + window);

        let context = &text[start..end];

        if self.config.options.highlight_matches {
            let relative_pos = position - start;
            let before = &context[..relative_pos];
            let matched = &context[relative_pos..relative_pos + match_length];
            let after = &context[relative_pos + match_length..];
            format!("{}**{}**{}", before, matched, after)
        } else {
            context.to_string()
        }
    }

    /// Check if matches meet proximity requirements
    fn check_proximity(&self, matches: &[MatchInfo], max_distance: usize) -> bool {
        if matches.len() < 2 {
            return true;
        }

        for i in 0..matches.len() - 1 {
            let current_end = matches[i].position + matches[i].length;
            let next_start = matches[i + 1].position;

            if next_start > current_end && (next_start - current_end) > max_distance {
                return false;
            }
        }

        true
    }

    /// Calculate match statistics
    fn calculate_stats(
        &self,
        text: &str,
        matches: &[MatchInfo],
        keyword_counts: &HashMap<String, usize>,
    ) -> MatchStats {
        let total_chars = text.len();
        let match_percentage = if self.config.keywords.is_empty() {
            0.0
        } else {
            (keyword_counts.len() as f64 / self.config.keywords.len() as f64) * 100.0
        };

        let most_frequent_keyword = keyword_counts
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(keyword, _)| keyword.clone());

        let average_distance = if matches.len() > 1 {
            let total_distance: usize = matches
                .windows(2)
                .map(|pair| pair[1].position - (pair[0].position + pair[0].length))
                .sum();
            Some(total_distance as f64 / (matches.len() - 1) as f64)
        } else {
            None
        };

        MatchStats {
            total_chars,
            match_percentage,
            most_frequent_keyword,
            average_distance,
        }
    }
}
