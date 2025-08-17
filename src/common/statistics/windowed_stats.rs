use crate::common::primitives::PercentageValue;
use crate::common::statistics::task_counts::TaskCounts;
use serde::{Deserialize, Serialize};

/// Helper function to serialize Instant as SystemTime
fn serialize_instant<S>(instant: &std::time::Instant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration_since_epoch =
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration,
            Err(_) => std::time::Duration::ZERO,
        };
    duration_since_epoch.serialize(serializer)
}

/// Helper function to deserialize SystemTime as Instant
fn deserialize_instant<'de, D>(deserializer: D) -> Result<std::time::Instant, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _duration = std::time::Duration::deserialize(deserializer)?;
    Ok(std::time::Instant::now())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowedStats {
    current_window: TaskCounts,
    previous_window: TaskCounts,
    window_duration_secs: u64,
    #[serde(
        serialize_with = "serialize_instant",
        deserialize_with = "deserialize_instant"
    )]
    window_start: std::time::Instant,
}

impl WindowedStats {
    pub fn new(window_duration_secs: u64) -> Self {
        Self {
            current_window: TaskCounts::new(),
            previous_window: TaskCounts::new(),
            window_duration_secs,
            window_start: std::time::Instant::now(),
        }
    }
    pub fn record_event(&mut self, event_type: super::window_event_type::WindowEventType) {
        self.check_window_rotation();
        match event_type {
            super::window_event_type::WindowEventType::TaskStarted => {
                self.current_window.increment_in_progress()
            }
            super::window_event_type::WindowEventType::TaskCompleted => {
                self.current_window.increment_completed()
            }
            super::window_event_type::WindowEventType::TaskFailed => {
                self.current_window.increment_failed()
            }
            super::window_event_type::WindowEventType::TaskRetrying => {
                self.current_window.increment_retrying()
            }
            super::window_event_type::WindowEventType::TaskDead => {
                self.current_window.increment_dead()
            }
        }
        self.current_window.increment_total();
    }
    pub fn current_window(&self) -> &TaskCounts {
        &self.current_window
    }
    pub fn previous_window(&self) -> &TaskCounts {
        &self.previous_window
    }
    pub fn window_progress(&self) -> PercentageValue {
        let elapsed = self.window_start.elapsed().as_secs();
        let progress = (elapsed as f64 / self.window_duration_secs as f64) * 100.0;
        PercentageValue::new(progress.min(100.0))
    }
    fn check_window_rotation(&mut self) {
        if self.window_start.elapsed().as_secs() >= self.window_duration_secs {
            self.previous_window = self.current_window.clone();
            self.current_window = TaskCounts::new();
            self.window_start = std::time::Instant::now();
        }
    }
}

impl Default for WindowedStats {
    fn default() -> Self {
        Self::new(60)
    }
}
