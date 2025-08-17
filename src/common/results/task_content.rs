use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContent {
    content: String,
    word_count: usize,
    detected_language: Option<String>,
    content_type: Option<String>,
    encoding: Option<String>,
}

impl TaskContent {
    pub fn new(
        content: String,
        word_count: usize,
        detected_language: Option<String>,
        content_type: Option<String>,
        encoding: Option<String>,
    ) -> Self {
        Self {
            content,
            word_count,
            detected_language,
            content_type,
            encoding,
        }
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn word_count(&self) -> usize {
        self.word_count
    }
    pub fn detected_language(&self) -> Option<&str> {
        self.detected_language.as_deref()
    }
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
    pub fn encoding(&self) -> Option<&str> {
        self.encoding.as_deref()
    }
}

impl Default for TaskContent {
    fn default() -> Self {
        Self {
            content: String::new(),
            word_count: 0,
            detected_language: None,
            content_type: None,
            encoding: None,
        }
    }
}
