// Level 3: ReqwestClient and ReqwestClientBuilder
use crate::config::crawler::defaults::{
    DEFAULT_APP_USER_AGENT, DEFAULT_HTTP_TIMEOUT_SECS, DEFAULT_TCP_KEEPALIVE_SECS,
};
use reqwest::{Client, Proxy};
use std::collections::HashMap;
use std::time::Duration;

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
        for proxy_url in &self.proxies {
            if !proxy_url.is_empty() {
                match Proxy::all(proxy_url) {
                    Ok(proxy) => {
                        client_builder = client_builder.proxy(proxy);
                    }
                    Err(_) => {}
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
    pub fn new() -> Self {
        match ReqwestClientBuilder::new().build() {
            Ok(client) => client,
            Err(_) => ReqwestClient {
                client: reqwest::Client::new(),
                timeout: Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS),
                headers: HashMap::new(),
            },
        }
    }
    pub fn builder() -> ReqwestClientBuilder {
        ReqwestClientBuilder::new()
    }
    pub fn with_timeout(timeout: Duration) -> Self {
        match ReqwestClientBuilder::new().with_timeout(timeout).build() {
            Ok(client) => client,
            Err(_) => ReqwestClient {
                client: reqwest::Client::new(),
                timeout,
                headers: HashMap::new(),
            },
        }
    }
    pub fn with_proxies(
        timeout: Duration,
        proxy_urls: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        ReqwestClientBuilder::new()
            .with_timeout(timeout)
            .with_proxies(proxy_urls)
            .build()
    }
    pub fn client(&self) -> &Client {
        &self.client
    }
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
    pub fn has_header(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}
