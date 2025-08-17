use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    username: String,
    password: String,
}

impl ProxyAuth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn password(&self) -> &str {
        &self.password
    }
}
