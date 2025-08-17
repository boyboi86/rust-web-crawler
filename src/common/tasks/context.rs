//! Task execution context
use crate::common::results::{TaskError, TaskResult};
use crate::common::tasks::CrawlTask;
use crate::common::timing::ExecutionTiming;

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub(crate) task: CrawlTask,
    pub(crate) execution_timing: ExecutionTiming,
    pub(crate) retry_errors: Vec<TaskError>,
}

impl TaskContext {
    pub fn new(task: CrawlTask) -> Self {
        Self {
            task,
            execution_timing: ExecutionTiming::new(),
            retry_errors: Vec::new(),
        }
    }

    pub fn task(&self) -> &CrawlTask {
        &self.task
    }

    pub fn task_mut(&mut self) -> &mut CrawlTask {
        &mut self.task
    }

    pub fn execution_timing(&self) -> &ExecutionTiming {
        &self.execution_timing
    }

    pub fn retry_errors(&self) -> &[TaskError] {
        &self.retry_errors
    }

    pub fn start_execution(&mut self) {
        self.task.start();
        self.execution_timing.start();
    }

    pub fn complete_execution(&mut self) {
        self.task.complete();
        self.execution_timing.complete();
    }

    pub fn fail_execution(&mut self, error: TaskError) {
        self.retry_errors.push(error);
        self.task.fail();
        self.execution_timing.complete();
    }

    pub fn prepare_retry(&mut self, error: TaskError) {
        self.retry_errors.push(error);
        self.task.retry();
        self.execution_timing = ExecutionTiming::new(); // Reset for retry
    }

    pub fn into_result<T>(self, data: Option<T>) -> TaskResult<T> {
        match data {
            Some(content) => TaskResult::success(
                self.task.id().clone(),
                self.task.url().clone(),
                content,
                self.execution_timing,
            ),
            None => {
                let error = match self.retry_errors.into_iter().last() {
                    Some(err) => err,
                    None => TaskError::internal("Task failed without specific error"),
                };
                TaskResult::failure(
                    self.task.id().clone(),
                    self.task.url().clone(),
                    error,
                    self.execution_timing,
                )
            }
        }
    }
}
