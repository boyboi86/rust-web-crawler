//! HTTP client building blocks for web crawler
//! Level 3 implementation: ReqwestClient and ReqwestClientBuilder

use crate::config::crawler::defaults::{
    DEFAULT_APP_USER_AGENT, DEFAULT_HTTP_TIMEOUT_SECS, DEFAULT_TCP_KEEPALIVE_SECS,
};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// ReqwestClient builder - centralized HTTP client configuration with full builder pattern
/// Single source of truth for all HTTP client settings
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
    timeout: Duration,
    headers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ReqwestClientBuilder {
    timeout: Duration,
    headers: HashMap<String, String>,
    proxies: Vec<String>,
    tcp_keepalive: Duration,
    danger_accept_invalid_certs: bool,
}

impl ReqwestClientBuilder {
    pub fn new() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), DEFAULT_APP_USER_AGENT.to_string());

        Self {
            timeout: Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS),
            headers: default_headers,
            proxies: Vec::new(),
            tcp_keepalive: Duration::from_secs(DEFAULT_TCP_KEEPALIVE_SECS),
            danger_accept_invalid_certs: false,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    pub fn with_proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxies.push(proxy_url.into());
        self
    }

    pub fn with_proxies(mut self, proxy_urls: Vec<String>) -> Self {
        self.proxies.extend(proxy_urls);
        self
    }

    pub fn with_tcp_keepalive(mut self, keepalive: Duration) -> Self {
        self.tcp_keepalive = keepalive;
        self
    }

    pub fn danger_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.danger_accept_invalid_certs = accept;
        self
    }

    pub fn build(self) -> Result<ReqwestClient, Box<dyn std::error::Error>> {
        let mut client_builder = Client::builder()
            .timeout(self.timeout)
            .tcp_keepalive(self.tcp_keepalive)
            .http1_title_case_headers()
            .http1_allow_obsolete_multiline_headers_in_responses(true)
            .http1_allow_spaces_after_header_name_in_responses(true)
            .danger_accept_invalid_certs(self.danger_accept_invalid_certs);

        // Add all proxies to the client (Spider-RS pattern)
        for proxy_url in &self.proxies {
            if !proxy_url.is_empty() {
                match Proxy::all(proxy_url) {
                    Ok(proxy) => {
                        client_builder = client_builder.proxy(proxy);
                        println!("Added proxy to client: {}", proxy_url);
                    }
                    Err(e) => {
                        println!("Failed to create proxy {}: {}", proxy_url, e);
                    }
                }
            }
        }

        let client = client_builder.build()?;
        Ok(ReqwestClient {
            client,
            timeout: self.timeout,
            headers: self.headers,
        })
    }
}

impl Default for ReqwestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestClient {
    /// Create a new ReqwestClient with sensible defaults using builder pattern
    pub fn new() -> Self {
        match ReqwestClientBuilder::new().build() {
            Ok(client) => client,
            Err(_) => {
                // If we can't create a default client, create a minimal one
                // This should not happen in normal circumstances
                ReqwestClient {
                    client: reqwest::Client::new(),
                    timeout: Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS),
                    headers: HashMap::new(),
                }
            }
        }
    }

    /// Create a builder for custom configuration
    pub fn builder() -> ReqwestClientBuilder {
        ReqwestClientBuilder::new()
    }

    /// Create with custom timeout (convenience method)
    pub fn with_timeout(timeout: Duration) -> Self {
        match ReqwestClientBuilder::new().with_timeout(timeout).build() {
            Ok(client) => client,
            Err(_) => {
                // If we can't create a client with timeout, create a minimal one
                ReqwestClient {
                    client: reqwest::Client::new(),
                    timeout,
                    headers: HashMap::new(),
                }
            }
        }
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get all headers
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Check if a specific header exists
    pub fn has_header(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    /// Get a specific header value
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Add or update a header
    pub fn set_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(key.into(), value.into());
    }

    /// Remove a header
    pub fn remove_header(&mut self, key: &str) -> Option<String> {
        self.headers.remove(key)
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reqwest_client_builder() {
        let client = match ReqwestClient::builder()
            .with_timeout(Duration::from_secs(60))
            .with_header("Custom-Header", "test-value")
            .build()
        {
            Ok(client) => client,
            Err(e) => panic!("Failed to build client: {}", e),
        };

        assert_eq!(client.timeout(), Duration::from_secs(60));
        assert!(client.has_header("Custom-Header"));
        assert_eq!(
            client.header("Custom-Header"),
            Some(&"test-value".to_string())
        );
    }

    #[test]
    fn test_reqwest_client_default() {
        let client = ReqwestClient::new();
        assert_eq!(client.timeout(), Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS));
        assert!(client.has_header("User-Agent"));
    }

    #[test]
    fn test_reqwest_client_header_management() {
        let mut client = ReqwestClient::new();
        
        client.set_header("Test-Header", "test-value");
        assert!(client.has_header("Test-Header"));
        assert_eq!(client.header("Test-Header"), Some(&"test-value".to_string()));
        
        let removed = client.remove_header("Test-Header");
        assert_eq!(removed, Some("test-value".to_string()));
        assert!(!client.has_header("Test-Header"));
    }
}
