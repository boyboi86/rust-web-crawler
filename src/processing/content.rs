/// Consolidated content processing and HTML utilities
///
/// This module assembles all content-related functionality including:
/// - Content extraction and validation (from processing/content.rs)
/// - HTML parsing utilities (from utils/html.rs)
/// - Text analysis and language detection integration
use anyhow::Error;
use lol_html::{HtmlRewriter, Settings, element};
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;
use whatlang::detect;

use crate::config::{LatinWordFilter, defaults};
use crate::core::{ContentProcessor, LangType};

/// Content processor with text extraction and validation
pub struct ContentExtractor {
    regex_cache: regex::Regex,
    accepted_languages: Vec<LangType>,
    latin_word_filter: LatinWordFilter,
}

impl ContentExtractor {
    pub fn new(
        accepted_languages: Vec<LangType>,
        latin_word_filter: LatinWordFilter,
    ) -> Result<Self, Error> {
        // Pre-compile regex for HTML cleaning
        let regex_cache = regex::Regex::new(r"<[^>]*>")?;

        Ok(Self {
            regex_cache,
            accepted_languages,
            latin_word_filter,
        })
    }
}

impl ContentProcessor for ContentExtractor {
    /// Extract and validate content from HTML (optimized approach)
    async fn extract_and_validate(&self, content: &str) -> Result<(String, usize), Error> {
        // Early exit for empty content
        if content.is_empty() {
            return Ok((String::new(), 0));
        }

        // Quick check for very short content to avoid processing overhead
        if content.len() < defaults::MIN_CONTENT_LENGTH_BYTES {
            return Ok((String::new(), 0));
        }

        // 1. Use lol-html for basic cleaning, then extract text manually
        let mut cleaned_html = Vec::new();

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    // Remove script and style content
                    element!("script", |el| {
                        el.remove();
                        Ok(())
                    }),
                    element!("style", |el| {
                        el.remove();
                        Ok(())
                    }),
                    // Keep text content from other elements
                    element!("*", |_el| {
                        // Keep the element but we'll extract text later
                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
            |c: &[u8]| {
                cleaned_html.extend_from_slice(c);
            },
        );

        rewriter.write(content.as_bytes())?;
        rewriter.end()?;

        let cleaned_html = String::from_utf8_lossy(&cleaned_html);

        // 2. Extract text using regex (faster than DOM parsing for large content)
        let text = self.regex_cache.replace_all(&cleaned_html, " ");

        // 3. Clean and normalize whitespace
        let normalized = self.normalize_text(&text);

        // 4. Count words and validate
        let word_count = self.count_words(&normalized);

        // 5. Apply minimum word count filter
        if word_count < defaults::MIN_WORD_COUNT_THRESHOLD {
            return Ok((String::new(), 0));
        }

        // 6. Language detection and filtering
        if !self.accepted_languages.is_empty() {
            if let Some(detected) = detect(&normalized) {
                if let Some(lang_type) = LangType::from_detected_lang(detected.lang()) {
                    if !self.accepted_languages.contains(&lang_type) {
                        return Ok((String::new(), 0));
                    }
                } else {
                    // Language not supported
                    return Ok((String::new(), 0));
                }
            }
        }

        Ok((normalized, word_count))
    }

    fn extract_text_from_cleaned_html(&self, html: &str) -> String {
        // Remove HTML tags and extract text
        let text = self.regex_cache.replace_all(html, " ");
        self.normalize_text(&text)
    }

    async fn extract_text_fallback(&self, content: &str) -> Result<(String, usize), Error> {
        // Simple fallback that just removes HTML tags
        let text = self.regex_cache.replace_all(content, " ");
        let normalized = self.normalize_text(&text);
        let word_count = self.count_words(&normalized);

        if word_count == 0 {
            return Err(anyhow::anyhow!("No meaningful content extracted"));
        }

        Ok((normalized, word_count))
    }
}

impl ContentExtractor {
    /// Normalize text by cleaning extra whitespace and applying filters
    fn normalize_text(&self, text: &str) -> String {
        // Remove extra whitespace and normalize
        let clean_re = regex::Regex::new(r"\s+").unwrap();
        let normalized = clean_re.replace_all(text.trim(), " ");

        // Apply Latin word filter if configured
        if !self.latin_word_filter.excluded_words.is_empty()
            || self.latin_word_filter.min_word_length > 1
        {
            self.apply_latin_filter(&normalized)
        } else {
            normalized.to_string()
        }
    }

