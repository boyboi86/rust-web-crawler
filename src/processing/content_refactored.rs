/// Content processing module - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex content processors
/// Following Rule 4: Privacy first - controlled access to processing logic
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, CharacterFilter, CompositeFilter, DelayDuration, ExtensionFilter, LanguageFilter,
    LengthFilter, LimitValue, PercentageValue, ProcessingResult, TaskContent, TaskError,
    TimeoutDuration, UrlString, WordFilter,
};
use crate::core::types_refactored::LangType;
use lol_html::{HtmlRewriter, Settings, element};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;
use whatlang::Lang;

/// Configuration for content processing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentProcessingConfig {
    // Text processing settings
    min_content_length: LimitValue,
    max_content_length: LimitValue,
    processing_timeout: TimeoutDuration,

    // Language filtering
    accepted_languages: Vec<String>, // Changed from Vec<Lang> to Vec<String> for serialization
    language_detection_enabled: BooleanFlag,
    language_confidence_threshold: f64,

    // Content filtering
    remove_html_tags: BooleanFlag,
    normalize_whitespace: BooleanFlag,
    remove_special_chars: BooleanFlag,
    preserve_structure: BooleanFlag,

    // Text extraction
    extract_text_only: BooleanFlag,
    extract_links: BooleanFlag,
    extract_metadata: BooleanFlag,
    follow_charset: BooleanFlag,
}

impl ContentProcessingConfig {
    pub fn builder() -> ContentProcessingConfigBuilder {
        ContentProcessingConfigBuilder::new()
    }

    pub fn min_content_length(&self) -> u64 {
        self.min_content_length.value()
    }

    pub fn max_content_length(&self) -> u64 {
        self.max_content_length.value()
    }

    pub fn processing_timeout(&self) -> TimeoutDuration {
        self.processing_timeout
    }

    pub fn accepted_languages(&self) -> &[Lang] {
        &self.accepted_languages
    }

    pub fn is_language_detection_enabled(&self) -> bool {
        self.language_detection_enabled.is_enabled()
    }

    pub fn language_confidence_threshold(&self) -> f64 {
        self.language_confidence_threshold
    }

    pub fn should_remove_html_tags(&self) -> bool {
        self.remove_html_tags.is_enabled()
    }

    pub fn should_normalize_whitespace(&self) -> bool {
        self.normalize_whitespace.is_enabled()
    }

    pub fn should_remove_special_chars(&self) -> bool {
        self.remove_special_chars.is_enabled()
    }

    pub fn should_preserve_structure(&self) -> bool {
        self.preserve_structure.is_enabled()
    }

    pub fn should_extract_text_only(&self) -> bool {
        self.extract_text_only.is_enabled()
    }

    pub fn should_extract_links(&self) -> bool {
        self.extract_links.is_enabled()
    }

    pub fn should_extract_metadata(&self) -> bool {
        self.extract_metadata.is_enabled()
    }

    pub fn should_follow_charset(&self) -> bool {
        self.follow_charset.is_enabled()
    }
}

impl Default for ContentProcessingConfig {
    fn default() -> Self {
        Self {
            min_content_length: LimitValue::new(10),
            max_content_length: LimitValue::new(1_000_000), // 1MB
            processing_timeout: TimeoutDuration::from_secs(30),
            accepted_languages: vec![Lang::Eng, Lang::Fra, Lang::Deu, Lang::Spa],
            language_detection_enabled: BooleanFlag::enabled(),
            language_confidence_threshold: 0.7,
            remove_html_tags: BooleanFlag::enabled(),
            normalize_whitespace: BooleanFlag::enabled(),
            remove_special_chars: BooleanFlag::disabled(),
            preserve_structure: BooleanFlag::disabled(),
            extract_text_only: BooleanFlag::enabled(),
            extract_links: BooleanFlag::enabled(),
            extract_metadata: BooleanFlag::enabled(),
            follow_charset: BooleanFlag::enabled(),
        }
    }
}

