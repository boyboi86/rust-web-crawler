//! Builder for creating crawl tasks
use crate::common::primitives::{SessionId, TaskId, UrlString};
use crate::common::results::TaskError;
use crate::common::tasks::crawl_task::TaskState;
use crate::common::tasks::{CrawlTask, TaskPriority};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CrawlTaskBuilder {
    id: Option<TaskId>,
    session_id: Option<SessionId>,
    url: Option<UrlString>,
    priority: TaskPriority,
    metadata: HashMap<String, String>,
}

impl CrawlTaskBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            session_id: None,
            url: None,
            priority: TaskPriority::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_id(mut self, id: TaskId) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_url(mut self, url: UrlString) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn build(self) -> Result<CrawlTask, TaskError> {
        let id = self
            .id
            .ok_or_else(|| TaskError::configuration("Task ID is required"))?;
        let session_id = self
            .session_id
            .ok_or_else(|| TaskError::configuration("Session ID is required"))?;
        let url = self
            .url
            .ok_or_else(|| TaskError::configuration("URL is required"))?;

        let identity = crate::common::tasks::TaskIdentity::new(id, session_id, url);
        let mut state = TaskState::new(self.priority);

        // Add metadata to state
        for (key, value) in self.metadata {
            state.add_metadata(key, value);
        }

        let timing = crate::common::timing::TaskTiming::new();

        Ok(CrawlTask {
            identity,
            state,
            timing,
        })
    }
}

impl Default for CrawlTaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}
