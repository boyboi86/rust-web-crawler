/// Language detection utilities

/// Detect language from content using whatlang crate
pub fn detect_language(content: &str) -> Option<String> {
    use whatlang::{Lang, detect};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_english() {
        let english_text =
            "This is a sample English text with many words to ensure proper detection.";
        let result = detect_language(english_text);
        assert_eq!(result, Some("en".to_string()));
    }

    #[test]
    fn test_language_code_to_name() {
        assert_eq!(language_code_to_name("en"), "English");
        assert_eq!(language_code_to_name("fr"), "French");
        assert_eq!(language_code_to_name("unknown"), "Unknown");
    }

    #[test]
    fn test_estimate_reading_time() {
        let time = estimate_reading_time(225); // Should be about 1 minute
        assert!(time.as_secs() >= 50 && time.as_secs() <= 70);
    }
}
