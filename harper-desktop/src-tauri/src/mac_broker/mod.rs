use cached::cached;
mod accessibility_activation;
mod accessibility_text;
mod app_catalog;
mod app_icons;
mod app_search_index;
mod core_foundation_utilities;
mod focused_window_pid;
mod window_stability;

use accessibility::TreeWalker;
use accessibility::ui_element::AXUIElement;
use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt,
};
use accessibility_sys::{error_string, pid_t};
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use harper_core::linting::Lint;
use objc2_app_kit::NSRunningApplication;
use std::process::Command;
use std::time::Duration;
use std::{
    collections::{BTreeMap, HashMap},
    error::Error as StdError,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::config::{Config, Integration};
use crate::os_broker::{AccessibilityPermissionStatus, AppSearchResult, OsBroker};
use crate::rect::ActionableLint;

use self::accessibility_activation::{
    ACCESSIBILITY_ACTIVATION_RETRY_INTERVAL, AccessibilityActivationState,
    AccessibilityActivationStatus, AccessibilityActivationVerification,
    accessibility_activation_verification_retry_interval,
    is_unsupported_accessibility_activation_error, release_accessibility_activation,
    set_enhanced_user_interface_preserving_previous, verify_accessibility_activation,
};
use self::accessibility_text::RectCollector;
use self::app_search_index::AppSearchIndex;
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
    /// The PID of the most recently focused PID, along with the time the measurement was taken.
    last_focused: Option<(pid_t, Instant)>,
    integrations: Arc<Mutex<Vec<Integration>>>,
    application_icon_cache: Mutex<HashMap<String, Vec<u8>>>,
    installed_app_search_index: Mutex<AppSearchIndex>,
    window_movement: Option<WindowMovementState>,
    accessibility_activation: Option<AccessibilityActivationState>,
}

impl MacBroker {
    pub fn new(integrations: Arc<Mutex<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            application_icon_cache: Mutex::new(HashMap::new()),
            installed_app_search_index: Mutex::new(AppSearchIndex::new()),
            window_movement: None,
            accessibility_activation: None,
        }
    }

    /// The process ID of the currently focused window.
    /// In the interest of performance, the returned value may be slightly stale.
    fn target_pid(&mut self) -> Result<Option<pid_t>, Box<dyn StdError>> {
        if let Some((last_focused, measurement_time)) = self.last_focused {
            if Instant::now().duration_since(measurement_time).as_secs() < 3 {
                return Ok(Some(last_focused));
            }
        }

        let focused_pid = focused_window_pid::focused_window_pid()?;
        let current_pid = std::process::id() as pid_t;

        if focused_pid == current_pid {
            Ok(self.last_focused.map(|v| v.0))
        } else {
            self.last_focused = Some((focused_pid, Instant::now()));
            Ok(Some(focused_pid))
        }
    }

    /// Check if the fronmost window for a given process is currently moving.
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

        let bundle_identifier = match bundle_identifier_for_pid(pid) {
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

        // Hide highlights while window is moving to avoid "sliding" behavior.
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
        if unsafe { AXIsProcessTrusted() } {
            AccessibilityPermissionStatus::Granted
        } else {
            AccessibilityPermissionStatus::NotGranted
        }
    }

    fn request_accessibility_permission(&self) -> AccessibilityPermissionStatus {
        let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
        let prompt_value = CFBoolean::true_value();
        let options: CFDictionary<CFString, CFBoolean> =
            CFDictionary::from_CFType_pairs(&[(prompt_key, prompt_value)]);

        if unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) } {
            AccessibilityPermissionStatus::Granted
        } else {
            AccessibilityPermissionStatus::NotGranted
        }
    }

    fn system_integration_display_name(&self, bundle_id: &str) -> String {
        app_catalog::system_integration_display_name(bundle_id)
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
        let bundle_id: &str = bundle_id;
        let bundle_id = bundle_id.trim();
        if bundle_id.is_empty() {
            return Err("Bundle ID cannot be empty.".to_string());
        }
        Command::new("open")
            .arg("-b")
            .arg(bundle_id)
            .spawn()
            .map_err(|error| format!("Failed to launch {bundle_id}: {error}"))?;
        Ok(())
    }

    fn search_apps(&self, query: &str) -> Result<Vec<AppSearchResult>, String> {
        let mut lock = self
            .installed_app_search_index
            .lock()
            .map_err(|_| "Could not lock search index.".to_owned())?;

        if lock.is_empty() {
            lock.populate()?;
        }

        Ok(lock.search(query))
    }
}

#[cached(max_size = 1_000)]
fn bundle_identifier_for_pid(pid: pid_t) -> Result<Option<String>, Box<dyn StdError>> {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return Ok(None);
    };
    let Some(bundle_identifier) = app.bundleIdentifier() else {
        return Ok(None);
    };

    Ok(Some(bundle_identifier.to_string()))
}

/// Adds a duration to an instant without panicking on overflow.
fn instant_after(now: Instant, duration: Duration) -> Instant {
    now.checked_add(duration).unwrap_or(now)
}
