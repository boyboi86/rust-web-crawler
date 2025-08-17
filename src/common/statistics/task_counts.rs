use crate::common::primitives::{CountValue, PercentageValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCounts {
    total: CountValue,
    pending: CountValue,
    in_progress: CountValue,
    completed: CountValue,
    failed: CountValue,
    retrying: CountValue,
    dead: CountValue,
}

impl TaskCounts {
    pub fn new() -> Self {
        Self {
            total: CountValue::default(),
            pending: CountValue::default(),
            in_progress: CountValue::default(),
            completed: CountValue::default(),
            failed: CountValue::default(),
            retrying: CountValue::default(),
            dead: CountValue::default(),
        }
    }
    pub fn total(&self) -> u64 {
        self.total.value()
    }
    pub fn pending(&self) -> u64 {
        self.pending.value()
    }
    pub fn in_progress(&self) -> u64 {
        self.in_progress.value()
    }
    pub fn completed(&self) -> u64 {
        self.completed.value()
    }
    pub fn failed(&self) -> u64 {
        self.failed.value()
    }
    pub fn retrying(&self) -> u64 {
        self.retrying.value()
    }
    pub fn dead(&self) -> u64 {
        self.dead.value()
    }
    pub fn increment_total(&mut self) {
        self.total.increment();
    }
    pub fn increment_pending(&mut self) {
        self.pending.increment();
    }
    pub fn increment_in_progress(&mut self) {
        self.in_progress.increment();
    }
    pub fn increment_completed(&mut self) {
        self.completed.increment();
    }
    pub fn increment_failed(&mut self) {
        self.failed.increment();
    }
    pub fn increment_retrying(&mut self) {
        self.retrying.increment();
    }
    pub fn increment_dead(&mut self) {
        self.dead.increment();
    }
    pub fn success_rate(&self) -> PercentageValue {
        if self.total.value() == 0 {
            PercentageValue::default()
        } else {
            let rate = (self.completed.value() as f64 / self.total.value() as f64) * 100.0;
            PercentageValue::new(rate)
        }
    }
    pub fn failure_rate(&self) -> PercentageValue {
        if self.total.value() == 0 {
            PercentageValue::default()
        } else {
            let rate = ((self.failed.value() + self.dead.value()) as f64
                / self.total.value() as f64)
                * 100.0;
            PercentageValue::new(rate)
        }
    }
    pub fn reset(&mut self) {
        self.total.reset();
        self.pending.reset();
        self.in_progress.reset();
        self.completed.reset();
        self.failed.reset();
        self.retrying.reset();
        self.dead.reset();
    }
}

impl Default for TaskCounts {
    fn default() -> Self {
        Self::new()
    }
}
