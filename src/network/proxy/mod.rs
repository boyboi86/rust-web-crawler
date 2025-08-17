// Proxy management module
// Handles geographic proxy selection and proxy source management

pub mod geo_selector;
pub mod provider;
pub mod rotation;
pub mod source_manager;

// Re-export main components for convenience
pub use geo_selector::{GeoProxySelector, ProxyConfig, ProxyRegionsConfig};
pub use provider::{ProxyProvider, ProxyProviderConfig, ProxyProviderError};
pub use rotation::*;
pub use source_manager::*;
pub use source_manager::{ProxyError, ProxyInfo, ProxyProtocol, ProxySource, ProxySourceManager};