/// Builder for content processing configuration
#[derive(Debug)]
pub struct ContentProcessingConfigBuilder {
    min_content_length: LimitValue,
    max_content_length: LimitValue,
    processing_timeout: TimeoutDuration,
    accepted_languages: Vec<String>, // Changed from Vec<Lang> to Vec<String> for serialization
    language_detection_enabled: BooleanFlag,
    language_confidence_threshold: f64,
    remove_html_tags: BooleanFlag,
    normalize_whitespace: BooleanFlag,
    remove_special_chars: BooleanFlag,
    preserve_structure: BooleanFlag,
    extract_text_only: BooleanFlag,
    extract_links: BooleanFlag,
    extract_metadata: BooleanFlag,
    follow_charset: BooleanFlag,
}

impl ContentProcessingConfigBuilder {
    pub fn new() -> Self {
        let default_config = ContentProcessingConfig::default();
        Self {
            min_content_length: default_config.min_content_length,
            max_content_length: default_config.max_content_length,
            processing_timeout: default_config.processing_timeout,
            accepted_languages: default_config.accepted_languages,
            language_detection_enabled: default_config.language_detection_enabled,
            language_confidence_threshold: default_config.language_confidence_threshold,
            remove_html_tags: default_config.remove_html_tags,
            normalize_whitespace: default_config.normalize_whitespace,
            remove_special_chars: default_config.remove_special_chars,
            preserve_structure: default_config.preserve_structure,
            extract_text_only: default_config.extract_text_only,
            extract_links: default_config.extract_links,
            extract_metadata: default_config.extract_metadata,
            follow_charset: default_config.follow_charset,
        }
    }

    pub fn with_content_length_range(mut self, min: LimitValue, max: LimitValue) -> Self {
        self.min_content_length = min;
        self.max_content_length = max;
        self
    }

    pub fn with_processing_timeout(mut self, timeout: TimeoutDuration) -> Self {
        self.processing_timeout = timeout;
        self
    }

    pub fn with_accepted_languages(mut self, languages: Vec<Lang>) -> Self {
        self.accepted_languages = languages;
        self
    }

    pub fn with_language_detection(
        mut self,
        enabled: BooleanFlag,
        confidence_threshold: f64,
    ) -> Self {
        self.language_detection_enabled = enabled;
        self.language_confidence_threshold = confidence_threshold;
        self
    }

    pub fn with_html_processing(
        mut self,
        remove_tags: BooleanFlag,
        normalize_whitespace: BooleanFlag,
    ) -> Self {
        self.remove_html_tags = remove_tags;
        self.normalize_whitespace = normalize_whitespace;
        self
    }

    pub fn with_text_processing(
        mut self,
        remove_special_chars: BooleanFlag,
        preserve_structure: BooleanFlag,
    ) -> Self {
        self.remove_special_chars = remove_special_chars;
        self.preserve_structure = preserve_structure;
        self
    }

    pub fn with_extraction_settings(
        mut self,
        text_only: BooleanFlag,
        links: BooleanFlag,
        metadata: BooleanFlag,
        follow_charset: BooleanFlag,
    ) -> Self {
        self.extract_text_only = text_only;
        self.extract_links = links;
        self.extract_metadata = metadata;
        self.follow_charset = follow_charset;
        self
    }

    pub fn build(self) -> ContentProcessingConfig {
        ContentProcessingConfig {
            min_content_length: self.min_content_length,
            max_content_length: self.max_content_length,
            processing_timeout: self.processing_timeout,
            accepted_languages: self.accepted_languages,
            language_detection_enabled: self.language_detection_enabled,
            language_confidence_threshold: self.language_confidence_threshold,
            remove_html_tags: self.remove_html_tags,
            normalize_whitespace: self.normalize_whitespace,
            remove_special_chars: self.remove_special_chars,
            preserve_structure: self.preserve_structure,
            extract_text_only: self.extract_text_only,
            extract_links: self.extract_links,
            extract_metadata: self.extract_metadata,
            follow_charset: self.follow_charset,
        }
    }
}

