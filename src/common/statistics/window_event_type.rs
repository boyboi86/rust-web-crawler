#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEventType {
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskRetrying,
    TaskDead,
}
