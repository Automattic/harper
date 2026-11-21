mod default;
mod discord;
mod router;

use accessibility::ui_element::AXUIElement;
use default::DefaultTextHandler;
use discord::DiscordTextHandler;
use harper_core::{Span, linting::Suggestion};
pub use router::RoutingTextHandler;
use std::sync::Arc;

type TextResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Editor captured at lint time; popup callbacks keep this target even if focus moves.
#[derive(Clone)]
pub struct TextTarget {
    pub pid: i32,
    pub bundle_id: Arc<str>,
    pub element: AXUIElement,
}

/// App-specific text access, independent of lint geometry and popup focus.
pub trait TextHandler: Send + Sync {
    /// Read the editor snapshot to lint.
    fn read_text(&self, target: &TextTarget) -> TextResult<String>;
    /// Apply a suggestion at character offsets in the original snapshot (not UTF-16 offsets).
    fn write_text(
        &self,
        target: &TextTarget,
        snapshot: &str,
        span: Span<char>,
        suggestion: &Suggestion,
    ) -> TextResult<()>;
}
