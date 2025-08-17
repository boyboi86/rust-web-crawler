use crate::common::primitives::{BooleanFlag, DelayDuration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    enabled: BooleanFlag,
    proxy_urls: Vec<String>,
    rotation_enabled: BooleanFlag,
    health_check_enabled: BooleanFlag,
    health_check_interval: DelayDuration,
    retry_failed: BooleanFlag,
}

impl ProxyConfig {
    pub fn new(
        enabled: BooleanFlag,
        proxy_urls: Vec<String>,
        rotation_enabled: BooleanFlag,
        health_check_enabled: BooleanFlag,
        health_check_interval: DelayDuration,
        retry_failed: BooleanFlag,
    ) -> Self {
        Self {
            enabled,
            proxy_urls,
            rotation_enabled,
            health_check_enabled,
            health_check_interval,
            retry_failed,
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.is_enabled()
    }
    pub fn proxy_urls(&self) -> &[String] {
        &self.proxy_urls
    }
    pub fn is_rotation_enabled(&self) -> bool {
        self.rotation_enabled.is_enabled()
    }
    pub fn is_health_check_enabled(&self) -> bool {
        self.health_check_enabled.is_enabled()
    }
    pub fn health_check_interval(&self) -> DelayDuration {
        self.health_check_interval
    }
    pub fn should_retry_failed(&self) -> bool {
        self.retry_failed.is_enabled()
    }
    pub fn add_proxy(&mut self, proxy_url: String) {
        self.proxy_urls.push(proxy_url);
    }
    pub fn remove_proxy(&mut self, proxy_url: &str) {
        self.proxy_urls.retain(|url| url != proxy_url);
    }
    pub fn has_proxies(&self) -> bool {
        !self.proxy_urls.is_empty()
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: BooleanFlag::disabled(),
            proxy_urls: Vec::new(),
            rotation_enabled: BooleanFlag::enabled(),
            health_check_enabled: BooleanFlag::enabled(),
            health_check_interval: DelayDuration::from_secs(300),
            retry_failed: BooleanFlag::enabled(),
        }
    }
}
