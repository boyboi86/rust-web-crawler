// =============================================================================
// API MODULE - Tauri Command Handlers
// =============================================================================
// This module contains all Tauri command handlers that provide the public
// API interface between the TypeScript frontend and Rust backend.

pub mod commands;

// Re-export public API
pub use commands::*;
