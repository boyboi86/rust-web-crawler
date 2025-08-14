/// Logging setup utilities
use anyhow::Error;

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

/// Log application startup information
pub fn log_startup_info(app_name: &str, version: &str) {
    tracing::info!(
        app_name = app_name,
        version = version,
        "Application starting"
    );
}

/// Log session summary
pub fn log_session_summary(
    session_id: &str,
    total_urls: usize,
    successful: usize,
    failed: usize,
    duration_ms: u64,
) {
    tracing::info!(
        session_id = session_id,
        total_urls = total_urls,
        successful = successful,
        failed = failed,
        duration_ms = duration_ms,
        success_rate = %format!("{:.1}%", (successful as f64 / total_urls as f64) * 100.0),
        "Session completed"
    );
}
