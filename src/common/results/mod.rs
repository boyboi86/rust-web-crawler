pub mod aliases;
pub mod error_severity;
pub mod session_result;
pub mod task_content;
pub mod task_error;
pub mod task_result;

pub use aliases::{ConfigResult, CrawlResult, NetworkResult, ProcessingResult};
pub use error_severity::ErrorSeverity;
pub use session_result::SessionResult;
pub use task_content::TaskContent;
pub use task_error::TaskError;
pub use task_result::TaskResult;