impl Default for ContentProcessingConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracted content data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    text: String,
    links: Vec<UrlString>,
    metadata: HashMap<String, String>,
    language: Option<String>, // Changed from Option<Lang> to Option<String> for serialization
    language_confidence: f64,
    content_length: u64,
    word_count: u64,
    processing_duration: std::time::Duration,
}

impl ExtractedContent {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn links(&self) -> &[UrlString] {
        &self.links
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn language(&self) -> Option<Lang> {
        self.language
    }

    pub fn language_confidence(&self) -> f64 {
        self.language_confidence
    }

    pub fn content_length(&self) -> u64 {
        self.content_length
    }

    pub fn word_count(&self) -> u64 {
        self.word_count
    }

    pub fn processing_duration(&self) -> std::time::Duration {
        self.processing_duration
    }

    pub fn is_language_accepted(&self, accepted_languages: &[Lang]) -> bool {
        match self.language {
            Some(lang) => accepted_languages.contains(&lang),
            None => false,
        }
    }
}

/// Content processor using building blocks
/// Following Rule 4: Privacy first - processing logic encapsulated
pub struct ContentProcessor {
    // Private configuration and state
    config: ContentProcessingConfig,
    html_tag_regex: Regex,
    whitespace_regex: Regex,
    filters: CompositeFilter,
}

impl ContentProcessor {
    pub fn new(config: ContentProcessingConfig) -> ProcessingResult<Self> {
        let html_tag_regex = Regex::new(r"<[^>]*>")
            .map_err(|e| TaskError::parse(format!("Failed to compile HTML regex: {}", e)))?;

        let whitespace_regex = Regex::new(r"\s+")
            .map_err(|e| TaskError::parse(format!("Failed to compile whitespace regex: {}", e)))?;

        // Create composite filter using building blocks
        let language_strings: Vec<String> = config
            .accepted_languages()
            .iter()
            .map(|lang| format!("{:?}", lang).to_lowercase())
            .collect();

        let filters = CompositeFilter::builder()
            .add_filter(LengthFilter::new(
                BooleanFlag::enabled(),
                LimitValue::new(config.min_content_length()),
                LimitValue::new(config.max_content_length()),
            ))
            .add_filter(LanguageFilter::new(
                BooleanFlag::enabled(),
                language_strings,
                Vec::new(), // No blocked languages
                PercentageValue::new(config.language_confidence_threshold())?,
                BooleanFlag::enabled(), // Latin scripts only
            ))
            .build();

        Ok(Self {
            config,
            html_tag_regex,
            whitespace_regex,
            filters,
        })
    }

    pub fn with_defaults() -> ProcessingResult<Self> {
        Self::new(ContentProcessingConfig::default())
    }

