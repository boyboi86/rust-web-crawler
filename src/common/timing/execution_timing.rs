use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

fn serialize_option_instant<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match instant {
        Some(_) => {
            let duration_since_epoch =
                match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(duration) => duration,
                    Err(_) => Duration::ZERO,
                };
            Some(duration_since_epoch).serialize(serializer)
        }
        None => None::<Duration>.serialize(serializer),
    }
}

fn deserialize_option_instant<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _duration: Option<Duration> = Option::deserialize(deserializer)?;
    Ok(_duration.map(|_| Instant::now()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTiming {
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    started_at: Option<Instant>,
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    completed_at: Option<Instant>,
    processing_time: Duration,
}

impl ExecutionTiming {
    pub fn new() -> Self {
        Self {
            started_at: None,
            completed_at: None,
            processing_time: Duration::default(),
        }
    }
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
    }
    pub fn complete(&mut self) {
        let now = Instant::now();
        self.completed_at = Some(now);
        if let Some(started) = self.started_at {
            self.processing_time = now.duration_since(started);
        }
    }
    pub fn started_at(&self) -> Option<Instant> {
        self.started_at
    }
    pub fn completed_at(&self) -> Option<Instant> {
        self.completed_at
    }
    pub fn processing_time(&self) -> Duration {
        self.processing_time
    }
    pub fn total_duration(&self) -> Duration {
        self.processing_time
    }
    pub fn is_running(&self) -> bool {
        self.started_at.is_some() && self.completed_at.is_none()
    }
    pub fn is_completed(&self) -> bool {
        self.started_at.is_some() && self.completed_at.is_some()
    }
}

impl Default for ExecutionTiming {
    fn default() -> Self {
        Self::new()
    }
}
