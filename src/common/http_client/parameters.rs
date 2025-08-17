//! API parameter handling building blocks
//! Level 3 implementation: BasicApiParam and ApiParameterSet

use serde::{Deserialize, Serialize};

/// BasicApiParam - standard parameter structure for API calls with privacy-first design
/// Building block for consistent API parameter handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicApiParam {
    key: String,    // Private fields - access through methods only
    value: String,  // Private fields - access through methods only
    required: bool, // Private fields - access through methods only
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

    // Controlled access methods (privacy-first)
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

/// ApiParameterSet - collection of API parameters with validation
/// Building block for managing sets of API parameters
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

    pub fn to_form_data(&self) -> Result<Vec<(String, String)>, String> {
        self.to_query_params()
    }

    pub fn get_param(&self, key: &str) -> Option<&BasicApiParam> {
        self.parameters.iter().find(|p| p.key() == key)
    }

    pub fn get_param_mut(&mut self, key: &str) -> Option<&mut BasicApiParam> {
        self.parameters.iter_mut().find(|p| p.key() == key)
    }

    pub fn remove_param(&mut self, key: &str) -> Option<BasicApiParam> {
        if let Some(pos) = self.parameters.iter().position(|p| p.key() == key) {
            Some(self.parameters.remove(pos))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.parameters.clear();
    }

    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &BasicApiParam> {
        self.parameters.iter()
    }
}

impl Default for ApiParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_api_param() {
        let param = BasicApiParam::new("key1", "value1");
        assert!(param.is_valid());

        let empty_param = BasicApiParam::new("key2", "");
        assert!(!empty_param.is_valid());

        let optional_empty = BasicApiParam::optional("key3", "");
        assert!(optional_empty.is_valid());
    }

    #[test]
    fn test_basic_api_param_privacy() {
        let param = BasicApiParam::new("secret_key", "secret_value");

        // Test controlled access
        assert_eq!(param.key(), "secret_key");
        assert_eq!(param.value(), "secret_value");
        assert!(param.is_required());

        let mut param = BasicApiParam::optional("optional_key", "");
        assert!(!param.is_required());

        param.set_value("new_value");
        assert_eq!(param.value(), "new_value");
    }

    #[test]
    fn test_api_parameter_set() {
        let mut params = ApiParameterSet::new();
        params.add_required("api_key", "test123");
        params.add_optional("format", "json");

        assert!(params.validate().is_ok());

        let query_params = match params.to_query_params() {
            Ok(params) => params,
            Err(e) => panic!("Failed to get query params: {}", e),
        };
        assert_eq!(query_params.len(), 2);
    }

    #[test]
    fn test_api_parameter_set_operations() {
        let mut params = ApiParameterSet::new();
        
        // Test adding and retrieving parameters
        params.add_required("key1", "value1");
        params.add_optional("key2", "value2");
        
        assert_eq!(params.len(), 2);
        assert!(!params.is_empty());
        
        let param = params.get_param("key1").unwrap();
        assert_eq!(param.key(), "key1");
        assert_eq!(param.value(), "value1");
        
        // Test removing parameters
        let removed = params.remove_param("key1");
        assert!(removed.is_some());
        assert_eq!(params.len(), 1);
        
        // Test clearing
        params.clear();
        assert!(params.is_empty());
    }

    #[test]
    fn test_api_parameter_set_validation() {
        let mut params = ApiParameterSet::new();
        params.add_required("required_param", "");
        params.add_optional("optional_param", "");
        
        // Should fail validation due to empty required parameter
        assert!(params.validate().is_err());
        
        // Fix the required parameter
        if let Some(param) = params.get_param_mut("required_param") {
            param.set_value("valid_value");
        }
        
        // Should now pass validation
        assert!(params.validate().is_ok());
    }
}
