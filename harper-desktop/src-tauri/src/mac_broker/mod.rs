mod accessibility_activation;
mod accessibility_permissions;
mod accessibility_text;
mod app_catalog;
mod app_icons;
mod core_foundation_utilities;
mod focused_application;
mod window_stability;

use accessibility::TreeWalker;
use accessibility::ui_element::AXUIElement;
use accessibility_sys::{error_string, pid_t};
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use harper_core::linting::Lint;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error as StdError,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::config::{Config, Integration};
use crate::os_broker::{
    AccessibilityPermissionStatus, AppSearchResult, OsBroker, in_memory_app_search_results,
};
use crate::rect::ActionableLint;

use self::accessibility_activation::{
    ACCESSIBILITY_ACTIVATION_RETRY_INTERVAL, AccessibilityActivationState,
    AccessibilityActivationStatus, AccessibilityActivationVerification,
    accessibility_activation_verification_retry_interval, instant_after,
    is_unsupported_accessibility_activation_error, release_accessibility_activation,
    set_enhanced_user_interface_preserving_previous, verify_accessibility_activation,
};
use self::accessibility_text::RectCollector;
use self::window_stability::{
    WINDOW_MOVEMENT_SETTLE_DURATION, WindowMovementState, frontmost_window_frame_for_pid,
    settled_window_state, window_frame_changed,
};

/// macOS implementation of the OS data the highlighter needs.
///
/// `MacBroker` owns focus memory because clicking the overlay can make the highlighter process the
/// focused application. Remembering the last non-highlighter PID lets accessibility reads continue
/// targeting the app the user was reviewing.
pub struct MacBroker {
    last_focused: Option<pid_t>,
    integrations: Arc<Mutex<Vec<Integration>>>,
    application_icon_cache: Mutex<HashMap<String, Vec<u8>>>,
    installed_app_search_results: Mutex<Option<Vec<AppSearchResult>>>,
    window_movement: Option<WindowMovementState>,
    accessibility_activation: Option<AccessibilityActivationState>,
}

