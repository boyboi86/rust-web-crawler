// Network-related functionality

pub mod client;
pub mod client_refactored;
pub mod dns;
pub mod proxy;
pub mod rate_limit;
pub mod robots;

// Re-export common networking components
pub use client::{ClientManager, HttpClientFactory};
pub use client_refactored::{
    ClientConfig, ClientConfigBuilder, ClientPool,
    HttpClientFactory as HttpClientFactoryRefactored, ResponseWrapper,
};
pub use dns::DnsCache;
pub use proxy::{
    GeoProxySelector, ProxyConfig, ProxyInfo, ProxyRegionsConfig, ProxySource, ProxySourceManager,
};
pub use rate_limit::{DomainRequestTracker, GlobalRateLimiter};
pub use robots::{RobotsCache, RobotsHandler};
