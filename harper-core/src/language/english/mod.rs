//! English language module.

pub mod dialects;
pub mod language_detection;
pub mod module;
pub mod stats;

// Re-export dialects for external use
pub use dialects::*;
// Re-export language detection
pub use language_detection::EnglishDetector;
