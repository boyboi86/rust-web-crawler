/// Keyword-based content extractor
use serde::{Deserialize, Serialize};

use super::config::KeywordConfig;
use super::matcher::{KeywordMatcher, MatchResult};
use crate::core::error::CrawlError;

/// Information about keyword matches in extracted content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordMatchInfo {
    /// Original text content
    pub original_content: String,
    /// Filtered content (only parts containing keywords)
    pub filtered_content: Option<String>,
    /// Match results
    pub match_result: MatchResult,
    /// Content statistics
    pub stats: ContentStats,
}

/// Statistics about the content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStats {
    /// Original content length
    pub original_length: usize,
    /// Filtered content length
    pub filtered_length: usize,
    /// Compression ratio (filtered/original)
    pub compression_ratio: f64,
    /// Number of paragraphs containing keywords
    pub relevant_paragraphs: usize,
}

impl Default for ContentStats {
    fn default() -> Self {
        Self {
            original_length: 0,
            filtered_length: 0,
            compression_ratio: 0.0,
            relevant_paragraphs: 0,
        }
    }
}

/// Keyword-based content extractor
pub struct KeywordExtractor {
    config: KeywordConfig,
    matcher: KeywordMatcher,
}

impl KeywordExtractor {
    /// Create a new keyword extractor
    pub fn new(config: KeywordConfig) -> Result<Self, CrawlError> {
        let matcher = KeywordMatcher::new(config.clone())?;

        Ok(Self { config, matcher })
    }

    /// Extract content based on keyword filtering
    pub fn extract_content(&self, content: &str) -> Result<KeywordMatchInfo, CrawlError> {
        if !self.config.should_filter() {
            // If keyword filtering is disabled, return original content
            return Ok(KeywordMatchInfo {
                original_content: content.to_string(),
                filtered_content: Some(content.to_string()),
                match_result: MatchResult::default(),
                stats: ContentStats {
                    original_length: content.len(),
                    filtered_length: content.len(),
                    compression_ratio: 1.0,
                    relevant_paragraphs: 0,
                },
            });
        }

        let match_result = self.matcher.match_keywords(content)?;

        if !match_result.found {
            return Err(CrawlError::KeywordNotFound);
        }

        let (filtered_content, stats) = self.filter_content(content, &match_result);

        Ok(KeywordMatchInfo {
            original_content: content.to_string(),
            filtered_content: Some(filtered_content),
            match_result,
            stats,
        })
    }

    /// Filter content to include only relevant parts
    fn filter_content(&self, content: &str, match_result: &MatchResult) -> (String, ContentStats) {
        let original_length = content.len();

        // Split content into paragraphs
        let paragraphs: Vec<&str> = content.split('\n').collect();
        let mut relevant_paragraphs: Vec<String> = Vec::new();
        let mut relevant_count = 0;

        // Find paragraphs containing keywords
        for paragraph in paragraphs {
            if paragraph.trim().is_empty() {
                continue;
            }

            // Check if this paragraph contains any of our matches
            let paragraph_has_match = match_result.matches.iter().any(|match_info| {
                let paragraph_start = content.find(paragraph).unwrap_or(0);
                let paragraph_end = paragraph_start + paragraph.len();
                match_info.position >= paragraph_start && match_info.position < paragraph_end
            });

            if paragraph_has_match {
                relevant_paragraphs.push(paragraph.to_string());
                relevant_count += 1;
            }
        }

        // If no relevant paragraphs found, extract sentences around matches
        if relevant_paragraphs.is_empty() {
            relevant_paragraphs = self.extract_sentences_around_matches(content, match_result);
            relevant_count = relevant_paragraphs.len();
        }

        let filtered_content = relevant_paragraphs.join("\n\n");
        let filtered_length = filtered_content.len();
        let compression_ratio = if original_length > 0 {
            filtered_length as f64 / original_length as f64
        } else {
            0.0
        };

        let stats = ContentStats {
            original_length,
            filtered_length,
            compression_ratio,
            relevant_paragraphs: relevant_count,
        };

        (filtered_content, stats)
    }

    /// Extract sentences around keyword matches
    fn extract_sentences_around_matches(
        &self,
        content: &str,
        match_result: &MatchResult,
    ) -> Vec<String> {
        let mut sentences = Vec::new();
        let sentence_window = 2; // Extract 2 sentences before and after match

        for match_info in &match_result.matches {
            // Find sentence boundaries around the match
            let sentence_start =
                self.find_sentence_start(content, match_info.position, sentence_window);
            let sentence_end = self.find_sentence_end(
                content,
                match_info.position + match_info.length,
                sentence_window,
            );

            let extracted = &content[sentence_start..sentence_end];
            if !extracted.trim().is_empty() {
                sentences.push(extracted.trim().to_string());
            }
        }

        // Remove duplicates and sort by position
        sentences.sort();
        sentences.dedup();
        sentences
    }

    /// Find the start of a sentence N sentences before the given position
    fn find_sentence_start(&self, content: &str, position: usize, window: usize) -> usize {
        let mut current_pos = position;
        let mut sentences_found = 0;

        while current_pos > 0 && sentences_found < window {
            if let Some(sentence_end) = content[..current_pos].rfind('.') {
                sentences_found += 1;
                if sentences_found >= window {
                    return sentence_end + 1;
                }
                current_pos = sentence_end;
            } else {
                break;
            }
        }

        0
    }

    /// Find the end of a sentence N sentences after the given position
    fn find_sentence_end(&self, content: &str, position: usize, window: usize) -> usize {
        let mut current_pos = position;
        let mut sentences_found = 0;

        while current_pos < content.len() && sentences_found < window {
            if let Some(relative_pos) = content[current_pos..].find('.') {
                let absolute_pos = current_pos + relative_pos;
                sentences_found += 1;
                if sentences_found >= window {
                    return std::cmp::min(absolute_pos + 1, content.len());
                }
                current_pos = absolute_pos + 1;
            } else {
                break;
            }
        }

        content.len()
    }

    /// Extract only the matched text with context
    pub fn extract_matches_only(&self, content: &str) -> Result<Vec<String>, CrawlError> {
        let match_result = self.matcher.match_keywords(content)?;

        if !match_result.found {
            return Err(CrawlError::KeywordNotFound);
        }

        Ok(match_result
            .matches
            .iter()
            .map(|match_info| {
                if let Some(ref context) = match_info.context {
                    context.clone()
                } else {
                    content[match_info.position..match_info.position + match_info.length]
                        .to_string()
                }
            })
            .collect())
    }

    /// Get summary statistics for the content
    pub fn get_content_summary(&self, content: &str) -> Result<ContentStats, CrawlError> {
        let match_result = self.matcher.match_keywords(content)?;

        if match_result.found {
            let (_, stats) = self.filter_content(content, &match_result);
            Ok(stats)
        } else {
            Ok(ContentStats {
                original_length: content.len(),
                filtered_length: 0,
                compression_ratio: 0.0,
                relevant_paragraphs: 0,
            })
        }
    }
}
