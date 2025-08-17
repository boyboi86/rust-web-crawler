/// HTTP client management - refactored using common building blocks
/// Following Rule 1: No hardcoding - all configuration external
/// Following Rule 3: Builder pattern for complex client configurations
/// Following Rule 4: Privacy first - controlled access through builders
/// Following Rule 8: Idiomatic Rust - Result<T,E>, functional patterns
use crate::common::{
    BooleanFlag, DelayDuration, LimitValue, NetworkResult, ProxyAuth, ProxyConfig, ProxyType,
    SingleProxyConfig, TaskError, TimeoutDuration, UrlString,
};
use reqwest::{Client, Proxy, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// HTTP client configuration using building blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    user_agent: String,
    timeout: TimeoutDuration,
    redirect_limit: LimitValue,
    connection_pool_size: LimitValue,
    idle_timeout: TimeoutDuration,
    compression_enabled: BooleanFlag,
    cookies_enabled: BooleanFlag,
    proxy_config: Option<ProxyConfig>,
    custom_headers: HashMap<String, String>,
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn timeout(&self) -> Duration {
        self.timeout.duration()
    }

    pub fn redirect_limit(&self) -> u32 {
        self.redirect_limit.value() as u32
    }

    pub fn connection_pool_size(&self) -> usize {
        self.connection_pool_size.value() as usize
    }

    pub fn idle_timeout(&self) -> Duration {
        self.idle_timeout.duration()
    }

    pub fn is_compression_enabled(&self) -> bool {
        self.compression_enabled.is_enabled()
    }

    pub fn is_cookies_enabled(&self) -> bool {
        self.cookies_enabled.is_enabled()
    }

    pub fn proxy_config(&self) -> Option<&ProxyConfig> {
        self.proxy_config.as_ref()
    }

    pub fn custom_headers(&self) -> &HashMap<String, String> {
        &self.custom_headers
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: "RustWebCrawler/1.0".to_string(),
            timeout: TimeoutDuration::from_secs(30),
            redirect_limit: LimitValue::new(10),
            connection_pool_size: LimitValue::new(10),
            idle_timeout: TimeoutDuration::from_secs(60),
            compression_enabled: BooleanFlag::enabled(),
            cookies_enabled: BooleanFlag::enabled(),
            proxy_config: None,
            custom_headers: HashMap::new(),
        }
    }
}

/// Builder for HTTP client configuration
#[derive(Debug)]
pub struct ClientConfigBuilder {
    user_agent: String,
    timeout: TimeoutDuration,
    redirect_limit: LimitValue,
    connection_pool_size: LimitValue,
    idle_timeout: TimeoutDuration,
    compression_enabled: BooleanFlag,
    cookies_enabled: BooleanFlag,
    proxy_config: Option<ProxyConfig>,
    custom_headers: HashMap<String, String>,
}