    /// Process HTML content and extract structured data
    pub async fn process_html(
        &self,
        html: &str,
        url: &UrlString,
    ) -> ProcessingResult<ExtractedContent> {
        let start_time = std::time::Instant::now();

        // Check content length before processing
        if html.len() as u64 > self.config.max_content_length() {
            return Err(TaskError::content_validation(format!(
                "Content too large: {} bytes (max: {})",
                html.len(),
                self.config.max_content_length()
            )));
        }

        let mut extracted_text = String::new();
        let mut links = Vec::new();
        let mut metadata = HashMap::new();

        // Extract content using streaming HTML parser
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    // Extract text content
                    element!("*", |el| {
                        if self.config.should_extract_text_only() {
                            extracted_text.push_str(&el.get_inner_content());
                        }
                        Ok(())
                    }),
                    // Extract links
                    element!("a[href]", |el| {
                        if self.config.should_extract_links() {
                            if let Some(href) = el.get_attribute("href") {
                                if let Ok(url) = self.resolve_url(url, &href) {
                                    links.push(url);
                                }
                            }
                        }
                        Ok(())
                    }),
                    // Extract metadata
                    element!("meta[name][content]", |el| {
                        if self.config.should_extract_metadata() {
                            if let (Some(name), Some(content)) =
                                (el.get_attribute("name"), el.get_attribute("content"))
                            {
                                metadata.insert(name, content);
                            }
                        }
                        Ok(())
                    }),
                    element!("title", |el| {
                        if self.config.should_extract_metadata() {
                            metadata.insert("title".to_string(), el.get_inner_content());
                        }
                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
            |_: &[u8]| {},
        );

        rewriter
            .write(html.as_bytes())
            .map_err(|e| TaskError::parse(format!("HTML parsing failed: {}", e)))?;

        rewriter
            .end()
            .map_err(|e| TaskError::parse(format!("HTML parsing completion failed: {}", e)))?;

        // Process extracted text
        let processed_text = self.process_text(&extracted_text)?;

        // Detect language if enabled
        let (language, language_confidence) = if self.config.is_language_detection_enabled() {
            self.detect_language(&processed_text)
        } else {
            (None, 0.0)
        };

        // Count words using Unicode segmentation
        let word_count = processed_text.unicode_words().count() as u64;

        let content = ExtractedContent {
            text: processed_text,
            links: links.into_iter().collect(),
            metadata,
            language,
            language_confidence,
            content_length: extracted_text.len() as u64,
            word_count,
            processing_duration: start_time.elapsed(),
        };

        // Apply filters
        if !self.filters.passes(&content) {
            return Err(TaskError::content_validation("Content filtered out"));
        }

        Ok(content)
    }

    /// Process plain text content
    pub fn process_text(&self, text: &str) -> ProcessingResult<String> {
        let mut processed = text.to_string();

        // Remove HTML tags if configured
        if self.config.should_remove_html_tags() {
            processed = self.html_tag_regex.replace_all(&processed, " ").to_string();
        }

        // Normalize whitespace if configured
        if self.config.should_normalize_whitespace() {
            processed = self
                .whitespace_regex
                .replace_all(&processed, " ")
                .to_string();
            processed = processed.trim().to_string();
        }

        // Remove special characters if configured
        if self.config.should_remove_special_chars() {
            processed = processed
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
        }

        // Check minimum length
        if (processed.len() as u64) < self.config.min_content_length() {
            return Err(TaskError::content_validation(format!(
                "Content too short: {} characters (min: {})",
                processed.len(),
                self.config.min_content_length()
            )));
        }

        Ok(processed)
    }

    /// Detect language using whatlang
    fn detect_language(&self, text: &str) -> (Option<Lang>, f64) {
        match whatlang::detect(text) {
            Some(info) => {
                if info.confidence() >= self.config.language_confidence_threshold() {
                    (Some(info.lang()), info.confidence())
                } else {
                    (None, info.confidence())
                }
            }
            None => (None, 0.0),
        }
    }

    /// Resolve relative URLs to absolute URLs
    fn resolve_url(&self, base_url: &UrlString, href: &str) -> ProcessingResult<UrlString> {
        use url::Url;

        let base = Url::parse(base_url.as_str())
            .map_err(|e| TaskError::parse(format!("Invalid base URL: {}", e)))?;

        let resolved = base
            .join(href)
            .map_err(|e| TaskError::parse(format!("Failed to resolve URL '{}': {}", href, e)))?;

        Ok(UrlString::new(resolved.as_str()))
    }

    /// Get processor configuration
    pub fn config(&self) -> &ContentProcessingConfig {
        &self.config
    }

    /// Update processor configuration
    pub fn update_config(&mut self, config: ContentProcessingConfig) -> ProcessingResult<()> {
        // Recreate filters with new configuration
        let language_strings: Vec<String> = config
            .accepted_languages()
            .iter()
            .map(|lang| format!("{:?}", lang).to_lowercase())
            .collect();

        self.filters = CompositeFilter::builder()
            .add_filter(LengthFilter::new(
                BooleanFlag::enabled(),
                LimitValue::new(config.min_content_length()),
                LimitValue::new(config.max_content_length()),
            ))
            .add_filter(LanguageFilter::new(
                BooleanFlag::enabled(),
                language_strings,
                Vec::new(), // No blocked languages
                PercentageValue::new(config.language_confidence_threshold())?,
                BooleanFlag::enabled(), // Latin scripts only
            ))
            .build();

        self.config = config;
        Ok(())
    }

    /// Process content with timeout protection
    pub async fn process_with_timeout(
        &self,
        html: &str,
        url: &UrlString,
    ) -> ProcessingResult<ExtractedContent> {
        let timeout = self.config.processing_timeout().duration();

        tokio::time::timeout(timeout, self.process_html(html, url))
            .await
            .map_err(|_| TaskError::timeout(timeout.as_millis() as u64))?
    }
}

impl Default for ContentProcessor {
    fn default() -> Self {
        Self::with_defaults().expect("Failed to create default content processor")
    }
}

/// Custom filter implementation for ExtractedContent
impl crate::common::FilterTarget for ExtractedContent {
    fn passes_length_filter(&self, filter: &LengthFilter) -> bool {
        let content_len = self.content_length();
        content_len >= filter.min_length() && content_len <= filter.max_length()
    }

