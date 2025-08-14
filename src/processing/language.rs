use crate::core::LangType;
/// Language detection and analysis
///
/// This module provides comprehensive language detection and analysis capabilities,
/// integrating with the core LangType system and providing utility functions.
/// Enhanced with advanced text cleaning and preprocessing (Feature 3).
use whatlang::{Lang, detect};

// Re-export text cleaning components (Level 3 extension)
pub use crate::processing::cleaning::{
    CharacterFilter, CleaningConfig, CleaningEngine, CleaningResult, CleaningRule, CleaningStats,
    LanguageFilter, LengthFilter, RuleType, TextCleaner, WordFilter,
};

/// Detect language from content using whatlang crate
pub fn detect_language(content: &str) -> Option<String> {
    if let Some(info) = detect(content) {
        match info.lang() {
            Lang::Eng => Some("en".to_string()),
            Lang::Fra => Some("fr".to_string()),
            Lang::Deu => Some("de".to_string()),
            Lang::Cmn => Some("zh".to_string()),
            Lang::Jpn => Some("ja".to_string()),
            Lang::Kor => Some("ko".to_string()),
            Lang::Spa => Some("es".to_string()),
            Lang::Ita => Some("it".to_string()),
            Lang::Por => Some("pt".to_string()),
            Lang::Rus => Some("ru".to_string()),
            Lang::Ara => Some("ar".to_string()),
            Lang::Hin => Some("hi".to_string()),
            _ => Some(format!("{:?}", info.lang()).to_lowercase()),
        }
    } else {
        None
    }
}

/// Detect language and return LangType
pub fn detect_language_type(content: &str) -> Option<LangType> {
    if let Some(info) = detect(content) {
        LangType::from_detected_lang(info.lang())
    } else {
        None
    }
}

/// Convert language code to display name
pub fn language_code_to_name(code: &str) -> &'static str {
    match code {
        "en" => "English",
        "fr" => "French",
        "de" => "German",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "es" => "Spanish",
        "it" => "Italian",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "ar" => "Arabic",
        "hi" => "Hindi",
        _ => "Unknown",
    }
}

/// Estimate reading time based on word count
pub fn estimate_reading_time(word_count: usize) -> std::time::Duration {
    // Average reading speed is about 200-250 words per minute
    const WORDS_PER_MINUTE: f64 = 225.0;

    let minutes = word_count as f64 / WORDS_PER_MINUTE;
    std::time::Duration::from_secs_f64(minutes * 60.0)
}

/// Estimate content difficulty based on average word length
pub fn estimate_content_difficulty(content: &str) -> ContentDifficulty {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return ContentDifficulty::Unknown;
    }

    let total_chars: usize = words.iter().map(|w| w.chars().count()).sum();
    let avg_word_length = total_chars as f64 / words.len() as f64;

    match avg_word_length {
        x if x < 4.0 => ContentDifficulty::Easy,
        x if x < 5.5 => ContentDifficulty::Medium,
        x if x < 7.0 => ContentDifficulty::Hard,
        _ => ContentDifficulty::VeryHard,
    }
}

/// Content difficulty levels
#[derive(Debug, Clone, PartialEq)]
pub enum ContentDifficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
    Unknown,
}

/// Language confidence score from detection
pub fn get_language_confidence(content: &str) -> Option<f64> {
    if let Some(info) = detect(content) {
        Some(info.confidence())
    } else {
        None
    }
}

/// Check if content is likely to be in a supported language
pub fn is_supported_language(content: &str, supported_languages: &[LangType]) -> bool {
    if let Some(detected_lang) = detect_language_type(content) {
        supported_languages.contains(&detected_lang)
    } else {
        false
    }
}

/// Get language statistics for content
pub struct LanguageStats {
    pub detected_language: Option<LangType>,
    pub confidence: f64,
    pub difficulty: ContentDifficulty,
    pub estimated_reading_time: std::time::Duration,
    pub word_count: usize,
}

pub fn analyze_language_stats(content: &str) -> LanguageStats {
    let word_count = content.split_whitespace().count();

    LanguageStats {
        detected_language: detect_language_type(content),
        confidence: get_language_confidence(content).unwrap_or(0.0),
        difficulty: estimate_content_difficulty(content),
        estimated_reading_time: estimate_reading_time(word_count),
        word_count,
    }
}
