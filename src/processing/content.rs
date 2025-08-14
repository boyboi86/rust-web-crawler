use anyhow::Error;
use lol_html::{HtmlRewriter, Settings, element};
use unicode_segmentation::UnicodeSegmentation;
use whatlang::detect;

use crate::config::defaults;
use crate::core::{ContentProcessor, LangType};

/// Content processor with text extraction and validation
pub struct ContentExtractor {
    regex_cache: regex::Regex,
    accepted_languages: Vec<LangType>,
}

impl ContentExtractor {
    pub fn new(accepted_languages: Vec<LangType>) -> Result<Self, Error> {
        // Pre-compile regex for HTML cleaning
        let regex_cache = regex::Regex::new(r"<[^>]*>")?;

        Ok(Self {
            regex_cache,
            accepted_languages,
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
                    // Remove unwanted elements entirely
                    element!(
                        "script, style, nav, header, footer, aside, noscript",
                        |el| {
                            el.remove();
                            Ok(())
                        }
                    ),
                ],
                ..Settings::default()
            },
            |chunk: &[u8]| {
                cleaned_html.extend_from_slice(chunk);
            },
        );

        // Process the HTML
        if rewriter.write(content.as_bytes()).is_err() {
            // Fallback: extract text using regex if lol-html fails
            return self.extract_text_fallback(content).await;
        }

        if rewriter.end().is_err() {
            return self.extract_text_fallback(content).await;
        }

        // Convert cleaned HTML back to string and extract text
        let cleaned_content = String::from_utf8_lossy(&cleaned_html);
        let mut extracted_text = self.extract_text_from_cleaned_html(&cleaned_content);

        // Clean up the extracted text
        extracted_text = extracted_text.trim().to_string();

        if extracted_text.is_empty() {
            return Ok((String::new(), 0));
        }

        // Early exit for very short extracted text
        if extracted_text.len() < defaults::MIN_EXTRACTED_TEXT_LENGTH {
            return Ok((String::new(), 0));
        }

        // 2. Detect language once and validate against accepted languages
        // Use sampling for large texts to speed up language detection
        let sample_text = if extracted_text.len() > defaults::LANG_DETECTION_SAMPLE_SIZE {
            // Safe string slicing at char boundaries
            extracted_text
                .chars()
                .take(defaults::LANG_DETECTION_SAMPLE_SIZE)
                .collect::<String>()
        } else {
            extracted_text.clone()
        };

        let detected_lang = detect(&sample_text);
        if let Some(detected_lang_type) =
            LangType::from_detection_with_validation(detected_lang, &self.accepted_languages)
        {
            // 3. Use optimized word counting
            let word_count = if matches!(
                detected_lang_type,
                LangType::Cmn | LangType::Jpn | LangType::Kor
            ) {
                // For CJK languages, use sampling for very large texts
                if extracted_text.len() > defaults::CJK_WORD_COUNT_SAMPLE_SIZE {
                    let sample = extracted_text
                        .chars()
                        .take(defaults::CJK_WORD_COUNT_SAMPLE_SIZE)
                        .collect::<String>();
                    let sample_count = sample.unicode_words().count();
                    // Extrapolate word count
                    (sample_count * extracted_text.chars().count())
                        / defaults::CJK_WORD_COUNT_SAMPLE_SIZE
                } else {
                    extracted_text.unicode_words().count()
                }
            } else {
                // For Latin-based languages, filter out very short words (< 3 characters)
                extracted_text
                    .unicode_words()
                    .filter(|word| word.len() >= defaults::MIN_WORD_LENGTH_LATIN)
                    .count()
            };

            // 4. Apply minimum word count threshold
            if word_count >= defaults::MIN_WORD_COUNT_THRESHOLD {
                Ok((extracted_text, word_count))
            } else {
                Ok((String::new(), word_count))
            }
        } else {
            // Language detection failed
            Ok((String::new(), 0))
        }
    }

    /// Helper method to extract text from cleaned HTML (optimized with pre-compiled regex)
    fn extract_text_from_cleaned_html(&self, html: &str) -> String {
        // Use pre-compiled regex for better performance
        let text = self.regex_cache.replace_all(html, " ");
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Fallback method for text extraction
    async fn extract_text_fallback(&self, content: &str) -> Result<(String, usize), Error> {
        let extracted_text = self.extract_text_from_cleaned_html(content);

        if extracted_text.is_empty() {
            return Ok((String::new(), 0));
        }

        // Detect language and count words using the same logic as the main method
        let detected_lang = detect(&extracted_text);
        if let Some(detected_lang_type) =
            LangType::from_detection_with_validation(detected_lang, &self.accepted_languages)
        {
            let word_count = if matches!(
                detected_lang_type,
                LangType::Cmn | LangType::Jpn | LangType::Kor
            ) {
                extracted_text.unicode_words().count()
            } else {
                extracted_text
                    .unicode_words()
                    .filter(|word| word.len() >= defaults::MIN_WORD_LENGTH_LATIN)
                    .count()
            };

            if word_count >= defaults::MIN_WORD_COUNT_THRESHOLD {
                Ok((extracted_text, word_count))
            } else {
                Ok((String::new(), word_count))
            }
        } else {
            Ok((String::new(), 0))
        }
    }
}
