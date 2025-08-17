// HTTP client management and configuration
use anyhow::Error;
use reqwest::{Client, Proxy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::config::defaults;

/// HTTP client factory with common configuration and proxy support
pub struct HttpClientFactory;

impl HttpClientFactory {
    /// Create a standard HTTP client with default settings
    pub fn create_default_client(user_agent: &str) -> Result<Client, Error> {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::limited(defaults::MAX_REDIRECTS))
            .user_agent(user_agent)
            .timeout(Duration::from_secs(defaults::REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(defaults::CONNECTION_POOL_SIZE)
            .pool_idle_timeout(Duration::from_secs(defaults::CONNECTION_IDLE_TIMEOUT_SECS))
            .build()?;
        Ok(client)
    }

    /// Create an HTTP client with proxy support
    pub fn create_proxy_client(proxy_url: &str, user_agent: &str) -> Result<Client, Error> {
        let proxy = if proxy_url.starts_with("socks5://") {
            Proxy::all(proxy_url)?
        } else {
            Proxy::http(proxy_url)?
        };

        let client = Client::builder()
            .proxy(proxy)
            .redirect(reqwest::redirect::Policy::limited(defaults::MAX_REDIRECTS))
            .user_agent(user_agent)
            .timeout(Duration::from_secs(defaults::REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(defaults::CONNECTION_POOL_SIZE)
            .pool_idle_timeout(Duration::from_secs(defaults::CONNECTION_IDLE_TIMEOUT_SECS))
            .build()?;
        Ok(client)
    }

    /// Create multiple clients with different proxy configurations
    pub fn create_proxy_pool(
        proxy_urls: &[String],
        user_agent: &str,
    ) -> Result<Vec<Client>, Error> {
        let mut clients = Vec::new();

        // Always include a default client (no proxy)
        clients.push(Self::create_default_client(user_agent)?);

        // Add proxy clients
        for proxy_url in proxy_urls {
            match Self::create_proxy_client(proxy_url, user_agent) {
                Ok(client) => clients.push(client),
                Err(e) => {
                    tracing::warn!("Failed to create proxy client for {}: {}", proxy_url, e);
                }
            }
        }

        Ok(clients)
    }
}

/// Manages a pool of HTTP clients with different configurations
pub struct ClientManager {
    clients: Vec<Client>,
    proxy_clients: Arc<Mutex<HashMap<String, Client>>>,
    current_index: Arc<Mutex<usize>>,
}

impl ClientManager {
    /// Create a new client manager with a pool of clients
    pub fn new(proxy_urls: Vec<String>, user_agent: &str) -> Result<Self, Error> {
        let clients = HttpClientFactory::create_proxy_pool(&proxy_urls, user_agent)?;

        Ok(Self {
            clients,
            proxy_clients: Arc::new(Mutex::new(HashMap::new())),
            current_index: Arc::new(Mutex::new(0)),
        })
    }

    /// Get the next client from the pool (round-robin)
    pub async fn get_client(&self) -> Client {
        if self.clients.is_empty() {
            // Fallback to default client if pool is empty
            return HttpClientFactory::create_default_client(
                crate::config::defaults::DEFAULT_APP_USER_AGENT,
            )
            .unwrap_or_else(|_| Client::new());
        }

        let mut index = self.current_index.lock().await;
        let client = self.clients[*index].clone();
        *index = (*index + 1) % self.clients.len();
        client
    }

    /// Get a specific client for a proxy URL
    pub async fn get_proxy_client(&self, proxy_url: &str) -> Option<Client> {
        let clients = self.proxy_clients.lock().await;
        clients.get(proxy_url).cloned()
    }

    /// Add a new proxy client to the pool
    pub async fn add_proxy_client(&self, proxy_url: String, user_agent: &str) -> Result<(), Error> {
        let client = HttpClientFactory::create_proxy_client(&proxy_url, user_agent)?;
        let mut clients = self.proxy_clients.lock().await;
        clients.insert(proxy_url, client);
        Ok(())
    }

    /// Get the number of available clients
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new(vec![], crate::config::defaults::DEFAULT_APP_USER_AGENT).unwrap_or_else(|_| {
            Self {
                clients: vec![Client::new()],
                proxy_clients: Arc::new(Mutex::new(HashMap::new())),
                current_index: Arc::new(Mutex::new(0)),
            }
        })
    }
}
