use crate::rect::PositionedLint;

/// Provides platform-specific state needed by the highlighter without coupling rendering to an OS.
///
/// The highlighter needs both accessibility-derived lint rectangles and global cursor position, but
/// those APIs are platform-specific. This trait keeps the event loop and renderer independent from
/// macOS accessibility and pointer APIs.
pub trait OsBroker {
    fn get_boxes(&mut self) -> Vec<PositionedLint>;

    fn cursor_position(&self) -> Option<egui::Pos2>;
}

/// No-op platform broker for targets that do not have an OS implementation yet.
///
/// This lets the highlighter compile on non-macOS platforms while making it explicit that there is
/// currently no accessibility or cursor integration there.
pub struct NoopBroker;

impl OsBroker for NoopBroker {
    fn get_boxes(&mut self) -> Vec<PositionedLint> {
        Vec::new()
    }

    fn cursor_position(&self) -> Option<egui::Pos2> {
        None
    }
}
