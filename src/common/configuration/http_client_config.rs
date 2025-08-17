use crate::common::primitives::{BooleanFlag, LimitValue, TimeoutDuration};
use crate::common::timing::TimingConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientConfig {
    timing: TimingConfig,
    user_agent: String,
    follow_redirects: BooleanFlag,
    max_redirects: LimitValue,
    compression: BooleanFlag,
    cookies: BooleanFlag,
}

impl HttpClientConfig {
    pub fn new(
        timing: TimingConfig,
        user_agent: String,
        follow_redirects: BooleanFlag,
        max_redirects: LimitValue,
        compression: BooleanFlag,
        cookies: BooleanFlag,
    ) -> Self {
        Self {
            timing,
            user_agent,
            follow_redirects,
            max_redirects,
            compression,
            cookies,
        }
    }
    pub fn timing(&self) -> &TimingConfig {
        &self.timing
    }
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
    pub fn should_follow_redirects(&self) -> bool {
        self.follow_redirects.is_enabled()
    }
    pub fn max_redirects(&self) -> u64 {
        self.max_redirects.value()
    }
    pub fn is_compression_enabled(&self) -> bool {
        self.compression.is_enabled()
    }
    pub fn are_cookies_enabled(&self) -> bool {
        self.cookies.is_enabled()
    }
    pub fn timeout(&self) -> TimeoutDuration {
        self.timing.timeout()
    }
    pub fn keepalive(&self) -> TimeoutDuration {
        self.timing.keepalive()
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timing: TimingConfig::default(),
            user_agent: "RustCrawler/1.0".to_string(),
            follow_redirects: BooleanFlag::enabled(),
            max_redirects: LimitValue::new(10),
            compression: BooleanFlag::enabled(),
            cookies: BooleanFlag::disabled(),
        }
    }
}
