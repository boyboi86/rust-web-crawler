// =============================================================================
// TAURI DESKTOP APPLICATION - MAIN LIBRARY
// =============================================================================
// This module provides the main entry point for the Tauri desktop application
// that wraps the Rust web crawler with a TypeScript frontend.
//
// Architecture follows 4-layer pattern:
// - api: Tauri command handlers (API interface)
// - actors: Actor pattern for concurrency and Send/non-Send bridging
// - core: Data types and models
// - utils: Utilities, validation, and helpers

// Module declarations
pub mod actors;
pub mod api;
pub mod commands;
pub mod config;
pub mod core;
pub mod utils;

// Re-exports for convenience
use crate::actors::CrawlerBridge;
use crate::api::*;
use crate::commands::config_commands::*;
use log::LevelFilter;

/// Application metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Main entry point for the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the application
    println!("🚀 Starting {} v{}", NAME, VERSION);

    tauri::Builder::default()
        // Register crawler bridge
        .manage(CrawlerBridge::new())
        // Register Tauri commands (API endpoints)
        .invoke_handler(tauri::generate_handler![
            // Legacy API commands
            get_default_config,
            validate_crawl_request_api,
            start_crawl,
            get_crawl_status,
            stop_crawl,
            // New configuration API commands
            get_app_config,
            get_config_preset,
            validate_config,
            get_config_summary,
            get_env_documentation,
            reset_config_to_defaults,
        ])
        // Setup application
        .setup(|app| {
            // Initialize logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(LevelFilter::Info)
                        .build(),
                )?;
                println!("🔧 Debug logging enabled");
            }

            println!("✅ Application setup complete");
            Ok(())
        })
        // Run the application
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