impl MacBroker {
    pub fn new(integrations: Arc<Mutex<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            application_icon_cache: Mutex::new(HashMap::new()),
            installed_app_search_results: Mutex::new(None),
            window_movement: None,
            accessibility_activation: None,
        }
    }

    fn target_pid(&mut self) -> Result<Option<pid_t>, Box<dyn StdError>> {
        let focused_pid = focused_application::focused_window_pid()?;
        let current_pid = std::process::id() as pid_t;

        if focused_pid == current_pid {
            Ok(self.last_focused)
        } else {
            self.last_focused = Some(focused_pid);
            Ok(Some(focused_pid))
        }
    }

    fn window_is_moving(&mut self, pid: pid_t) -> bool {
        let Some(frame) = frontmost_window_frame_for_pid(pid) else {
            self.window_movement = None;
            return true;
        };

        let now = Instant::now();
        let Some(state) = &mut self.window_movement else {
            self.window_movement = Some(settled_window_state(pid, frame, now));
            return false;
        };

        if state.pid != pid {
            *state = settled_window_state(pid, frame, now);
            return false;
        }

        if window_frame_changed(state.frame, frame) {
            state.frame = frame;
            state.last_changed_at = now;
            return true;
        }

        now.duration_since(state.last_changed_at) < WINDOW_MOVEMENT_SETTLE_DURATION
    }

    /// Clears activation state and restores any saved AX attribute value.
    fn reset_accessibility_activation(&mut self) {
        if let Some(state) = self.accessibility_activation.take() {
            release_accessibility_activation(&state);
        }
    }

    /// Activates the focused app and waits until text range bounds are usable.
    fn ensure_accessibility_activation(
        &mut self,
        pid: pid_t,
        bundle_id: &str,
        app: &AXUIElement,
    ) -> bool {
        let needs_new_activation = match &self.accessibility_activation {
            Some(state) => state.pid != pid || state.bundle_id != bundle_id,
            None => true,
        };

        if needs_new_activation {
            self.reset_accessibility_activation();
            return self.request_enhanced_user_interface(pid, bundle_id, app);
        }

        let Some(status) = self
            .accessibility_activation
            .as_ref()
            .map(|state| state.status)
        else {
            return self.request_enhanced_user_interface(pid, bundle_id, app);
        };

        let now = Instant::now();
        match status {
            AccessibilityActivationStatus::Ready => true,
            AccessibilityActivationStatus::Pending {
                ready_at,
                verification_attempts,
            } => {
                if now < ready_at {
                    return false;
                }

                let verification = verify_accessibility_activation(app);

                if verification == AccessibilityActivationVerification::FoundTextRangeBounds {
                    eprintln!(
                        "Accessibility activation verified for {bundle_id} pid {pid}: {verification:?}"
                    );
                    if let Some(state) = &mut self.accessibility_activation {
                        state.status = AccessibilityActivationStatus::Ready;
                    }
                    return true;
                }

                let next_verification_attempts = verification_attempts.saturating_add(1);
                let retry_interval = accessibility_activation_verification_retry_interval(
                    next_verification_attempts,
                );

                eprintln!(
                    "Accessibility activation for {bundle_id} pid {pid} is not ready for text metrics yet: {verification:?}; retrying verification in {} ms",
                    retry_interval.as_millis()
                );
                if let Some(state) = &mut self.accessibility_activation {
                    state.status = AccessibilityActivationStatus::Pending {
                        ready_at: instant_after(now, retry_interval),
                        verification_attempts: next_verification_attempts,
                    };
                }

                false
            }
            AccessibilityActivationStatus::RetryLater => {
                let Some(last_attempted_at) = self
                    .accessibility_activation
                    .as_ref()
                    .map(|state| state.last_attempted_at)
                else {
                    return self.request_enhanced_user_interface(pid, bundle_id, app);
                };

                if now.duration_since(last_attempted_at) < ACCESSIBILITY_ACTIVATION_RETRY_INTERVAL {
                    return false;
                }

                self.reset_accessibility_activation();
                self.request_enhanced_user_interface(pid, bundle_id, app)
            }
        }
    }

    /// Requests `AXEnhancedUserInterface`, preserving the previous value if readable.
    fn request_enhanced_user_interface(
        &mut self,
        pid: pid_t,
        bundle_id: &str,
        app: &AXUIElement,
    ) -> bool {
        let now = Instant::now();
        let settle_duration = accessibility_activation::CHROMIUM_ACCESSIBILITY_SETTLE_DURATION;

        match set_enhanced_user_interface_preserving_previous(app, true) {
            Ok(enhanced_user_interface_restore_value) => {
                eprintln!(
                    "Requested AXEnhancedUserInterface for {bundle_id} pid {pid}; waiting for Chromium debounce"
                );
                self.accessibility_activation = Some(AccessibilityActivationState {
                    pid,
                    bundle_id: bundle_id.to_string(),
                    status: AccessibilityActivationStatus::Pending {
                        ready_at: instant_after(now, settle_duration),
                        verification_attempts: 0,
                    },
                    last_attempted_at: now,
                    enhanced_user_interface_restore_value,
                });
                false
            }
            Err(error) if is_unsupported_accessibility_activation_error(error) => {
                eprintln!(
                    "AXEnhancedUserInterface unsupported for {bundle_id} pid {pid}: {}; proceeding to verification",
                    error_string(error)
                );
                self.accessibility_activation = Some(AccessibilityActivationState {
                    pid,
                    bundle_id: bundle_id.to_string(),
                    status: AccessibilityActivationStatus::Pending {
                        ready_at: now,
                        verification_attempts: 0,
                    },
                    last_attempted_at: now,
                    enhanced_user_interface_restore_value: None,
                });
                false
            }
            Err(error) => {
                eprintln!(
                    "Unable to request AXEnhancedUserInterface for {bundle_id} pid {pid}: {}",
                    error_string(error)
                );
                self.accessibility_activation = Some(AccessibilityActivationState {
                    pid,
                    bundle_id: bundle_id.to_string(),
                    status: AccessibilityActivationStatus::RetryLater,
                    last_attempted_at: now,
                    enhanced_user_interface_restore_value: None,
                });
                false
            }
        }
    }

    fn installed_app_search_results(&self) -> Result<Vec<AppSearchResult>, String> {
        if let Some(results) = self
            .installed_app_search_results
            .lock()
            .map_err(|error| format!("Failed to read app search cache: {error}"))?
            .clone()
        {
            return Ok(results);
        }

        let results = app_catalog::discover_app_search_results()?;
        *self
            .installed_app_search_results
            .lock()
            .map_err(|error| format!("Failed to update app search cache: {error}"))? =
            Some(results.clone());

        Ok(results)
    }
}

