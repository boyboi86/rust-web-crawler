#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

/// Identifier primitive building blocks for session and task identification
/// Level 3 implementation - complete struct and functionality for ID types
use serde::{Deserialize, Serialize};

/// Building block for session ID - ensures consistent session identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Generate a new random session ID
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        Self(format!("session_{}", timestamp))
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Building block for task ID - ensures consistent task identification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TaskId(String);

impl TaskId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Generate a new random task ID
    pub fn generate() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        Self(format!("task_{}", timestamp))
    }

    /// Generate a task ID with a specific prefix
    pub fn generate_with_prefix(prefix: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        Self(format!("{}_{}", prefix, timestamp))
    }
}

impl From<String> for TaskId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TaskId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let session_id = SessionId::new("session_123");
        assert_eq!(session_id.as_str(), "session_123");
        assert!(!session_id.is_empty());
        assert_eq!(session_id.len(), 11);
    }

    #[test]
    fn test_session_id_generate() {
        let session_id = SessionId::generate();
        assert!(session_id.as_str().starts_with("session_"));
        assert!(!session_id.is_empty());
    }

    #[test]
    fn test_session_id_from_string() {
        let session_id = SessionId::from("test_session".to_string());
        assert_eq!(session_id.as_str(), "test_session");
    }

    #[test]
    fn test_session_id_from_str() {
        let session_id = SessionId::from("test_session");
        assert_eq!(session_id.as_str(), "test_session");
    }

    #[test]
    fn test_session_id_display() {
        let session_id = SessionId::new("display_test");
        assert_eq!(format!("{}", session_id), "display_test");
    }

    #[test]
    fn test_task_id_creation() {
        let task_id = TaskId::new("task_456");
        assert_eq!(task_id.as_str(), "task_456");
        assert!(!task_id.is_empty());
        assert_eq!(task_id.len(), 8);
    }

    #[test]
    fn test_task_id_generate() {
        let task_id = TaskId::generate();
        assert!(task_id.as_str().starts_with("task_"));
        assert!(!task_id.is_empty());
    }

    #[test]
    fn test_task_id_generate_with_prefix() {
        let task_id = TaskId::generate_with_prefix("crawl");
        assert!(task_id.as_str().starts_with("crawl_"));
        assert!(!task_id.is_empty());
    }

    #[test]
    fn test_task_id_from_string() {
        let task_id = TaskId::from("test_task".to_string());
        assert_eq!(task_id.as_str(), "test_task");
    }

    #[test]
    fn test_task_id_from_str() {
        let task_id = TaskId::from("test_task");
        assert_eq!(task_id.as_str(), "test_task");
    }

    #[test]
    fn test_task_id_display() {
        let task_id = TaskId::new("display_task");
        assert_eq!(format!("{}", task_id), "display_task");
    }

    #[test]
    fn test_empty_identifiers() {
        let empty_session = SessionId::default();
        let empty_task = TaskId::default();

        assert!(empty_session.is_empty());
        assert!(empty_task.is_empty());
        assert_eq!(empty_session.len(), 0);
        assert_eq!(empty_task.len(), 0);
    }

    #[test]
    fn test_identifier_uniqueness() {
        let session1 = SessionId::generate();
        let session2 = SessionId::generate();
        let task1 = TaskId::generate();
        let task2 = TaskId::generate();

        // IDs should be different (timestamp-based)
        assert_ne!(session1, session2);
        assert_ne!(task1, task2);
    }
}
