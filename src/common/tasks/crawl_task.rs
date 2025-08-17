//! Complete crawl task composed of building blocks
use crate::common::primitives::{SessionId, TaskId, UrlString};
use crate::common::tasks::{TaskIdentity, TaskPriority, TaskStatus};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    status: TaskStatus,
    priority: TaskPriority,
    priority_score: crate::common::primitives::PriorityScore,
    attempt_count: crate::common::primitives::AttemptCount,
    metadata: HashMap<String, String>,
}

impl TaskState {
    pub fn new(priority: TaskPriority) -> Self {
        Self {
            status: TaskStatus::default(),
            priority,
            priority_score: priority.as_score(),
            attempt_count: crate::common::primitives::AttemptCount::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn priority_score(&self) -> crate::common::primitives::PriorityScore {
        self.priority_score
    }

    pub fn attempt_count(&self) -> u32 {
        self.attempt_count.value()
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    pub fn increment_attempt(&mut self) {
        self.attempt_count.increment();
    }

    pub fn adjust_priority(&mut self, adjustment: f64) {
        self.priority_score.adjust(adjustment);
        self.priority = TaskPriority::from_score(self.priority_score.value());
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl Default for TaskState {
    fn default() -> Self {
        Self::new(TaskPriority::default())
    }
}
use crate::common::timing::TaskTiming;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlTask {
    pub(crate) identity: TaskIdentity,
    pub(crate) state: TaskState,
    pub(crate) timing: TaskTiming,
}

impl CrawlTask {
    pub fn builder() -> crate::common::tasks::CrawlTaskBuilder {
        crate::common::tasks::CrawlTaskBuilder::new()
    }

    pub fn identity(&self) -> &TaskIdentity {
        &self.identity
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut TaskState {
        &mut self.state
    }

    pub fn timing(&self) -> &TaskTiming {
        &self.timing
    }

    pub fn timing_mut(&mut self) -> &mut TaskTiming {
        &mut self.timing
    }

    pub fn id(&self) -> &TaskId {
        self.identity.id()
    }

    pub fn session_id(&self) -> &SessionId {
        self.identity.session_id()
    }

    pub fn url(&self) -> &UrlString {
        self.identity.url()
    }

    pub fn status(&self) -> &TaskStatus {
        self.state.status()
    }

    pub fn priority(&self) -> TaskPriority {
        self.state.priority()
    }

    pub fn priority_score(&self) -> crate::common::primitives::PriorityScore {
        self.state.priority_score()
    }

    pub fn attempt_count(&self) -> u32 {
        self.state.attempt_count()
    }

    pub fn start(&mut self) {
        self.state.set_status(TaskStatus::InProgress);
        self.timing.mark_started();
    }

    pub fn complete(&mut self) {
        self.state.set_status(TaskStatus::Completed);
    }

    pub fn fail(&mut self) {
        self.state.set_status(TaskStatus::Failed);
        self.state.increment_attempt();
        self.timing.mark_attempt();
    }

    pub fn retry(&mut self) {
        self.state.set_status(TaskStatus::Retrying);
        self.state.increment_attempt();
        self.timing.mark_attempt();
    }

    pub fn mark_dead(&mut self) {
        self.state.set_status(TaskStatus::Dead);
    }

    pub fn is_ready(&self) -> bool {
        matches!(
            self.state.status(),
            TaskStatus::Pending | TaskStatus::Retrying
        ) && self.timing.is_ready()
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.state.status(), TaskStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.state.status(), TaskStatus::Failed | TaskStatus::Dead)
    }
}
