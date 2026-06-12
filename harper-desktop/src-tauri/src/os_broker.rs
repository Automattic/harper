use harper_core::linting::Lint;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::rect::ActionableLint;

pub type LintText = Box<dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AccessibilityPermissionStatus {
    Granted,
    NotGranted,
    Unsupported,
}

/// Provides platform-specific state needed by the highlighter without coupling rendering to an OS.
///
/// The highlighter needs both accessibility-derived lint rectangles and global cursor position, but
/// those APIs are platform-specific. This trait keeps the event loop and renderer independent from
/// macOS accessibility and pointer APIs.
pub trait OsBroker {
    fn get_boxes(
        &mut self,
        lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint>;

    fn cursor_position(&self) -> Option<egui::Pos2>;

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Unsupported
    }

    fn request_accessibility_permission(&self) -> AccessibilityPermissionStatus {
        self.accessibility_permission_status()
    }

    fn system_integration_display_name(&self, _bundle_id: &str) -> Option<String> {
        None
    }

    fn integration_display_name(&self, bundle_id: &str) -> String {
        self.system_integration_display_name(bundle_id)
            .unwrap_or_else(|| fallback_integration_display_name(bundle_id))
    }

    /// Returns the bundle identifiers for installed graphical applications.
    ///
    /// Implementations should return stable bundle ID strings, sorted and deduplicated where
    /// possible. Platforms that do not support bundle IDs should return an error.
    fn installed_application_bundle_ids(&self) -> Result<Vec<String>, String> {
        Err("Listing installed application bundle IDs is only supported on macOS.".to_string())
    }

    /// Returns the application icon for `bundle_id` encoded as PNG bytes.
    ///
    /// The broker returns raw bytes so callers can choose their own transport format, such as a
    /// Tauri command converting them into a data URL.
    fn application_icon_png(&self, _bundle_id: &str) -> Result<Vec<u8>, String> {
        Err("Reading application icons by bundle ID is only supported on macOS.".to_string())
    }

    fn launch_app_bundle(&self, _bundle_id: &str) -> Result<(), String> {
        Err("Launching apps by bundle ID is only supported on macOS.".to_string())
    }
}

fn fallback_integration_display_name(bundle_id: &str) -> String {
    let trimmed = bundle_id.trim();

    if trimmed.is_empty() {
        return "Unknown app".to_string();
    }

    trimmed
        .split('.')
        .rev()
        .find(|segment| !segment.is_empty())
        .unwrap_or(trimmed)
        .to_string()
}

/// No-op platform broker for targets that do not have an OS implementation yet.
///
/// This lets the highlighter compile on non-macOS platforms while making it explicit that there is
/// currently no accessibility or cursor integration there.
#[cfg(not(target_os = "macos"))]
#[derive(Default)]
pub struct NoopBroker;

#[cfg(not(target_os = "macos"))]
impl OsBroker for NoopBroker {
    fn get_boxes(
        &mut self,
        _lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint> {
        Vec::new()
    }

    fn cursor_position(&self) -> Option<egui::Pos2> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::fallback_integration_display_name;

    #[test]
    fn fallback_handles_empty_bundle_ids() {
        assert_eq!(fallback_integration_display_name(""), "Unknown app");
        assert_eq!(fallback_integration_display_name("   "), "Unknown app");
    }

    #[test]
    fn fallback_uses_last_non_empty_segment_without_changing_case() {
        assert_eq!(
            fallback_integration_display_name("com.microsoft.VSCode"),
            "VSCode"
        );
        assert_eq!(
            fallback_integration_display_name("com.googlecode.iterm2"),
            "iterm2"
        );
        assert_eq!(fallback_integration_display_name("com.example."), "example");
    }
}
