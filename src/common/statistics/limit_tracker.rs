use crate::common::primitives::{CountValue, LimitValue, PercentageValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitTracker {
    current: CountValue,
    limit: LimitValue,
    high_water_mark: CountValue,
}

impl LimitTracker {
    pub fn new(limit: LimitValue) -> Self {
        Self {
            current: CountValue::default(),
            limit,
            high_water_mark: CountValue::default(),
        }
    }
    pub fn current(&self) -> u64 {
        self.current.value()
    }
    pub fn limit(&self) -> u64 {
        self.limit.value()
    }
    pub fn high_water_mark(&self) -> u64 {
        self.high_water_mark.value()
    }
    pub fn is_at_limit(&self) -> bool {
        self.limit.is_exceeded(self.current.value())
    }
    pub fn remaining(&self) -> u64 {
        self.limit.remaining(self.current.value())
    }
    pub fn utilization(&self) -> PercentageValue {
        if self.limit.value() == 0 {
            PercentageValue::default()
        } else {
            let rate = (self.current.value() as f64 / self.limit.value() as f64) * 100.0;
            PercentageValue::new(rate)
        }
    }
    pub fn increment(&mut self) -> bool {
        if !self.is_at_limit() {
            self.current.increment();
            if self.current.value() > self.high_water_mark.value() {
                self.high_water_mark = self.current;
            }
            true
        } else {
            false
        }
    }
    pub fn decrement(&mut self) {
        if self.current.value() > 0 {
            let mut current_val = self.current.value();
            current_val -= 1;
            self.current = CountValue::new(current_val);
        }
    }
    pub fn reset(&mut self) {
        self.current.reset();
    }
    pub fn set_limit(&mut self, limit: LimitValue) {
        self.limit = limit;
    }
}

impl Default for LimitTracker {
    fn default() -> Self {
        Self::new(LimitValue::new(1000))
    }
}
