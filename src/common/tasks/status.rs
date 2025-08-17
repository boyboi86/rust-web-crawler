//! Task status tracking
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Retrying,
    Dead, // Exceeded max retries
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}
