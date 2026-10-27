use std::collections::BTreeMap;

use egui::Pos2;
use harper_core::Span;
use harper_core::linting::{Lint, Suggestion};

use crate::os_broker::{AccessibilityPermissionStatus, OsBroker};
use crate::rect::ActionableLint;

/// A broker that allows one to mock an operating system for testing or debugging purposes.
/// For the time being, most functionality does nothing.
pub struct DebugBroker;

impl OsBroker for DebugBroker {
    fn get_boxes(
        &mut self,
        _: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Option<Vec<ActionableLint>> {
        Some(vec![ActionableLint::new(
            crate::rect::Rect::new(100.0, 100.0, 50.0, 25.0),
            "ExampleRule".to_string(),
            Lint {
                span: Span::new(0, 4),
                lint_kind: harper_core::linting::LintKind::Miscellaneous,
                suggestions: vec![
                    Suggestion::ReplaceWith("Sug1".chars().collect()),
                    Suggestion::Remove,
                ],
                message: "This is a mocked message for a suggestion.".to_string(),
                priority: 1,
            },
            "This is some mocking text.".to_string(),
            |_| {},
        )])
    }

    fn cursor_position(&self) -> Option<Pos2> {
        Some(Pos2 { x: 105.0, y: 105.0 })
    }

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }

    fn request_accessibility_permission(&self) -> AccessibilityPermissionStatus {
        AccessibilityPermissionStatus::Granted
    }

    fn installed_application_bundle_ids(&self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }

    fn application_icon_png(&self, _bundle_id: &str) -> Result<Vec<u8>, String> {
        Err("Cannot get icon PNGs with the mocked broker.".to_string())
    }

    fn launch_app_bundle(&self, _bundle_id: &str) -> Result<(), String> {
        Err("Cannot launch an app with the mocked broker.".to_string())
    }

    fn search_apps(&self, _query: &str) -> Result<Vec<crate::os_broker::AppSearchResult>, String> {
        Err("Cannot search installed apps with the mocked broker.".to_string())
    }
}
