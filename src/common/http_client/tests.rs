#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

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
    fn test_retry_policy() {
        let policy = RetryPolicy::new();
        assert_eq!(policy.max_attempts(), 3);

        let delay1 = policy.calculate_delay(0);
        let delay2 = policy.calculate_delay(1);
        assert!(delay2 > delay1);
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
}