impl Default for MacBroker {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(Config::curated_integrations())))
    }
}

impl Drop for MacBroker {
    fn drop(&mut self) {
        self.reset_accessibility_activation();
    }
}

pub(super) type LintCallback<'a> = dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>> + 'a;

impl OsBroker for MacBroker {
    fn get_boxes(&mut self, lint_text: &mut LintCallback) -> Vec<ActionableLint> {
        let pid = match self.target_pid() {
            Ok(Some(pid)) => pid,
            Ok(None) => {
                self.window_movement = None;
                self.reset_accessibility_activation();
                return Vec::new();
            }
            Err(err) => {
                self.window_movement = None;
                self.reset_accessibility_activation();
                eprintln!("Unable to identify focused window: {err}");
                return Vec::new();
            }
        };

        let bundle_identifier = match focused_application::bundle_identifier_for_pid(pid) {
            Ok(Some(bundle_identifier)) => bundle_identifier,
            Ok(None) => {
                self.window_movement = None;
                self.reset_accessibility_activation();
                return Vec::new();
            }
            Err(error) => {
                self.window_movement = None;
                self.reset_accessibility_activation();
                eprintln!("Unable to identify focused app bundle: {error}");
                return Vec::new();
            }
        };

        let integration_enabled = match self.integrations.lock() {
            Ok(integrations) => {
                Config::is_integration_enabled_in(&integrations, &bundle_identifier)
            }
            Err(error) => {
                eprintln!("Unable to read integrations: {error}");
                false
            }
        };

        if !integration_enabled {
            self.window_movement = None;
            self.reset_accessibility_activation();
            return Vec::new();
        }

        if self.window_is_moving(pid) {
            return Vec::new();
        }

        let el = AXUIElement::application(pid);
        if !self.ensure_accessibility_activation(pid, &bundle_identifier, &el) {
            return Vec::new();
        }

        let walker = TreeWalker::new();
        let collector = RectCollector::new(lint_text);

        walker.walk(&el, &collector);

        collector.unwrap_rects()
    }

    fn cursor_position(&self) -> Option<egui::Pos2> {
        let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
        let event = CGEvent::new(source).ok()?;
        let location = event.location();

        Some(egui::pos2(location.x as f32, location.y as f32))
    }

    fn accessibility_permission_status(&self) -> AccessibilityPermissionStatus {
        accessibility_permissions::accessibility_permission_status()
    }

    fn request_accessibility_permission(&self) -> AccessibilityPermissionStatus {
        accessibility_permissions::request_accessibility_permission()
    }

    fn system_integration_display_name(&self, bundle_id: &str) -> Option<String> {
        app_catalog::system_integration_display_name(bundle_id)
    }

    fn installed_application_bundle_ids(&self) -> Result<Vec<String>, String> {
        app_catalog::installed_application_bundle_ids()
    }

    fn application_icon_png(&self, bundle_id: &str) -> Result<Vec<u8>, String> {
        let bundle_id = bundle_id.trim();

        if bundle_id.is_empty() {
            return Err("Bundle ID cannot be empty.".to_string());
        }

        if let Some(icon_png) = self
            .application_icon_cache
            .lock()
            .map_err(|error| format!("Failed to read application icon cache: {error}"))?
            .get(bundle_id)
            .cloned()
        {
            return Ok(icon_png);
        }

        let icon_png = app_icons::application_icon_png(bundle_id)?;
        self.application_icon_cache
            .lock()
            .map_err(|error| format!("Failed to update application icon cache: {error}"))?
            .insert(bundle_id.to_string(), icon_png.clone());

        Ok(icon_png)
    }

    fn launch_app_bundle(&self, bundle_id: &str) -> Result<(), String> {
        app_catalog::launch_app_bundle(bundle_id)
    }

