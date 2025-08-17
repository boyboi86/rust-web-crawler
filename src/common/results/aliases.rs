use crate::common::results::task_content::TaskContent;
use crate::common::results::task_error::TaskError;
use crate::common::results::task_result::TaskResult;

pub type CrawlResult = TaskResult<TaskContent>;
pub type NetworkResult<T> = Result<T, TaskError>;
pub type ConfigResult<T> = Result<T, TaskError>;
pub type ProcessingResult<T> = Result<T, TaskError>;
