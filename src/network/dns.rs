use anyhow::Error;
use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::config::defaults;
use crate::core::DnsResolver;

/// DNS resolution implementation with caching
pub struct DnsCache {
    cache: Arc<Mutex<HashMap<String, (String, Instant)>>>,
}

impl Default for DnsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_cache(&self) -> Arc<Mutex<HashMap<String, (String, Instant)>>> {
        self.cache.clone()
    }
}

impl DnsResolver for DnsCache {
    /// Resolve hostname to IP address with caching
    async fn resolve_hostname(&self, hostname: &str) -> Result<IpAddr, Error> {
        // Check cache first with TTL validation
        {
            let dns_cache = self.cache.lock().await;
            if let Some((ip_str, cached_at)) = dns_cache.get(hostname) {
                let ttl = Duration::from_secs(defaults::DNS_CACHE_TTL_SECS);
                if cached_at.elapsed() < ttl {
                    return Ok(IpAddr::from_str(ip_str)?);
                }
            }
        }

        // Cache miss or expired, perform DNS resolution
        let hostname_clone = hostname.to_string();
        let resolved_ip = tokio::task::spawn_blocking(move || {
            let socket_addr = format!("{}:80", hostname_clone);
            socket_addr
                .to_socket_addrs()
                .map_err(|e| anyhow::anyhow!("DNS resolution failed: {}", e))?
                .next()
                .map(|addr| addr.ip())
                .ok_or_else(|| {
                    anyhow::anyhow!("No IP address found for domain: {}", hostname_clone)
                })
        })
        .await??;

        // Update cache with the resolved IP
        {
            let mut dns_cache = self.cache.lock().await;
            dns_cache.insert(
                hostname.to_string(),
                (resolved_ip.to_string(), Instant::now()),
            );
        }

        Ok(resolved_ip)
    }

    /// Resolve domain to IP address with caching
    async fn resolve_domain(&self, domain: &str) -> Result<String, Error> {
        // Check cache first
        {
            let dns_cache = self.cache.lock().await;
            if let Some((ip, cached_at)) = dns_cache.get(domain) {
                let ttl = Duration::from_secs(defaults::DNS_CACHE_TTL_SECS);
                if cached_at.elapsed() < ttl {
                    return Ok(ip.clone());
                }
            }
        }

        // Cache miss or expired, perform DNS resolution
        let domain_clone = domain.to_string();
        let resolved = tokio::task::spawn_blocking(move || {
            let socket_addr = format!("{}:80", domain_clone);
            socket_addr
                .to_socket_addrs()
                .map_err(|e| anyhow::anyhow!("DNS resolution failed: {}", e))?
                .next()
                .ok_or_else(|| anyhow::anyhow!("No IP address found for domain: {}", domain_clone))
        })
        .await??;

        let ip = resolved.ip().to_string();

        // Update cache
        {
            let mut dns_cache = self.cache.lock().await;
            dns_cache.insert(domain.to_string(), (ip.clone(), Instant::now()));
        }

        Ok(ip)
    }

    /// Clean up expired DNS cache entries
    async fn cleanup_dns_cache(&self) {
        let mut dns_cache = self.cache.lock().await;
        let ttl = Duration::from_secs(defaults::DNS_CACHE_TTL_SECS);
        dns_cache.retain(|_, (_, cached_at)| cached_at.elapsed() < ttl);
    }

    /// Get diagnostic information about DNS cache
    async fn get_dns_cache_stats(&self) -> HashMap<String, String> {
        let dns_cache = self.cache.lock().await;
        let mut stats = HashMap::new();

        for (domain, (ip, cached_at)) in dns_cache.iter() {
            let age_secs = cached_at.elapsed().as_secs();
            stats.insert(domain.clone(), format!("{} (cached {}s ago)", ip, age_secs));
        }

        stats
    }
}
