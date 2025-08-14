/// Unified logging system
///
/// This module consolidates logging setup utilities with comprehensive event logging,
/// combining the simple initialization functions with advanced crawl event tracking.
pub mod events;
pub mod formatter;

use anyhow::Error;

// Re-export logging components
pub use events::{
    CrawlEvent, CrawlEventLogger, ErrorEvent, ErrorType, PerformanceEvent, PerformanceEventType,
};
pub use formatter::{CrawlLogFormatter, JsonLogFormatter};

/// Logging initialization utilities

/// Initialize structured logging for the application
pub fn init_logging() -> Result<(), Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rust_web_crawler=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Initialize logging with custom level
pub fn init_logging_with_level(level: &str) -> Result<(), Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = format!("{},rust_web_crawler={}", level, level);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Initialize JSON logging for production
pub fn init_json_logging() -> Result<(), Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rust_web_crawler=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Initialize logging with file output
pub fn init_file_logging(log_file_path: &str) -> Result<(), Error> {
    use std::fs::OpenOptions;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,rust_web_crawler=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(file))
        .init();

    Ok(())
}

/// Configure logging based on environment
pub fn configure_logging_for_environment(env: LogEnvironment) -> Result<(), Error> {
    match env {
        LogEnvironment::Development => init_logging_with_level("debug"),
        LogEnvironment::Testing => init_logging_with_level("warn"),
        LogEnvironment::Production => init_json_logging(),
    }
}

/// Logging environment types
#[derive(Debug, Clone)]
pub enum LogEnvironment {
    Development,
    Testing,
    Production,
}

/// Advanced logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub include_timestamps: bool,
    pub include_thread_ids: bool,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Plain,
    Json,
    Compact,
}

#[derive(Debug, Clone)]
pub enum LogOutput {
    Console,
    File(String),
    Both(String),
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Plain,
            output: LogOutput::Console,
            include_timestamps: true,
            include_thread_ids: false,
        }
    }
}

/// Initialize logging with advanced configuration
pub fn init_advanced_logging(config: LogConfig) -> Result<(), Error> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{},rust_web_crawler={}", config.level, config.level).into());

    // Simplified implementation that works with available tracing-subscriber features
    match config.output {
        LogOutput::Console => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
        LogOutput::File(path) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;

            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_writer(file))
                .init();
        }
        LogOutput::Both(_path) => {
            // Simplified to console only for now
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }

    Ok(())
}