    /// Apply Latin word filtering
    fn apply_latin_filter(&self, text: &str) -> String {
        let words: Vec<&str> = text.unicode_words().collect();
        if words.is_empty() {
            return text.to_string();
        }

        // Filter words based on the latin word filter configuration
        let filtered_words: Vec<&str> = words
            .iter()
            .filter(|word| {
                // Apply minimum word length filter
                if word.len() < self.latin_word_filter.min_word_length {
                    return false;
                }

                // Apply numeric exclusion filter
                if self.latin_word_filter.exclude_numeric && word.chars().all(|c| c.is_numeric()) {
                    return false;
                }

                // Apply excluded words filter
                if self
                    .latin_word_filter
                    .excluded_words
                    .contains(&word.to_lowercase())
                {
                    return false;
                }

                true
            })
            .copied()
            .collect();

        if filtered_words.is_empty() {
            String::new()
        } else {
            filtered_words.join(" ")
        }
    }

    /// Count words in text using Unicode segmentation
    fn count_words(&self, text: &str) -> usize {
        text.unicode_words().count()
    }
}

// ============================================================================
// HTML Utility Functions (assembled from utils/html.rs)
// ============================================================================

/// Extract title from HTML content
pub fn extract_title_from_html(content: &str) -> Option<String> {
    let re = Regex::new(r"<title[^>]*>([^<]+)</title>").ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract links from HTML content
pub fn extract_links_from_html(content: &str) -> Vec<String> {
    let re = Regex::new(r#"href\s*=\s*["']([^"']+)["']"#).unwrap_or_else(|_| {
        // Fallback to a simpler pattern if the complex one fails
        Regex::new(r#"href=["']([^"']+)["']"#).unwrap()
    });

    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|link| {
            // Filter out non-HTTP links
            link.starts_with("http") || link.starts_with("//")
        })
        .collect()
}

/// Extract meta description from HTML content
pub fn extract_meta_description(content: &str) -> Option<String> {
    let re = Regex::new(
        r#"<meta[^>]*name\s*=\s*["']description["'][^>]*content\s*=\s*["']([^"']*)["']"#,
    )
    .ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// Extract all text content from HTML (strip tags)
pub fn extract_text_content(html: &str) -> String {
    let re = Regex::new(r"<[^>]*>").unwrap();
    let text = re.replace_all(html, " ");

    // Clean up multiple spaces and normalize whitespace
    let clean_re = Regex::new(r"\s+").unwrap();
    clean_re.replace_all(&text, " ").trim().to_string()
}

/// Check if HTML content appears to be a valid page
pub fn is_valid_html_page(content: &str) -> bool {
    // Check for basic HTML structure
    let has_html_tag = content.to_lowercase().contains("<html");
    let has_body_tag = content.to_lowercase().contains("<body");
    let has_title_tag = content.to_lowercase().contains("<title");

    // Should have at least some HTML structure
    has_html_tag || has_body_tag || has_title_tag
}

/// Extract all image URLs from HTML content
pub fn extract_image_urls(content: &str) -> Vec<String> {
    let re = Regex::new(r#"<img[^>]*src\s*=\s*["']([^"']+)["']"#).unwrap();

    re.captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|url| {
            // Filter for common image extensions
            let lower_url = url.to_lowercase();
            lower_url.ends_with(".jpg")
                || lower_url.ends_with(".jpeg")
                || lower_url.ends_with(".png")
                || lower_url.ends_with(".gif")
                || lower_url.ends_with(".webp")
                || lower_url.ends_with(".svg")
        })
        .collect()
}
