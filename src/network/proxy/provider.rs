/// ProxyProvider building block - unified interface for all proxy sources
/// Implements the building block pattern for single source of truth
use crate::common::building_blocks::{ApiParameterSet, ReqwestClient, RetryPolicy};
use crate::core::types::Region;
use std::collections::HashMap;
use tokio::time::Duration;

/// ProxyProvider error types
#[derive(Debug)]
pub enum ProxyProviderError {
    NetworkError(String),
    ParseError(String),
    RateLimited,
    Unauthorized,
    ConfigurationError(String),
}

/// ProxyInfo structure with comprehensive details
#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub url: String,
    pub region: Region,
    pub protocol: ProxyProtocol,
    pub speed_ms: Option<u32>,
    pub success_rate: f64,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks4,
    Socks5,
}

/// Configuration for proxy providers - single source of truth
#[derive(Debug, Clone)]
pub struct ProxyProviderConfig {
    pub free_sources: Vec<String>,
    pub paid_sources: HashMap<String, ApiCredentials>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub rate_limit_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ApiCredentials {
    pub api_key: String,
    pub endpoint: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyProviderConfig {
    fn default() -> Self {
        Self {
            free_sources: vec![
                "https://www.proxy-list.download/api/v1/get?type=http".to_string(),
                "https://api.proxyscrape.com/v2/?request=get&protocol=http".to_string(),
            ],
            paid_sources: HashMap::new(),
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_ms: 1000,
        }
    }
}

/// Building block trait for all proxy sources
#[async_trait::async_trait]
pub trait ProxySource: Send + Sync {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyProviderError>;
    fn source_name(&self) -> &str;
    fn is_paid(&self) -> bool;
    fn max_requests_per_day(&self) -> Option<u32>;
}

/// Unified ProxyProvider - main building block
pub struct ProxyProvider {
    sources: Vec<Box<dyn ProxySource>>,
    config: ProxyProviderConfig,
    client: ReqwestClient,
    retry_policy: RetryPolicy,
}

impl ProxyProvider {
    /// Create a new ProxyProvider with configuration
    pub fn new(config: ProxyProviderConfig) -> Self {
        let client = ReqwestClient::with_timeout(Duration::from_secs(config.timeout_seconds));
        let retry_policy = RetryPolicy::new()
            .with_max_attempts(config.max_retries)
            .with_base_delay(Duration::from_millis(config.rate_limit_ms));

        Self {
            sources: Vec::new(),
            config,
            client,
            retry_policy,
        }
    }

    /// Add a free proxy source
    pub fn add_free_source(&mut self, source: Box<dyn ProxySource>) {
        self.sources.push(source);
    }

    /// Add a paid proxy source
    pub fn add_paid_source(&mut self, source: Box<dyn ProxySource>) {
        self.sources.push(source);
    }

    /// Fetch proxies from all sources
    pub async fn fetch_all_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyProviderError> {
        let mut all_proxies = Vec::new();

        for source in &self.sources {
            match source.fetch_proxies().await {
                Ok(mut proxies) => {
                    all_proxies.append(&mut proxies);
                    tracing::info!(
                        "Fetched {} proxies from {}",
                        proxies.len(),
                        source.source_name()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch from {}: {:?}", source.source_name(), e);
                }
            }

            // Rate limiting between sources
            tokio::time::sleep(Duration::from_millis(self.config.rate_limit_ms)).await;
        }

        Ok(all_proxies)
    }

    /// Fetch proxies from specific region
    pub async fn fetch_proxies_by_region(
        &self,
        region: Region,
    ) -> Result<Vec<ProxyInfo>, ProxyProviderError> {
        let all_proxies = self.fetch_all_proxies().await?;
        Ok(all_proxies
            .into_iter()
            .filter(|proxy| proxy.region == region || proxy.region == Region::Global)
            .collect())
    }

    /// Get only paid proxy sources
    pub async fn fetch_premium_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyProviderError> {
        let mut premium_proxies = Vec::new();

        for source in &self.sources {
            if source.is_paid() {
                match source.fetch_proxies().await {
                    Ok(mut proxies) => {
                        premium_proxies.append(&mut proxies);
                        tracing::info!(
                            "Fetched {} premium proxies from {}",
                            proxies.len(),
                            source.source_name()
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to fetch premium from {}: {:?}",
                            source.source_name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(premium_proxies)
    }

    /// Get configuration
    pub fn config(&self) -> &ProxyProviderConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_provider_config() {
        let config = ProxyProviderConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.free_sources.is_empty());
    }

    #[test]
    fn test_proxy_info_creation() {
        let proxy = ProxyInfo {
            url: "http://127.0.0.1:8080".to_string(),
            region: Region::Global,
            protocol: ProxyProtocol::Http,
            speed_ms: Some(100),
            success_rate: 0.95,
            last_checked: chrono::Utc::now(),
            source: "TestSource".to_string(),
        };

        assert_eq!(proxy.url, "http://127.0.0.1:8080");
        assert_eq!(proxy.success_rate, 0.95);
    }
}
