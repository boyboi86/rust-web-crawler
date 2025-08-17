//! Task identity management
//! Level 3 implementation: TaskIdentity

use crate::common::primitives::{SessionId, TaskId, UrlString};
use serde::{Deserialize, Serialize};

/// Building block for task identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskIdentity {
    id: TaskId,
    session_id: SessionId,
    url: UrlString,
}

impl TaskIdentity {
    pub fn new(id: TaskId, session_id: SessionId, url: UrlString) -> Self {
        Self {
            id,
            session_id,
            url,
        }
    }

    pub fn id(&self) -> &TaskId {
        &self.id
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn url(&self) -> &UrlString {
        &self.url
    }

    pub fn belongs_to_session(&self, session_id: &SessionId) -> bool {
        &self.session_id == session_id
    }

    pub fn has_id(&self, task_id: &TaskId) -> bool {
        &self.id == task_id
    }

    pub fn matches_url(&self, url: &UrlString) -> bool {
        &self.url == url
    }

    pub fn url_str(&self) -> &str {
        self.url.as_str()
    }

    pub fn clone_id(&self) -> TaskId {
        self.id.clone()
    }

    pub fn clone_session_id(&self) -> SessionId {
        self.session_id.clone()
    }

    pub fn clone_url(&self) -> UrlString {
        self.url.clone()
    }
}

impl PartialEq for TaskIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.session_id == other.session_id
    }
}

impl Eq for TaskIdentity {}

impl std::hash::Hash for TaskIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.session_id.hash(state);
    }
}

impl std::fmt::Display for TaskIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task[{}] in Session[{}] for URL[{}]", 
               self.id.as_str(), 
               self.session_id.as_str(), 
               self.url.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_identity_creation() {
        let task_id = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity = TaskIdentity::new(task_id.clone(), session_id.clone(), url.clone());

        assert_eq!(identity.id(), &task_id);
        assert_eq!(identity.session_id(), &session_id);
        assert_eq!(identity.url(), &url);
    }

    #[test]
    fn test_task_identity_comparisons() {
        let task_id1 = TaskId::generate();
        let task_id2 = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity1 = TaskIdentity::new(task_id1.clone(), session_id.clone(), url.clone());
        let identity2 = TaskIdentity::new(task_id2.clone(), session_id.clone(), url.clone());

        // Different task IDs should not be equal
        assert_ne!(identity1, identity2);

        // Same task ID and session should be equal
        let identity3 = TaskIdentity::new(task_id1.clone(), session_id.clone(), url.clone());
        assert_eq!(identity1, identity3);
    }

    #[test]
    fn test_task_identity_belongs_to_session() {
        let task_id = TaskId::generate();
        let session_id1 = SessionId::generate();
        let session_id2 = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity = TaskIdentity::new(task_id, session_id1.clone(), url);

        assert!(identity.belongs_to_session(&session_id1));
        assert!(!identity.belongs_to_session(&session_id2));
    }

    #[test]
    fn test_task_identity_has_id() {
        let task_id1 = TaskId::generate();
        let task_id2 = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity = TaskIdentity::new(task_id1.clone(), session_id, url);

        assert!(identity.has_id(&task_id1));
        assert!(!identity.has_id(&task_id2));
    }

    #[test]
    fn test_task_identity_matches_url() {
        let task_id = TaskId::generate();
        let session_id = SessionId::generate();
        let url1 = UrlString::new("https://example.com").unwrap();
        let url2 = UrlString::new("https://other.com").unwrap();

        let identity = TaskIdentity::new(task_id, session_id, url1.clone());

        assert!(identity.matches_url(&url1));
        assert!(!identity.matches_url(&url2));
    }

    #[test]
    fn test_task_identity_url_str() {
        let task_id = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com/path").unwrap();

        let identity = TaskIdentity::new(task_id, session_id, url);

        assert_eq!(identity.url_str(), "https://example.com/path");
    }

    #[test]
    fn test_task_identity_cloning() {
        let task_id = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity = TaskIdentity::new(task_id.clone(), session_id.clone(), url.clone());

        assert_eq!(identity.clone_id(), task_id);
        assert_eq!(identity.clone_session_id(), session_id);
        assert_eq!(identity.clone_url(), url);
    }

    #[test]
    fn test_task_identity_display() {
        let task_id = TaskId::generate();
        let session_id = SessionId::generate();
        let url = UrlString::new("https://example.com").unwrap();

        let identity = TaskIdentity::new(task_id, session_id, url);
        let display_str = identity.to_string();

        assert!(display_str.contains("Task["));
        assert!(display_str.contains("Session["));
        assert!(display_str.contains("URL["));
        assert!(display_str.contains("https://example.com"));
    }
}
