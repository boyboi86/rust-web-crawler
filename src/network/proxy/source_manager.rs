// Comprehensive proxy sourcing and management system
use crate::common::building_blocks::{
    ApiParameterSet, ReqwestClient, ResponseValidator, RetryPolicy,
};
use crate::core::types::Region;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, sleep};

/// Comprehensive proxy management with multiple sources
pub struct ProxySourceManager {
    sources: Vec<Box<dyn ProxySource>>,
    health_checker: ProxyHealthChecker,
    rotation_strategy: RotationStrategy,
}

#[async_trait::async_trait]
#[async_trait::async_trait]
pub trait ProxySource {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyError>;
    fn source_name(&self) -> &str;
    fn is_paid(&self) -> bool;
    fn max_requests_per_day(&self) -> Option<u32>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub url: String,
    pub region: Region,
    pub protocol: ProxyProtocol,
    pub speed_ms: Option<u32>,
    pub success_rate: f32,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks5,
}

#[derive(Debug)]
pub enum ProxyError {
    NetworkError(String),
    ParseError(String),
    RateLimited,
    Unauthorized,
}

/// 1. FREE PROXY SOURCES
#[derive(Debug)]
pub struct FreeProxyListSource {
    client: ReqwestClient,
    retry_policy: RetryPolicy,
}

#[async_trait::async_trait]
impl ProxySource for FreeProxyListSource {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyError> {
        // Fetches from free-proxy-list.net API
        let response = self
            .client
            .client()
            .get("https://www.proxy-list.download/api/v1/get?type=http")
            .send()
            .await
            .map_err(|e| ProxyError::NetworkError(e.to_string()))?;

        let proxy_list: String = response
            .text()
            .await
            .map_err(|e| ProxyError::ParseError(e.to_string()))?;

        Ok(self.parse_proxy_list(&proxy_list))
    }

    fn source_name(&self) -> &str {
        "FreeProxyList"
    }

    fn is_paid(&self) -> bool {
        false
    }

    fn max_requests_per_day(&self) -> Option<u32> {
        Some(1000)
    }
}

impl FreeProxyListSource {
    /// Create a new FreeProxyListSource with building blocks
    pub fn new() -> Self {
        Self {
            client: ReqwestClient::with_timeout(Duration::from_secs(10)),
            retry_policy: RetryPolicy::new(),
        }
    }

