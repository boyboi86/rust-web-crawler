use crate::common::primitives::{DelayDuration, TimeoutDuration};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

fn serialize_instant<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let duration_since_epoch =
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration,
            Err(_) => Duration::ZERO,
        };
    duration_since_epoch.serialize(serializer)
}

fn deserialize_instant<'de, D>(deserializer: D) -> Result<Instant, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _duration = Duration::deserialize(deserializer)?;
    Ok(Instant::now())
}

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
pub struct TaskTiming {
    #[serde(
        serialize_with = "serialize_instant",
        deserialize_with = "deserialize_instant"
    )]
    created_at: Instant,
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    started_at: Option<Instant>,
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    last_attempt_at: Option<Instant>,
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    delay_until: Option<Instant>,
}

impl TaskTiming {
    pub fn new() -> Self {
        Self {
            created_at: Instant::now(),
            started_at: None,
            last_attempt_at: None,
            delay_until: None,
        }
    }
    pub fn created_at(&self) -> Instant {
        self.created_at
    }
    pub fn started_at(&self) -> Option<Instant> {
        self.started_at
    }
    pub fn last_attempt_at(&self) -> Option<Instant> {
        self.last_attempt_at
    }
    pub fn delay_until(&self) -> Option<Instant> {
        self.delay_until
    }
    pub fn mark_started(&mut self) {
        self.started_at = Some(Instant::now());
    }
    pub fn mark_attempt(&mut self) {
        self.last_attempt_at = Some(Instant::now());
    }
    pub fn set_delay(&mut self, delay: DelayDuration) {
        self.delay_until = Some(Instant::now() + delay.duration());
    }
    pub fn is_ready(&self) -> bool {
        match self.delay_until {
            Some(delay_until) => Instant::now() >= delay_until,
            None => true,
        }
    }
    pub fn elapsed_since_created(&self) -> Duration {
        self.created_at.elapsed()
    }
    pub fn elapsed_since_started(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }
}

impl Default for TaskTiming {
    fn default() -> Self {
        Self::new()
    }
}
