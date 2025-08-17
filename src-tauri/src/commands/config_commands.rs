// =============================================================================
// CONFIGURATION API COMMANDS
// =============================================================================
// Tauri commands to expose configuration to the frontend, ensuring the
// frontend always gets its configuration from the backend single source of truth.

use crate::config::AppConfig;
use crate::config::defaults::{ConfigPreset, dev_config, test_config, yahoo_config};
use serde::{Deserialize, Serialize};
use tauri::command;

// =============================================================================
// FRONTEND CONFIGURATION RESPONSE
// =============================================================================

/// Configuration data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub app_config: AppConfig,
    pub presets: FrontendPresets,
    pub environment_info: EnvironmentInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendPresets {
    pub yahoo: AppConfig,
    pub test: AppConfig,
    pub development: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub is_debug: bool,
    pub version: String,
    pub platform: String,
    pub config_source: String,
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Get the complete application configuration for the frontend
#[command]
pub async fn get_app_config() -> Result<FrontendConfig, String> {
    let app_config =
        AppConfig::load().map_err(|e| format!("Failed to load configuration: {}", e))?;

    let frontend_config = FrontendConfig {
        app_config: app_config.clone(),
        presets: FrontendPresets {
            yahoo: yahoo_config(),
            test: test_config(),
            development: dev_config(),
        },
        environment_info: EnvironmentInfo {
            is_debug: app_config.development.debug_mode,
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            config_source: determine_config_source(),
        },
    };

    Ok(frontend_config)
}

/// Get a specific configuration preset
#[command]
pub async fn get_config_preset(preset_name: String) -> Result<AppConfig, String> {
    let preset = match preset_name.to_lowercase().as_str() {
        "development" | "dev" => ConfigPreset::Development,
        "production" | "prod" => ConfigPreset::Production,
        "testing" | "test" => ConfigPreset::Testing,
        "debug" => ConfigPreset::Debug,
        "yahoo" => return Ok(yahoo_config()),
        _ => return Err(format!("Unknown preset: {}", preset_name)),
    };

    Ok(AppConfig::for_preset(preset))
}

/// Validate a configuration object sent from the frontend
#[command]
pub async fn validate_config(config: AppConfig) -> Result<bool, String> {
    config.validate().map(|_| true).map_err(|e| e.to_string())
}

/// Get the current configuration summary as a human-readable string
#[command]
pub async fn get_config_summary() -> Result<String, String> {
    let config = AppConfig::load().map_err(|e| format!("Failed to load configuration: {}", e))?;

    Ok(config.summary())
}

/// Get available environment variables documentation
#[command]
pub async fn get_env_documentation() -> Result<String, String> {
    Ok(r#"
Environment Variables for Configuration Override:

Network Configuration:
- CRAWLER_USER_AGENT: Custom user agent string
- CRAWLER_REQUEST_TIMEOUT_SECS: Request timeout (1-300 seconds)
- CRAWLER_MAX_REDIRECTS: Maximum redirects (0-20)
- CRAWLER_POLITENESS_DELAY_MS: Delay between requests (milliseconds)

Crawling Configuration:
- CRAWLER_MAX_TOTAL_URLS: Maximum URLs to crawl (1-10000)
- CRAWLER_MAX_CRAWL_DEPTH: Maximum crawl depth (1-10)
- CRAWLER_MAX_CONCURRENT_REQUESTS: Max concurrent requests (1-50)

Rate Limiting:
- CRAWLER_RATE_LIMIT_RPS: Requests per second (1-100)

Proxy Configuration:
- CRAWLER_PROXY_POOL: Comma-separated list of proxy URLs

Development:
- CRAWLER_DEBUG_MODE: Enable debug mode (true/false)

Example:
set CRAWLER_USER_AGENT="Custom Bot 1.0"
set CRAWLER_MAX_TOTAL_URLS=500
set CRAWLER_DEBUG_MODE=true
"#
    .to_string())
}

/// Reset configuration to defaults (useful for development)
#[command]
pub async fn reset_config_to_defaults() -> Result<AppConfig, String> {
    // For security, only allow this in debug mode
    let current_config =
        AppConfig::load().map_err(|e| format!("Failed to load current configuration: {}", e))?;

    if !current_config.development.debug_mode {
        return Err("Reset to defaults is only allowed in debug mode".to_string());
    }

    Ok(AppConfig::default())
}

// =============================================================================
// CONFIGURATION UTILITIES
// =============================================================================

/// Determine where the configuration is being loaded from
fn determine_config_source() -> String {
    let mut sources = Vec::new();

    // Check for common environment variables
    if std::env::var("CRAWLER_USER_AGENT").is_ok() {
        sources.push("Environment Variables");
    }

    // Check for config file (if we implement file-based config later)
    if std::path::Path::new("crawler_config.toml").exists() {
        sources.push("Config File");
    }

    if sources.is_empty() {
        "Built-in Defaults".to_string()
    } else {
        format!("Defaults + {}", sources.join(" + "))
    }
}

// =============================================================================
// CONFIGURATION MIDDLEWARE
// =============================================================================

/// Middleware to ensure all crawl requests use proper configuration
pub fn ensure_config_loaded() -> Result<AppConfig, String> {
    AppConfig::load().map_err(|e| format!("Configuration error: {}", e))
}

/// Apply user overrides to base configuration
pub fn apply_user_overrides(
    mut base_config: AppConfig,
    user_agent: Option<String>,
    max_urls: Option<u32>,
) -> AppConfig {
    if let Some(ua) = user_agent {
        if !ua.trim().is_empty() {
            base_config.network.user_agent = ua;
        }
    }

    if let Some(max) = max_urls {
        if max > 0 && max <= 10000 {
            base_config.crawling.max_total_urls = max;
        }
    }

    base_config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_app_config() {
        let result = get_app_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(!config.app_config.network.user_agent.is_empty());
        assert!(config.app_config.crawling.max_total_urls > 0);
    }

    #[tokio::test]
    async fn test_get_config_preset() {
        let result = get_config_preset("development".to_string()).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.development.debug_mode);
    }

    #[tokio::test]
    async fn test_validate_config() {
        let valid_config = AppConfig::default();
        let result = validate_config(valid_config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
