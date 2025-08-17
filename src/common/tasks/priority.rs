//! Task priority levels and conversion utilities
use crate::common::primitives::PriorityScore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    Medium = 3,
    High = 4,
    Critical = 5,
}

impl TaskPriority {
    pub fn as_score(&self) -> PriorityScore {
        PriorityScore::new(*self as i32 as f64)
    }

    pub fn from_score(score: f64) -> Self {
        match score as i32 {
            1 => TaskPriority::Low,
            2 => TaskPriority::Normal,
            3 => TaskPriority::Medium,
            4 => TaskPriority::High,
            5.. => TaskPriority::Critical,
            _ => TaskPriority::Low,
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}
