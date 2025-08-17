use crate::common::primitives::{TaskId, UrlString};
use crate::common::results::task_content::TaskContent;
use crate::common::results::task_error::TaskError;
use crate::common::timing::ExecutionTiming;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult<T> {
    task_id: TaskId,
    url: UrlString,
    result: Result<T, TaskError>,
    timing: ExecutionTiming,
    metadata: HashMap<String, String>,
}

impl<T> TaskResult<T> {
    pub fn success(task_id: TaskId, url: UrlString, data: T, timing: ExecutionTiming) -> Self {
        Self {
            task_id,
            url,
            result: Ok(data),
            timing,
            metadata: HashMap::new(),
        }
    }
    pub fn failure(
        task_id: TaskId,
        url: UrlString,
        error: TaskError,
        timing: ExecutionTiming,
    ) -> Self {
        Self {
            task_id,
            url,
            result: Err(error),
            timing,
            metadata: HashMap::new(),
        }
    }
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }
    pub fn url(&self) -> &UrlString {
        &self.url
    }
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
    pub fn data(&self) -> Option<&T> {
        self.result.as_ref().ok()
    }
    pub fn error(&self) -> Option<&TaskError> {
        self.result.as_ref().err()
    }
    pub fn timing(&self) -> &ExecutionTiming {
        &self.timing
    }
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    pub fn into_result(self) -> Result<T, TaskError> {
        self.result
    }
}
