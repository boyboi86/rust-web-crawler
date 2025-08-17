use crate::common::primitives::SessionId;
use crate::common::results::task_content::TaskContent;
use crate::common::results::task_error::TaskError;
use crate::common::results::task_result::TaskResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    session_id: SessionId,
    total_tasks: u64,
    successful_tasks: u64,
    failed_tasks: u64,
    results: Vec<TaskResult<TaskContent>>,
    errors: Vec<TaskError>,
    started_at: std::time::SystemTime,
    completed_at: Option<std::time::SystemTime>,
}

impl SessionResult {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            results: Vec::new(),
            errors: Vec::new(),
            started_at: std::time::SystemTime::now(),
            completed_at: None,
        }
    }
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
    pub fn total_tasks(&self) -> u64 {
        self.total_tasks
    }
    pub fn successful_tasks(&self) -> u64 {
        self.successful_tasks
    }
    pub fn failed_tasks(&self) -> u64 {
        self.failed_tasks
    }
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.successful_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }
    pub fn results(&self) -> &[TaskResult<TaskContent>] {
        &self.results
    }
    pub fn errors(&self) -> &[TaskError] {
        &self.errors
    }
    pub fn add_result(&mut self, result: TaskResult<TaskContent>) {
        if result.is_success() {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
            if let Some(error) = result.error() {
                self.errors.push(error.clone());
            }
        }
        self.results.push(result);
        self.total_tasks += 1;
    }
    pub fn complete(&mut self) {
        self.completed_at = Some(std::time::SystemTime::now());
    }
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.completed_at
            .and_then(|completed| completed.duration_since(self.started_at).ok())
    }
}

impl Default for SessionResult {
    fn default() -> Self {
        Self::new(SessionId::new("default"))
    }
}