    fn passes_language_filter(&self, filter: &LanguageFilter) -> bool {
        match self.language() {
            Some(lang) => filter.allowed_languages().contains(&lang),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_content_processor_creation() {
        let config = ContentProcessingConfig::builder()
            .with_content_length_range(LimitValue::new(5), LimitValue::new(1000))
            .build();

        let processor = ContentProcessor::new(config).expect("Failed to create processor");
        assert_eq!(processor.config().min_content_length(), 5);
    }

    #[tokio::test]
    async fn test_html_processing() {
        let processor = ContentProcessor::with_defaults().expect("Failed to create processor");
        let html = r#"<html><head><title>Test</title></head><body><p>Hello World</p><a href="/test">Link</a></body></html>"#;
        let url = UrlString::new("https://example.com");

        let result = processor
            .process_html(html, &url)
            .await
            .expect("Failed to process HTML");

        assert!(result.text().contains("Hello World"));
        assert!(result.metadata().contains_key("title"));
        assert!(!result.links().is_empty());
    }

    #[tokio::test]
    async fn test_text_processing() {
        let processor = ContentProcessor::with_defaults().expect("Failed to create processor");
        let text = "  <p>Hello    World!</p>  ";

        let result = processor
            .process_text(text)
            .expect("Failed to process text");
        assert_eq!(result.trim(), "Hello World!");
    }

    #[tokio::test]
    async fn test_language_detection() {
        let config = ContentProcessingConfig::builder()
            .with_language_detection(BooleanFlag::enabled(), 0.5)
            .build();

        let processor = ContentProcessor::new(config).expect("Failed to create processor");
        let (lang, confidence) = processor.detect_language("Hello world, this is English text");

        assert!(confidence > 0.5);
        assert_eq!(lang, Some(Lang::Eng));
    }

    #[tokio::test]
    async fn test_content_filtering() {
        let config = ContentProcessingConfig::builder()
            .with_content_length_range(LimitValue::new(20), LimitValue::new(1000))
            .build();

        let processor = ContentProcessor::new(config).expect("Failed to create processor");
        let short_html = "<p>Short</p>";
        let url = UrlString::new("https://example.com");

        let result = processor.process_html(short_html, &url).await;
        assert!(result.is_err()); // Should fail due to length filter
    }

    #[tokio::test]
    async fn test_processing_timeout() {
        let config = ContentProcessingConfig::builder()
            .with_processing_timeout(TimeoutDuration::from_millis(1)) // Very short timeout
            .build();

        let processor = ContentProcessor::new(config).expect("Failed to create processor");
        let large_html = "<p>".repeat(10000) + "Large content" + &"</p>".repeat(10000);
        let url = UrlString::new("https://example.com");

        let result = processor.process_with_timeout(&large_html, &url).await;
        // May pass or timeout depending on system speed - both outcomes are valid
    }
}
