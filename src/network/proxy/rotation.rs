use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Proxy ignore configuration similar to Spider-RS
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyIgnore {
    /// Skip Chrome-based requests
    Chrome,
    /// Skip HTTP requests
    Http,
    /// Don't ignore any requests
    No,
}

impl Default for ProxyIgnore {
    fn default() -> Self {
        ProxyIgnore::No
    }
}

/// Enhanced proxy configuration matching Spider-RS patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestProxy {
    /// The proxy address (http://ip:port, socks5://ip:port, etc.)
    pub addr: String,
    /// Which request types to ignore this proxy for
    pub ignore: ProxyIgnore,
    /// Whether this proxy is currently working (health status)
    pub healthy: bool,
    /// Last successful use timestamp
    pub last_success: Option<std::time::SystemTime>,
    /// Failure count for circuit breaker pattern
    pub failure_count: u32,
}

impl RequestProxy {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            ignore: ProxyIgnore::No,
            healthy: true,
            last_success: None,
            failure_count: 0,
        }
    }

    pub fn with_ignore(mut self, ignore: ProxyIgnore) -> Self {
        self.ignore = ignore;
        self
    }

    /// Check if this proxy should be used for HTTP requests
    pub fn should_use_for_http(&self) -> bool {
        self.healthy && self.ignore != ProxyIgnore::Http
    }

    /// Check if this proxy should be used for Chrome requests
    pub fn should_use_for_chrome(&self) -> bool {
        self.healthy && self.ignore != ProxyIgnore::Chrome
    }

    /// Mark proxy as failed
    pub fn mark_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= 3 {
            self.healthy = false;
        }
    }

    /// Mark proxy as successful
    pub fn mark_success(&mut self) {
        self.failure_count = 0;
        self.healthy = true;
        self.last_success = Some(std::time::SystemTime::now());
    }

    /// Get cleaned proxy URL for reqwest (handle SOCKS on Linux)
    pub fn get_reqwest_url(&self) -> String {
        // Spider-RS pattern: replace socks:// with http:// on Linux since reqwest doesn't support SOCKS
        #[cfg(target_os = "linux")]
        {
            if self.addr.starts_with("socks://") {
                return self.addr.replacen("socks://", "http://", 1);
            }
        }
        self.addr.clone()
    }
}

/// Proxy rotation manager following Spider-RS patterns
#[derive(Debug)]
pub struct ProxyRotationManager {
    proxies: Arc<tokio::sync::RwLock<Vec<RequestProxy>>>,
    current_index: Arc<AtomicUsize>,
}

impl ProxyRotationManager {
    pub fn new(proxies: Vec<RequestProxy>) -> Self {
        Self {
            proxies: Arc::new(tokio::sync::RwLock::new(proxies)),
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get all healthy HTTP proxies (Spider-RS style: all at once)
    pub async fn get_http_proxies(&self) -> Vec<String> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .filter(|p| p.should_use_for_http())
            .map(|p| p.get_reqwest_url())
            .collect()
    }

    /// Get all healthy Chrome proxies
    pub async fn get_chrome_proxies(&self) -> Vec<String> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .filter(|p| p.should_use_for_chrome())
            .map(|p| p.addr.clone())
            .collect()
    }

    /// Get next proxy for round-robin (if we want traditional rotation)
    pub async fn get_next_proxy(&self) -> Option<RequestProxy> {
        let proxies = self.proxies.read().await;
        let healthy_proxies: Vec<_> = proxies.iter().filter(|p| p.healthy).collect();

        if healthy_proxies.is_empty() {
            return None;
        }

        let index = self.current_index.fetch_add(1, Ordering::SeqCst) % healthy_proxies.len();
        healthy_proxies.get(index).map(|&p| p.clone())
    }

    /// Mark a proxy as failed
    pub async fn mark_proxy_failed(&self, addr: &str) {
        let mut proxies = self.proxies.write().await;
        if let Some(proxy) = proxies.iter_mut().find(|p| p.addr == addr) {
            proxy.mark_failure();
        }
    }

    /// Mark a proxy as successful
    pub async fn mark_proxy_success(&self, addr: &str) {
        let mut proxies = self.proxies.write().await;
        if let Some(proxy) = proxies.iter_mut().find(|p| p.addr == addr) {
            proxy.mark_success();
        }
    }

    /// Add new proxies to the rotation
    pub async fn add_proxies(&self, new_proxies: Vec<RequestProxy>) {
        let mut proxies = self.proxies.write().await;
        for new_proxy in new_proxies {
            // Don't add duplicates
            if !proxies.iter().any(|p| p.addr == new_proxy.addr) {
                proxies.push(new_proxy);
            }
        }
    }

    /// Get health status of all proxies
    pub async fn get_health_status(&self) -> Vec<(String, bool, u32)> {
        let proxies = self.proxies.read().await;
        proxies
            .iter()
            .map(|p| (p.addr.clone(), p.healthy, p.failure_count))
            .collect()
    }

    /// Remove unhealthy proxies
    pub async fn cleanup_unhealthy(&self) {
        let mut proxies = self.proxies.write().await;
        proxies.retain(|p| p.healthy || p.failure_count < 5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_ignore_patterns() {
        let http_proxy = RequestProxy::new("http://proxy1.com:8080".to_string());
        assert!(http_proxy.should_use_for_http());
        assert!(http_proxy.should_use_for_chrome());

        let chrome_only =
            RequestProxy::new("http://proxy2.com:8080".to_string()).with_ignore(ProxyIgnore::Http);
        assert!(!chrome_only.should_use_for_http());
        assert!(chrome_only.should_use_for_chrome());

        let http_only = RequestProxy::new("http://proxy3.com:8080".to_string())
            .with_ignore(ProxyIgnore::Chrome);
        assert!(http_only.should_use_for_http());
        assert!(!http_only.should_use_for_chrome());
    }

    #[test]
    fn test_socks_url_conversion() {
        let socks_proxy = RequestProxy::new("socks://127.0.0.1:1080".to_string());

        #[cfg(target_os = "linux")]
        {
            assert_eq!(socks_proxy.get_reqwest_url(), "http://127.0.0.1:1080");
        }

        #[cfg(not(target_os = "linux"))]
        {
            assert_eq!(socks_proxy.get_reqwest_url(), "socks://127.0.0.1:1080");
        }
    }

    #[tokio::test]
    async fn test_rotation_manager() {
        let proxies = vec![
            RequestProxy::new("http://proxy1.com:8080".to_string()),
            RequestProxy::new("http://proxy2.com:8080".to_string()),
            RequestProxy::new("http://proxy3.com:8080".to_string()),
        ];

        let manager = ProxyRotationManager::new(proxies);

        let http_proxies = manager.get_http_proxies().await;
        assert_eq!(http_proxies.len(), 3);

        // Test round robin
        let proxy1 = manager.get_next_proxy().await.unwrap();
        let proxy2 = manager.get_next_proxy().await.unwrap();
        assert_ne!(proxy1.addr, proxy2.addr);
    }
}