    fn parse_proxy_list(&self, proxy_list: &str) -> Vec<ProxyInfo> {
        proxy_list
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || !line.contains(':') {
                    return None;
                }

                // Parse IP:PORT format
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    Some(ProxyInfo {
                        url: format!("http://{}:{}", parts[0], parts[1]),
                        region: Region::Global, // Free proxies don't specify region
                        protocol: ProxyProtocol::Http,
                        speed_ms: None,
                        success_rate: 0.5, // Assume 50% for free proxies
                        last_checked: chrono::Utc::now(),
                        source: "FreeProxyList".to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// 2. PREMIUM PROXY SOURCES
#[derive(Debug)]
pub struct BrightDataSource {
    client: ReqwestClient,
    api_params: ApiParameterSet,
    endpoint: String,
    retry_policy: RetryPolicy,
}

#[async_trait::async_trait]
impl ProxySource for BrightDataSource {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyError> {
        // BrightData (formerly Luminati) - Premium residential proxies
        // Provides real residential IP addresses

        // Get the API key from parameters
        let api_key = self
            .api_params
            .to_headers()
            .map_err(|e| ProxyError::ParseError(e))?
            .get("Authorization")
            .ok_or_else(|| ProxyError::ParseError("Missing API key".to_string()))?
            .clone();

        let response = self
            .client
            .client()
            .get(&format!("{}/proxies", self.endpoint))
            .header("Authorization", api_key)
            .send()
            .await
            .map_err(|e| ProxyError::NetworkError(e.to_string()))?;

        if response.status() == 429 {
            return Err(ProxyError::RateLimited);
        }

        let proxies: BrightDataResponse = response
            .json()
            .await
            .map_err(|e| ProxyError::ParseError(e.to_string()))?;

        Ok(proxies.into_proxy_info())
    }

    fn source_name(&self) -> &str {
        "BrightData"
    }
    fn is_paid(&self) -> bool {
        true
    }
    fn max_requests_per_day(&self) -> Option<u32> {
        None
    } // Unlimited for paid
}

impl BrightDataSource {
    /// Create a new BrightDataSource with building blocks
    pub fn new(api_key: String, endpoint: String) -> Self {
        let mut api_params = ApiParameterSet::new();
        api_params.add_required("Authorization", format!("Bearer {}", api_key));

        Self {
            client: ReqwestClient::with_timeout(Duration::from_secs(15)),
            api_params,
            endpoint,
            retry_policy: RetryPolicy::aggressive(),
        }
    }
}

#[derive(Debug)]
pub struct SmartProxySource {
    client: Client,
    username: String,
    password: String,
}

#[async_trait::async_trait]
impl ProxySource for SmartProxySource {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyError> {
        // SmartProxy - Geographic targeting specialists
        // Excellent for country-specific requirements
        let endpoints = vec![
            ("us", "http://us.smartproxy.com:10000"),
            ("uk", "http://uk.smartproxy.com:10000"),
            ("de", "http://de.smartproxy.com:10000"),
            ("sg", "http://sg.smartproxy.com:10000"),
            ("jp", "http://jp.smartproxy.com:10000"),
        ];

        let mut proxies = Vec::new();
        for (country, endpoint) in endpoints {
            proxies.push(ProxyInfo {
                url: format!("{}:{}@{}", self.username, self.password, endpoint),
                region: self.country_to_region(country),
                protocol: ProxyProtocol::Http,
                speed_ms: None,
                success_rate: 0.95, // SmartProxy typically has high success rates
                last_checked: chrono::Utc::now(),
                source: "SmartProxy".to_string(),
            });
        }

        Ok(proxies)
    }

    fn source_name(&self) -> &str {
        "SmartProxy"
    }
    fn is_paid(&self) -> bool {
        true
    }
    fn max_requests_per_day(&self) -> Option<u32> {
        None
    }
}

impl SmartProxySource {
    fn country_to_region(&self, country: &str) -> Region {
        match country {
            "us" | "ca" => Region::NorthAmerica,
            "uk" | "de" | "fr" | "es" | "it" | "nl" => Region::Europe,
            "sg" | "jp" | "kr" | "au" | "hk" => Region::AsiaPacific,
            "cn" => Region::China,
            _ => Region::Global,
        }
    }
}

/// 3. ROTATING PROXY SERVICES
#[derive(Debug)]
pub struct ProxyRotatorSource {
    client: Client,
    api_endpoints: Vec<String>,
}

#[async_trait::async_trait]
impl ProxySource for ProxyRotatorSource {
    async fn fetch_proxies(&self) -> Result<Vec<ProxyInfo>, ProxyError> {
        // Services like ProxyScrape, ProxyList.geonode.com
        let mut all_proxies = Vec::new();

        for endpoint in &self.api_endpoints {
            match self.fetch_from_endpoint(endpoint).await {
                Ok(mut proxies) => all_proxies.append(&mut proxies),
                Err(e) => tracing::warn!("Failed to fetch from {}: {:?}", endpoint, e),
            }

            // Rate limiting between API calls
            sleep(Duration::from_millis(500)).await;
        }

        Ok(all_proxies)
    }

    fn source_name(&self) -> &str {
        "ProxyRotator"
    }
    fn is_paid(&self) -> bool {
        false
    }
    fn max_requests_per_day(&self) -> Option<u32> {
        Some(5000)
    }
}

impl ProxyRotatorSource {
    async fn fetch_from_endpoint(&self, endpoint: &str) -> Result<Vec<ProxyInfo>, ProxyError> {
        let response = self
            .client
            .get(endpoint)
            .send()
            .await
            .map_err(|e| ProxyError::NetworkError(e.to_string()))?;

        let proxy_list: String = response
            .text()
            .await
            .map_err(|e| ProxyError::ParseError(e.to_string()))?;

        Ok(proxy_list
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || !line.contains(':') {
                    return None;
                }

                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    Some(ProxyInfo {
                        url: format!("http://{}:{}", parts[0], parts[1]),
                        region: Region::Global,
                        protocol: ProxyProtocol::Http,
                        speed_ms: None,
                        success_rate: 0.6, // Rotating proxies have moderate success
                        last_checked: chrono::Utc::now(),
                        source: "ProxyRotator".to_string(),
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}

/// PROXY HEALTH MONITORING
pub struct ProxyHealthChecker {
    test_urls: Vec<String>,
    timeout: Duration,
    concurrent_checks: usize,
}

impl ProxyHealthChecker {
    pub fn new() -> Self {
        Self {
            test_urls: vec![
                "http://httpbin.org/ip".to_string(),
                "https://api.ipify.org".to_string(),
                "http://icanhazip.com".to_string(),
            ],
            timeout: Duration::from_secs(10),
            concurrent_checks: 50,
        }
    }

    /// Test proxy health and speed
    pub async fn check_proxy_health(&self, proxy: &ProxyInfo) -> ProxyHealthResult {
        let start = std::time::Instant::now();

        // Create client with proxy
        let proxy_url = &proxy.url;
        let client = match self.create_proxy_client(proxy_url).await {
            Ok(client) => client,
            Err(e) => {
                return ProxyHealthResult {
                    is_working: false,
                    response_time: Duration::from_secs(999),
                    error: Some(e.to_string()),
                };
            }
        };

        // Test with multiple URLs
        for test_url in &self.test_urls {
            match tokio::time::timeout(self.timeout, client.get(test_url).send()).await {
                Ok(Ok(response)) if response.status().is_success() => {
                    return ProxyHealthResult {
                        is_working: true,
                        response_time: start.elapsed(),
                        error: None,
                    };
                }
                Ok(Ok(_)) => continue,  // Try next URL
                Ok(Err(_)) => continue, // Try next URL
                Err(_) => continue,     // Timeout, try next URL
            }
        }

        ProxyHealthResult {
            is_working: false,
            response_time: start.elapsed(),
            error: Some("All test URLs failed".to_string()),
        }
    }

    async fn create_proxy_client(&self, proxy_url: &str) -> Result<Client, reqwest::Error> {
        let proxy = if proxy_url.starts_with("socks5://") {
            reqwest::Proxy::all(proxy_url)?
        } else {
            reqwest::Proxy::http(proxy_url)?
        };

        Client::builder().proxy(proxy).timeout(self.timeout).build()
    }
}

#[derive(Debug)]
pub struct ProxyHealthResult {
    pub is_working: bool,
    pub response_time: Duration,
    pub error: Option<String>,
}

/// ROTATION STRATEGIES
#[derive(Debug, Clone)]
pub enum RotationStrategy {
    RoundRobin,
    Random,
    HealthBased,      // Prefer faster, more reliable proxies
    GeographicRandom, // Random within the target region
    LoadBalanced,     // Distribute load evenly
}

/// COMPLETE PROXY CONFIGURATION
pub struct ProxyManagerConfig {
    // Free sources (for testing/light usage)
    pub enable_free_sources: bool,
    pub free_source_limits: HashMap<String, u32>,

    // Premium sources (for production)
    pub bright_data_config: Option<BrightDataConfig>,
    pub smart_proxy_config: Option<SmartProxyConfig>,
    pub rotating_proxy_apis: Vec<String>,

    // Health monitoring
    pub health_check_interval: Duration,
    pub min_success_rate: f32,
    pub max_response_time: Duration,

    // Rotation settings
    pub rotation_strategy: RotationStrategy,
    pub regional_balancing: bool,
}

#[derive(Debug)]
pub struct BrightDataConfig {
    pub api_key: String,
    pub endpoint: String,
    pub zones: Vec<String>, // Different geographic zones
}

#[derive(Debug)]
pub struct SmartProxyConfig {
    pub username: String,
    pub password: String,
    pub countries: Vec<String>,
}

/// COST-EFFECTIVE SOURCING RECOMMENDATIONS
impl ProxyManagerConfig {
    pub fn create_cost_effective_setup() -> ProxyManagerConfig {
        ProxyManagerConfig {
            // Start with free sources for development
            enable_free_sources: true,
            free_source_limits: [
                ("FreeProxyList".to_string(), 1000),
                ("ProxyRotator".to_string(), 5000),
            ]
            .into_iter()
            .collect(),

            // Add premium sources for production
            bright_data_config: None, // $500+/month but highest quality
            smart_proxy_config: None, // $50+/month good balance
            rotating_proxy_apis: vec![
                "https://api.proxyscrape.com/v2/".to_string(),
                "https://proxylist.geonode.com/api/".to_string(),
            ],

            health_check_interval: Duration::from_secs(300), // 5 minutes
            min_success_rate: 0.7,
            max_response_time: Duration::from_secs(15),

            rotation_strategy: RotationStrategy::GeographicRandom,
            regional_balancing: true,
        }
    }

    pub fn create_production_setup() -> ProxyManagerConfig {
        ProxyManagerConfig {
            enable_free_sources: false, // Disable for production reliability

            // Premium sources only
            bright_data_config: Some(BrightDataConfig {
                api_key: "your_bright_data_key".to_string(),
                endpoint: "https://brightdata.com/api/v1".to_string(),
                zones: vec!["residential_us".to_string(), "residential_eu".to_string()],
            }),

            smart_proxy_config: Some(SmartProxyConfig {
                username: "your_username".to_string(),
                password: "your_password".to_string(),
                countries: vec![
                    "US".to_string(),
                    "UK".to_string(),
                    "DE".to_string(),
                    "SG".to_string(),
                ],
            }),

            health_check_interval: Duration::from_secs(60), // 1 minute
            min_success_rate: 0.95,
            max_response_time: Duration::from_secs(5),

            rotation_strategy: RotationStrategy::HealthBased,
            regional_balancing: true,

            ..Default::default()
        }
    }
}

// Supporting types
#[derive(Deserialize)]
struct BrightDataResponse {
    proxies: Vec<BrightDataProxy>,
}

#[derive(Deserialize)]
struct BrightDataProxy {
    ip: String,
    port: u16,
    country: String,
}

impl BrightDataResponse {
    fn into_proxy_info(self) -> Vec<ProxyInfo> {
        self.proxies
            .into_iter()
            .map(|p| ProxyInfo {
                url: format!("http://{}:{}", p.ip, p.port),
                region: country_to_region(&p.country),
                protocol: ProxyProtocol::Http,
                speed_ms: None,
                success_rate: 0.9,
                last_checked: chrono::Utc::now(),
                source: "BrightData".to_string(),
            })
            .collect()
    }
}

fn country_to_region(country: &str) -> Region {
    match country.to_uppercase().as_str() {
        "US" | "CA" => Region::NorthAmerica,
        "UK" | "DE" | "FR" | "ES" | "IT" | "NL" => Region::Europe,
        "SG" | "JP" | "KR" | "AU" | "HK" => Region::AsiaPacific,
        "CN" => Region::China,
        _ => Region::Global,
    }
}

impl Default for ProxyManagerConfig {
    fn default() -> Self {
        Self::create_cost_effective_setup()
    }
}
