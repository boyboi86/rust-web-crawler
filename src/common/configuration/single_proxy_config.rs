use crate::common::configuration::{ProxyAuth, ProxyType};
use crate::common::primitives::BooleanFlag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleProxyConfig {
    url: crate::common::UrlString,
    proxy_type: ProxyType,
    auth: Option<ProxyAuth>,
    enabled: BooleanFlag,
}

impl SingleProxyConfig {
    pub fn builder() -> SingleProxyConfigBuilder {
        SingleProxyConfigBuilder::new()
    }
    pub fn url(&self) -> &crate::common::UrlString {
        &self.url
    }
    pub fn proxy_type(&self) -> ProxyType {
        self.proxy_type
    }
    pub fn auth(&self) -> Option<&ProxyAuth> {
        self.auth.as_ref()
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.is_enabled()
    }
}

impl Default for SingleProxyConfig {
    fn default() -> Self {
        Self {
            url: crate::common::UrlString::new("http://localhost:8080"),
            proxy_type: ProxyType::Http,
            auth: None,
            enabled: BooleanFlag::disabled(),
        }
    }
}

#[derive(Debug)]
pub struct SingleProxyConfigBuilder {
    url: crate::common::UrlString,
    proxy_type: ProxyType,
    auth: Option<ProxyAuth>,
    enabled: BooleanFlag,
}

impl SingleProxyConfigBuilder {
    pub fn new() -> Self {
        let default_config = SingleProxyConfig::default();
        Self {
            url: default_config.url,
            proxy_type: default_config.proxy_type,
            auth: default_config.auth,
            enabled: default_config.enabled,
        }
    }
    pub fn with_url(mut self, url: crate::common::UrlString) -> Self {
        self.url = url;
        self
    }
    pub fn with_proxy_type(mut self, proxy_type: ProxyType) -> Self {
        self.proxy_type = proxy_type;
        self
    }
    pub fn with_auth(mut self, auth: ProxyAuth) -> Self {
        self.auth = Some(auth);
        self
    }
    pub fn with_enabled(mut self, enabled: BooleanFlag) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn build(self) -> SingleProxyConfig {
        SingleProxyConfig {
            url: self.url,
            proxy_type: self.proxy_type,
            auth: self.auth,
            enabled: self.enabled,
        }
    }
}

impl Default for SingleProxyConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
