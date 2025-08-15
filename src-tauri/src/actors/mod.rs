// =============================================================================
// ACTORS MODULE - Actor Pattern Implementation
// =============================================================================
// This module contains actor pattern implementations for managing
// concurrent operations and bridging Send/non-Send boundaries.

pub mod crawler_actor;

// Re-export public actor interfaces
pub use crawler_actor::*;
