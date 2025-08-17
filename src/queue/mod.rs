/// Task queue management and caching utilities
/// Following Rule 7: Feature-based organization with max 3-level hierarchy
/// Following Rule 4: Privacy first - controlled exports only
pub mod cache;
pub mod task_queue_refactored;

// Legacy module for compatibility (will be deprecated)
pub mod task_queue;

// Re-export queue components using new refactored implementations
pub use cache::TtlCache;
pub use task_queue_refactored::{
    QueueResult, QueueState, TaskQueue, TaskQueueConfig, TaskQueueConfigBuilder,
};

// Legacy exports for backward compatibility
pub use task_queue::TaskQueue as LegacyTaskQueue;