impl ClientConfigBuilder {
    pub fn new() -> Self {
        let default_config = ClientConfig::default();
        Self {
            user_agent: default_config.user_agent,
            timeout: default_config.timeout,
            redirect_limit: default_config.redirect_limit,
            connection_pool_size: default_config.connection_pool_size,
            idle_timeout: default_config.idle_timeout,
            compression_enabled: default_config.compression_enabled,
            cookies_enabled: default_config.cookies_enabled,
            proxy_config: default_config.proxy_config,
            custom_headers: default_config.custom_headers,
        }
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn with_timeout(mut self, timeout: TimeoutDuration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_redirect_limit(mut self, limit: LimitValue) -> Self {
        self.redirect_limit = limit;
        self
    }

    pub fn with_connection_pool_size(mut self, size: LimitValue) -> Self {
        self.connection_pool_size = size;
        self
    }

    pub fn with_idle_timeout(mut self, timeout: TimeoutDuration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    pub fn with_compression(mut self, enabled: BooleanFlag) -> Self {
        self.compression_enabled = enabled;
        self
    }

    pub fn with_cookies(mut self, enabled: BooleanFlag) -> Self {
        self.cookies_enabled = enabled;
        self
    }

    pub fn with_proxy(mut self, proxy_config: ProxyConfig) -> Self {
        self.proxy_config = Some(proxy_config);
        self
    }

    pub fn with_custom_header(mut self, name: String, value: String) -> Self {
        self.custom_headers.insert(name, value);
        self
    }

    pub fn with_custom_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.custom_headers.extend(headers);
        self
    }

    pub fn build(self) -> ClientConfig {
        ClientConfig {
            user_agent: self.user_agent,
            timeout: self.timeout,
            redirect_limit: self.redirect_limit,
            connection_pool_size: self.connection_pool_size,
            idle_timeout: self.idle_timeout,
            compression_enabled: self.compression_enabled,
            cookies_enabled: self.cookies_enabled,
            proxy_config: self.proxy_config,
            custom_headers: self.custom_headers,
        }
    }
}

impl Default for ClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP client factory using building blocks
/// Following Rule 4: Privacy first - all construction through builders
pub struct HttpClientFactory {
    // Private state - not exposed
    _marker: std::marker::PhantomData<()>,
}

impl HttpClientFactory {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Create HTTP client from configuration
    pub fn create_client(&self, config: &ClientConfig) -> NetworkResult<Client> {
        let mut client_builder = Client::builder()
            .user_agent(config.user_agent())
            .timeout(config.timeout())
            .redirect(reqwest::redirect::Policy::limited(
                config.redirect_limit() as usize
            ))
            .pool_max_idle_per_host(config.connection_pool_size())
            .pool_idle_timeout(Some(config.idle_timeout()));

        // Apply compression settings
        if config.is_compression_enabled() {
            client_builder = client_builder.gzip(true).deflate(true);
        }

        // Apply cookie settings
        if config.is_cookies_enabled() {
            // Note: Cookie store requires cookies feature to be enabled in Cargo.toml
            // For now, we'll skip cookie store configuration
            // client_builder = client_builder.cookie_store(true);
        }

        // Apply proxy if configured
        if let Some(proxy_config) = config.proxy_config() {
            let proxy = self.create_proxy(proxy_config)?;
            client_builder = client_builder.proxy(proxy);
        }

        // Build client
        client_builder
            .build()
            .map_err(|e| TaskError::network(format!("Failed to create HTTP client: {}", e)))
    }

    /// Create proxy from configuration
    fn create_proxy(&self, proxy_config: &ProxyConfig) -> NetworkResult<Proxy> {
        let proxy_url = proxy_config.url().as_str();

        let proxy = match proxy_config.proxy_type() {
            crate::common::ProxyType::Http => Proxy::http(proxy_url),
            crate::common::ProxyType::Https => Proxy::https(proxy_url),
            crate::common::ProxyType::Socks5 => Proxy::all(proxy_url),
        };

        let mut proxy =
            proxy.map_err(|e| TaskError::network(format!("Failed to create proxy: {}", e)))?;

        // Apply authentication if configured
        if let Some(auth) = proxy_config.auth() {
            proxy = proxy.basic_auth(auth.username(), auth.password());
        }

        Ok(proxy)
    }

    /// Create client with specific user agent
    pub fn create_with_user_agent(&self, user_agent: String) -> NetworkResult<Client> {
        let config = ClientConfig::builder().with_user_agent(user_agent).build();

        self.create_client(&config)
    }

    /// Create client with proxy
    pub fn create_with_proxy(&self, proxy_config: ProxyConfig) -> NetworkResult<Client> {
        let config = ClientConfig::builder().with_proxy(proxy_config).build();

        self.create_client(&config)
    }

    /// Create client optimized for fast requests
    pub fn create_fast_client(&self) -> NetworkResult<Client> {
        let config = ClientConfig::builder()
            .with_timeout(TimeoutDuration::from_secs(10))
            .with_connection_pool_size(LimitValue::new(20))
            .with_redirect_limit(LimitValue::new(5))
            .build();

        self.create_client(&config)
    }

    /// Create client optimized for reliable requests
    pub fn create_reliable_client(&self) -> NetworkResult<Client> {
        let config = ClientConfig::builder()
            .with_timeout(TimeoutDuration::from_secs(60))
            .with_redirect_limit(LimitValue::new(20))
            .with_connection_pool_size(LimitValue::new(5))
            .build();

        self.create_client(&config)
    }
}

impl Default for HttpClientFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Pooled HTTP client manager
/// Following Rule 4: Privacy first - managed access to client pool
pub struct ClientPool {
    // Private client storage
    clients: Arc<RwLock<HashMap<String, Client>>>,
    factory: HttpClientFactory,
    default_config: ClientConfig,
}

impl ClientPool {
    pub fn new(default_config: ClientConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            factory: HttpClientFactory::new(),
            default_config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ClientConfig::default())
    }

    /// Get or create a client with specific configuration
    pub async fn get_client(
        &self,
        key: &str,
        config: Option<&ClientConfig>,
    ) -> NetworkResult<Client> {
        // Check if client already exists
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(key) {
                return Ok(client.clone());
            }
        }

        // Create new client
        let client_config = config.unwrap_or(&self.default_config);
        let client = self.factory.create_client(client_config)?;

        // Store for future use
        {
            let mut clients = self.clients.write().await;
            clients.insert(key.to_string(), client.clone());
        }

        Ok(client)
    }

    /// Get default client
    pub async fn get_default_client(&self) -> NetworkResult<Client> {
        self.get_client("default", None).await
    }

