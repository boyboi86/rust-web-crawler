use crate::common::primitives::DelayDuration;
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

fn deserialize_option_instant<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _duration: Option<Duration> = Option::deserialize(deserializer)?;
    Ok(_duration.map(|_| Instant::now()))
}

fn deserialize_instant<'de, D>(deserializer: D) -> Result<Instant, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let _duration = Duration::deserialize(deserializer)?;
    Ok(Instant::now())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitTiming {
    #[serde(
        serialize_with = "serialize_option_instant",
        deserialize_with = "deserialize_option_instant"
    )]
    last_request: Option<Instant>,
    #[serde(
        serialize_with = "serialize_instant",
        deserialize_with = "deserialize_instant"
    )]
    window_start: Instant,
    request_interval: DelayDuration,
}

impl RateLimitTiming {
    pub fn new(request_interval: DelayDuration) -> Self {
        Self {
            last_request: None,
            window_start: Instant::now(),
            request_interval,
        }
    }
    pub fn can_make_request(&self) -> bool {
        match self.last_request {
            Some(last) => last.elapsed() >= self.request_interval.duration(),
            None => true,
        }
    }
    pub fn mark_request(&mut self) {
        self.last_request = Some(Instant::now());
    }
    pub fn time_until_next_request(&self) -> Option<Duration> {
        self.last_request.map(|last| {
            let elapsed = last.elapsed();
            self.request_interval.duration().saturating_sub(elapsed)
        })
    }
    pub fn reset_window(&mut self) {
        self.window_start = Instant::now();
        self.last_request = None;
    }
    pub fn window_elapsed(&self) -> Duration {
        self.window_start.elapsed()
    }
}

impl Default for RateLimitTiming {
    fn default() -> Self {
        Self::new(DelayDuration::from_millis(100))
    }
}
