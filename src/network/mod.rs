// Network-related functionality

pub mod client;
pub mod dns;
pub mod rate_limit;
pub mod robots;

// Re-export common networking components
pub use client::{ClientManager, HttpClientFactory};
pub use dns::DnsCache;
pub use rate_limit::{DomainRequestTracker, GlobalRateLimiter};
pub use robots::{RobotsCache, RobotsHandler};