    /// Get client with specific user agent
    pub async fn get_user_agent_client(&self, user_agent: &str) -> NetworkResult<Client> {
        let key = format!("ua_{}", user_agent);
        let config = ClientConfig::builder()
            .with_user_agent(user_agent.to_string())
            .build();

        self.get_client(&key, Some(&config)).await
    }

    /// Get client with proxy
    pub async fn get_proxy_client(&self, proxy_config: &ProxyConfig) -> NetworkResult<Client> {
        let key = format!("proxy_{}", proxy_config.url().as_str());
        let config = ClientConfig::builder()
            .with_proxy(proxy_config.clone())
            .build();

        self.get_client(&key, Some(&config)).await
    }

    /// Clear all cached clients
    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        clients.clear();
    }

    /// Get current client count
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Remove specific client from pool
    pub async fn remove_client(&self, key: &str) -> bool {
        let mut clients = self.clients.write().await;
        clients.remove(key).is_some()
    }
}

impl Default for ClientPool {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// HTTP response wrapper with enhanced functionality
#[derive(Debug)]
pub struct ResponseWrapper {
    response: Response,
    url: UrlString,
    request_duration: Duration,
}

impl ResponseWrapper {
    pub fn new(response: Response, url: UrlString, request_duration: Duration) -> Self {
        Self {
            response,
            url,
            request_duration,
        }
    }

    pub fn response(&self) -> &Response {
        &self.response
    }

    pub fn into_response(self) -> Response {
        self.response
    }

    pub fn url(&self) -> &UrlString {
        &self.url
    }

    pub fn request_duration(&self) -> Duration {
        self.request_duration
    }

    pub fn status(&self) -> reqwest::StatusCode {
        self.response.status()
    }

    pub fn is_success(&self) -> bool {
        self.response.status().is_success()
    }

    pub fn is_redirect(&self) -> bool {
        self.response.status().is_redirection()
    }

    pub fn is_client_error(&self) -> bool {
        self.response.status().is_client_error()
    }

    pub fn is_server_error(&self) -> bool {
        self.response.status().is_server_error()
    }

    pub fn content_length(&self) -> Option<u64> {
        self.response.content_length()
    }

    pub fn headers(&self) -> &reqwest::header::HeaderMap {
        self.response.headers()
    }

    /// Get response text with error handling
    pub async fn text(self) -> NetworkResult<String> {
        self.response
            .text()
            .await
            .map_err(|e| TaskError::network(format!("Failed to read response text: {}", e)))
    }

    /// Get response bytes with error handling
    pub async fn bytes(self) -> NetworkResult<bytes::Bytes> {
        self.response
            .bytes()
            .await
            .map_err(|e| TaskError::network(format!("Failed to read response bytes: {}", e)))
    }

    /// Get response JSON with error handling
    pub async fn json<T: for<'de> serde::Deserialize<'de>>(self) -> NetworkResult<T> {
        self.response
            .json()
            .await
            .map_err(|e| TaskError::network(format!("Failed to parse response JSON: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_config_builder() {
        let config = ClientConfig::builder()
            .with_user_agent("TestAgent/1.0".to_string())
            .with_timeout(TimeoutDuration::from_secs(15))
            .with_redirect_limit(LimitValue::new(5))
            .build();

        assert_eq!(config.user_agent(), "TestAgent/1.0");
        assert_eq!(config.timeout(), Duration::from_secs(15));
        assert_eq!(config.redirect_limit(), 5);
    }

    #[tokio::test]
    async fn test_client_factory() {
        let factory = HttpClientFactory::new();
        let config = ClientConfig::default();

        let client = factory
            .create_client(&config)
            .expect("Failed to create client");
        assert!(client.clone().into());
    }

    #[tokio::test]
    async fn test_client_pool() {
        let pool = ClientPool::with_defaults();

        let client1 = pool
            .get_default_client()
            .await
            .expect("Failed to get client");
        let client2 = pool
            .get_default_client()
            .await
            .expect("Failed to get client");

        // Should return the same cached client
        assert_eq!(pool.client_count().await, 1);
    }

    #[tokio::test]
    async fn test_fast_client_configuration() {
        let factory = HttpClientFactory::new();
        let client = factory
            .create_fast_client()
            .expect("Failed to create fast client");
        assert!(client.clone().into());
    }

    #[tokio::test]
    async fn test_reliable_client_configuration() {
        let factory = HttpClientFactory::new();
        let client = factory
            .create_reliable_client()
            .expect("Failed to create reliable client");
        assert!(client.clone().into());
    }

    #[tokio::test]
    async fn test_proxy_configuration() {
        let proxy_config = ProxyConfig::builder()
            .with_url(UrlString::new("http://proxy.example.com:8080"))
            .with_proxy_type(crate::common::ProxyType::Http)
            .build();

        let config = ClientConfig::builder().with_proxy(proxy_config).build();

        assert!(config.proxy_config().is_some());
    }
}
