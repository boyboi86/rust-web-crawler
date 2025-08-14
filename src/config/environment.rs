use crate::config::WebCrawlerConfig;
use crate::core::DomainRateLimit;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub crawler: WebCrawlerConfig,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub persist_queue: bool,
    pub queue_file_path: String,
    pub results_output_path: String,
    pub checkpoint_interval_secs: u64,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            crawler: WebCrawlerConfig::default(),
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("crawler.log".to_string()),
                max_file_size_mb: 100,
                max_files: 5,
            },
            monitoring: MonitoringConfig {
                enable_metrics: false,
                metrics_port: 9090,
                health_check_interval_secs: 30,
            },
            storage: StorageConfig {
                persist_queue: true,
                queue_file_path: "queue_state.json".to_string(),
                results_output_path: "crawl_results".to_string(),
                checkpoint_interval_secs: 60,
            },
        }
    }
}

impl EnvironmentConfig {
    /// Load configuration from file with environment variable overrides
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let mut config: EnvironmentConfig = toml::from_str(&config_content)?;

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(env) = std::env::var("CRAWLER_ENV") {
            self.environment = match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "staging" | "stage" => Environment::Staging,
                "production" | "prod" => Environment::Production,
                _ => self.environment.clone(),
            };
        }

        if let Ok(level) = std::env::var("LOG_LEVEL") {
            self.logging.level = level;
        }

        if let Ok(port) = std::env::var("METRICS_PORT")
            && let Ok(port) = port.parse()
        {
            self.monitoring.metrics_port = port;
        }
    }

    /// Get configuration optimized for the current environment
    pub fn optimize_for_environment(&mut self) {
        match self.environment {
            Environment::Development => {
                self.crawler.default_rate_limit = Some(DomainRateLimit {
                    max_requests_per_second: 1, // Slower for dev
                    window_size_ms: 1000,
                });
                self.logging.level = "debug".to_string();
                self.monitoring.enable_metrics = true;
            }
            Environment::Staging => {
                self.crawler.default_rate_limit = Some(DomainRateLimit {
                    max_requests_per_second: 5,
                    window_size_ms: 1000,
                });
                self.logging.level = "info".to_string();
                self.monitoring.enable_metrics = true;
            }
            Environment::Production => {
                self.crawler.default_rate_limit = Some(DomainRateLimit {
                    max_requests_per_second: 10,
                    window_size_ms: 1000,
                });
                self.logging.level = "warn".to_string();
                self.monitoring.enable_metrics = true;
                self.storage.persist_queue = true; // Always persist in prod
            }
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.monitoring.metrics_port < 1024 {
            return Err(anyhow::anyhow!("Metrics port must be >= 1024"));
        }

        if self.logging.max_file_size_mb == 0 {
            return Err(anyhow::anyhow!("Log file size must be > 0"));
        }

        if self.storage.checkpoint_interval_secs == 0 {
            return Err(anyhow::anyhow!("Checkpoint interval must be > 0"));
        }

        Ok(())
    }

    /// Save current configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let config_content = toml::to_string_pretty(self)?;
        fs::write(path, config_content)?;
        Ok(())
    }
}
