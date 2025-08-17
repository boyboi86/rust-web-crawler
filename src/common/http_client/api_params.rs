// Level 3: BasicApiParam and ApiParameterSet
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicApiParam {
    key: String,
    value: String,
    required: bool,
}

impl BasicApiParam {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            required: true,
        }
    }
    pub fn optional(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            required: false,
        }
    }
    pub fn is_valid(&self) -> bool {
        !self.required || !self.value.is_empty()
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn is_required(&self) -> bool {
        self.required
    }
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
    pub fn make_optional(&mut self) {
        self.required = false;
    }
    pub fn make_required(&mut self) {
        self.required = true;
    }
}

#[derive(Debug, Clone)]
pub struct ApiParameterSet {
    parameters: Vec<BasicApiParam>,
}

impl ApiParameterSet {
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }
    pub fn add_param(&mut self, param: BasicApiParam) {
        self.parameters.push(param);
    }
    pub fn add_required(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.add_param(BasicApiParam::new(key, value));
    }
    pub fn add_optional(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.add_param(BasicApiParam::optional(key, value));
    }
    pub fn validate(&self) -> Result<(), String> {
        for param in &self.parameters {
            if !param.is_valid() {
                return Err(format!("Required parameter '{}' is empty", param.key()));
            }
        }
        Ok(())
    }
    pub fn to_query_params(&self) -> Result<Vec<(String, String)>, String> {
        self.validate()?;
        Ok(self
            .parameters
            .iter()
            .filter(|p| !p.value().is_empty())
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect())
    }
    pub fn to_headers(&self) -> Result<HashMap<String, String>, String> {
        self.validate()?;
        Ok(self
            .parameters
            .iter()
            .filter(|p| !p.value().is_empty())
            .map(|p| (p.key().to_string(), p.value().to_string()))
            .collect())
    }
}

impl Default for ApiParameterSet {
    fn default() -> Self {
        Self::new()
    }
}
