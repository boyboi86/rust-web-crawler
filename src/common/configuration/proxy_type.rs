use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Http
    }
}
