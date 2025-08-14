/// Utility functions and helpers
///
/// This module provides common utility functions used across the application,
/// reducing code duplication in main.rs and other modules.
pub mod html;
pub mod language;
pub mod logging;
pub mod session;
pub mod url_validation;

pub use html::{extract_links_from_html, extract_title_from_html};
pub use language::detect_language;
pub use logging::init_logging;
pub use session::log_session_summary;
pub use url_validation::is_valid_crawl_url;