    fn search_apps(&self, query: &str) -> Result<Vec<AppSearchResult>, String> {
        let query = query.trim();
        let installed_apps = self.installed_app_search_results()?;

        let in_memory_results = in_memory_app_search_results(&installed_apps, query);

        if query.is_empty()
            || in_memory_results
                .iter()
                .any(|result| result.bundle_id == query)
        {
            return Ok(in_memory_results);
        }

        if let Some(app_path) = app_catalog::application_path_for_bundle_id(query) {
            return Ok(vec![
                app_catalog::app_search_result_from_app_path(&app_path)
                    .unwrap_or_else(|| app_catalog::app_search_result_from_bundle_id(self, query)),
            ]);
        }

        Ok(in_memory_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rect::Rect;
    use std::time::Duration;

    #[test]
    fn supported_text_roles_include_text_area_and_text_field() {
        assert!(accessibility_text::is_supported_text_role("AXTextArea"));
        assert!(accessibility_text::is_supported_text_role("AXTextField"));
    }

    #[test]
    fn supported_text_roles_reject_unrelated_roles() {
        assert!(!accessibility_text::is_supported_text_role("AXButton"));
        assert!(!accessibility_text::is_supported_text_role("AXStaticText"));
        assert!(!accessibility_text::is_supported_text_role(""));
    }

    #[test]
    fn finds_char_subslice_respecting_offset() {
        let haystack: Vec<char> = "one\ntwo two".chars().collect();
        let needle: Vec<char> = "two".chars().collect();

        assert_eq!(
            accessibility_text::find_char_subslice(&haystack, &needle, 0),
            Some(4)
        );
        assert_eq!(
            accessibility_text::find_char_subslice(&haystack, &needle, 5),
            Some(8)
        );
        assert_eq!(
            accessibility_text::find_char_subslice(&haystack, &needle, 9),
            None
        );
        assert_eq!(
            accessibility_text::find_char_subslice(&haystack, &[], 0),
            None
        );
        assert_eq!(
            accessibility_text::find_char_subslice(&[], &needle, 0),
            None
        );
    }

    #[test]
    fn chromium_activation_uses_chromium_settle_duration() {
        assert_eq!(
            accessibility_activation::CHROMIUM_ACCESSIBILITY_SETTLE_DURATION,
            Duration::from_secs(3)
        );
    }

    #[test]
    fn deduplicates_and_sorts_bundle_ids() {
        assert_eq!(
            app_catalog::deduplicate_and_sort_bundle_ids(vec![
                "com.example.Beta".to_string(),
                " com.example.Alpha ".to_string(),
                "com.example.Beta".to_string(),
                "".to_string(),
                "(null)".to_string(),
            ]),
            vec!["com.example.Alpha", "com.example.Beta"]
        );
    }

    #[test]
    fn escapes_spotlight_strings() {
        assert_eq!(
            app_catalog::escape_spotlight_string(r#"com.example.\"quoted\""#),
            r#"com.example.\\\"quoted\\\""#
        );
    }

    #[test]
    fn empty_bundle_id_has_no_application_path() {
        assert_eq!(app_catalog::application_path_for_bundle_id(""), None);
        assert_eq!(app_catalog::application_path_for_bundle_id("   "), None);
    }

    #[test]
    fn verification_retry_slows_after_fast_attempts() {
        assert_eq!(
            accessibility_activation::accessibility_activation_verification_retry_interval(
                accessibility_activation::ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS
            ),
            accessibility_activation::ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL
        );
        assert_eq!(
            accessibility_activation::accessibility_activation_verification_retry_interval(
                accessibility_activation::ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS + 1
            ),
            accessibility_activation::ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL
        );
    }

    #[test]
    fn text_range_bounds_probe_requires_non_zero_geometry() {
        let usable = accessibility_text::TextRangeBoundsProbe::Success(Rect {
            x: 10.0,
            y: 20.0,
            width: 1.0,
            height: 12.0,
        });
        let unavailable = accessibility_text::TextRangeBoundsProbe::Unavailable;

        assert!(usable.has_usable_text_metrics());
        assert!(!unavailable.has_usable_text_metrics());
        assert!(accessibility_text::rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }));
        assert!(!accessibility_text::rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 1.0,
        }));
    }
}
