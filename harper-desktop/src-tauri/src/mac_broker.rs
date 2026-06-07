use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCopyParameterizedAttributeValue, AXUIElementGetPid, AXUIElementSetAttributeValue,
    AXValueCreate, AXValueGetType, AXValueGetValue, AXValueRef, error_string,
    kAXBoundsForRangeParameterizedAttribute, kAXErrorAttributeUnsupported, kAXErrorIllegalArgument,
    kAXErrorNoValue, kAXErrorNotImplemented, kAXErrorParameterizedAttributeUnsupported,
    kAXErrorSuccess, kAXFocusedApplicationAttribute, kAXTrustedCheckOptionPrompt,
    kAXValueTypeCFRange, kAXValueTypeCGRect, pid_t,
};
use core::{ffi::c_void, mem::MaybeUninit};
use core_foundation::array::CFArray;
use core_foundation::base::{CFRange, CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer,
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
};
use harper_core::linting::{Lint, Suggestion};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::NSRect;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    error::Error as StdError,
    path::Path,
    process::Command,
    ptr,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::config::{Config, Integration};
use crate::os_broker::{AccessibilityPermissionStatus, OsBroker};
use crate::rect::{ActionableLint, Rect};

const WINDOW_MOVEMENT_SETTLE_DURATION: Duration = Duration::from_millis(150);
const ACCESSIBILITY_ACTIVATION_RETRY_INTERVAL: Duration = Duration::from_secs(10);
const ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL: Duration = Duration::from_millis(250);
const ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL: Duration = Duration::from_secs(1);
const ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS: u8 = 20;
const CHROMIUM_ACCESSIBILITY_SETTLE_DURATION: Duration = Duration::from_secs(3);
const WINDOW_FRAME_TOLERANCE: f64 = 0.5;
type WindowInfo = CFDictionary<CFString, CFType>;

/// macOS implementation of the OS data the highlighter needs.
///
/// `MacBroker` owns focus memory because clicking the overlay can make the highlighter process the
/// focused application. Remembering the last non-highlighter PID lets accessibility reads continue
/// targeting the app the user was reviewing.
pub struct MacBroker {
    last_focused: Option<pid_t>,
    integrations: Rc<RefCell<Vec<Integration>>>,
    window_movement: Option<WindowMovementState>,
    accessibility_activation: Option<AccessibilityActivationState>,
}

#[derive(Debug, Clone)]
struct WindowMovementState {
    pid: pid_t,
    frame: Rect,
    last_changed_at: Instant,
}

/// Tracks accessibility activation for the focused app and any value to restore.
#[derive(Debug, Clone)]
struct AccessibilityActivationState {
    pid: pid_t,
    bundle_id: String,
    status: AccessibilityActivationStatus,
    last_attempted_at: Instant,
    enhanced_user_interface_restore_value: Option<bool>,
}

/// State of the focused app's accessibility activation attempt.
#[derive(Debug, Clone, Copy)]
enum AccessibilityActivationStatus {
    Pending {
        ready_at: Instant,
        verification_attempts: u8,
    },
    Ready,
    RetryLater,
}

/// Result of checking whether an activated app exposes usable text geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessibilityActivationVerification {
    FoundTextRangeBounds,
    FoundSupportedTextElement,
    NoSupportedTextElement,
}

impl MacBroker {
    pub fn new(integrations: Rc<RefCell<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            window_movement: None,
            accessibility_activation: None,
        }
    }

    fn target_pid(&mut self) -> Result<Option<pid_t>, Box<dyn StdError>> {
        let focused_pid = focused_window_pid()?;
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
        let settle_duration = CHROMIUM_ACCESSIBILITY_SETTLE_DURATION;

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
        Self::new(Rc::new(RefCell::new(Config::curated_integrations())))
    }
}

impl Drop for MacBroker {
    fn drop(&mut self) {
        self.reset_accessibility_activation();
    }
}

type LintCallback<'a> = dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>> + 'a;

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

        if !Config::is_integration_enabled_in(&self.integrations.borrow(), &bundle_identifier) {
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

    fn system_integration_display_name(&self, bundle_id: &str) -> Option<String> {
        system_integration_display_name(bundle_id)
    }

    fn launch_app_bundle(&self, bundle_id: &str) -> Result<(), String> {
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
}

fn system_integration_display_name(bundle_id: &str) -> Option<String> {
    let bundle_id = bundle_id.trim();

    if bundle_id.is_empty() {
        return None;
    }

    let predicate_bundle_id = bundle_id.replace('\\', "\\\\").replace('"', "\\\"");
    let output = Command::new("mdfind")
        .arg(format!(
            "kMDItemCFBundleIdentifier == \"{predicate_bundle_id}\""
        ))
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| display_name_from_app_path(line.trim()))
        .next()
}

fn display_name_from_app_path(path: &str) -> Option<String> {
    let file_name = Path::new(path).file_name()?.to_str()?;
    let display_name = file_name.strip_suffix(".app").unwrap_or(file_name).trim();

    if display_name.is_empty() {
        None
    } else {
        Some(display_name.to_string())
    }
}

fn focused_window_pid() -> Result<pid_t, Box<dyn StdError>> {
    let system = AXUIElement::system_wide();
    let app = match ax_element_attribute(&system, kAXFocusedApplicationAttribute) {
        Ok(app) => app,
        Err(error) => {
            if let Some(pid) = frontmost_application_pid() {
                return Ok(pid);
            }

            return Err(error);
        }
    };

    let mut pid: pid_t = 0;
    let err = unsafe { AXUIElementGetPid(app.as_concrete_TypeRef(), &mut pid) };

    if err != kAXErrorSuccess {
        if let Some(pid) = frontmost_application_pid() {
            return Ok(pid);
        }

        return Err(format!("AXUIElementGetPid failed: {}", error_string(err)).into());
    }

    Ok(pid)
}

/// Fallback PID lookup when system-wide focused-application AX lookup fails.
fn frontmost_application_pid() -> Option<pid_t> {
    NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map(|app| app.processIdentifier())
}

fn bundle_identifier_for_pid(pid: pid_t) -> Result<Option<String>, Box<dyn StdError>> {
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
        return Ok(None);
    };
    let Some(bundle_identifier) = app.bundleIdentifier() else {
        return Ok(None);
    };

    Ok(Some(bundle_identifier.to_string()))
}

/// Backs off activation verification after repeated misses.
fn accessibility_activation_verification_retry_interval(verification_attempts: u8) -> Duration {
    if verification_attempts <= ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS {
        ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL
    } else {
        ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL
    }
}

/// Adds a duration to an instant without panicking on overflow.
fn instant_after(now: Instant, duration: Duration) -> Instant {
    now.checked_add(duration).unwrap_or(now)
}

/// Restores `AXEnhancedUserInterface` when a previous value was captured.
fn release_accessibility_activation(state: &AccessibilityActivationState) {
    if state.enhanced_user_interface_restore_value.is_none() {
        return;
    }

    let app = AXUIElement::application(state.pid);

    restore_boolean_attribute(
        &app,
        "AXEnhancedUserInterface",
        state.enhanced_user_interface_restore_value,
    );
}

/// Returns whether an AX error means the target does not support this attribute.
fn is_unsupported_accessibility_activation_error(error: i32) -> bool {
    error == kAXErrorAttributeUnsupported
        || error == kAXErrorNoValue
        || error == kAXErrorNotImplemented
}

/// Sets `AXEnhancedUserInterface` while returning its old value when readable.
fn set_enhanced_user_interface_preserving_previous(
    app: &AXUIElement,
    enabled: bool,
) -> Result<Option<bool>, i32> {
    match set_boolean_attribute_preserving_previous(app, "AXEnhancedUserInterface", enabled) {
        Ok(previous) => Ok(previous),
        Err(error) => {
            // Some apps (notably Electron/Chromium ones such as Slack) reject
            // `AXEnhancedUserInterface` (e.g. with kAXErrorNotImplemented) but
            // support the Electron-specific `AXManualAccessibility` attribute to
            // enable their accessibility tree. Fall back to it. No restore value
            // is preserved because restoration targets `AXEnhancedUserInterface`.
            match set_boolean_attribute(app, "AXManualAccessibility", enabled) {
                Ok(()) => {
                    eprintln!("Activated accessibility via AXManualAccessibility fallback");
                    Ok(None)
                }
                Err(_) => Err(error),
            }
        }
    }
}

/// Restores a boolean AX attribute if a previous value was captured.
fn restore_boolean_attribute(element: &AXUIElement, name: &str, restore_value: Option<bool>) {
    let Some(restore_value) = restore_value else {
        return;
    };

    if let Err(error) = set_boolean_attribute(element, name, restore_value) {
        eprintln!(
            "Unable to restore {name} to {restore_value}: {}",
            error_string(error)
        );
    }
}

/// Sets a boolean AX attribute and returns the previous value when readable.
fn set_boolean_attribute_preserving_previous(
    element: &AXUIElement,
    name: &str,
    value: bool,
) -> Result<Option<bool>, i32> {
    let previous = boolean_attribute_value(element, name).ok();
    set_boolean_attribute(element, name, value)?;

    Ok(previous)
}

/// Reads a boolean AX attribute from an accessibility element.
fn boolean_attribute_value(element: &AXUIElement, name: &str) -> Result<bool, String> {
    let attribute = CFString::new(name);
    let mut value = ptr::null();
    let error = unsafe {
        AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            attribute.as_concrete_TypeRef(),
            &mut value,
        )
    };

    if error != kAXErrorSuccess {
        return Err(format!("error:{}", error_string(error)));
    }

    if value.is_null() {
        return Err("null".to_string());
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };
    if !value.instance_of::<CFBoolean>() {
        return Err("non_boolean".to_string());
    }

    let value = unsafe { CFBoolean::wrap_under_get_rule(value.as_CFTypeRef() as _) };
    Ok(bool::from(value))
}

/// Writes a boolean AX attribute and returns the raw AX error on failure.
fn set_boolean_attribute(element: &AXUIElement, name: &str, value: bool) -> Result<(), i32> {
    let attribute = CFString::new(name);
    let value = if value {
        CFBoolean::true_value()
    } else {
        CFBoolean::false_value()
    };
    let error = unsafe {
        AXUIElementSetAttributeValue(
            element.as_concrete_TypeRef(),
            attribute.as_concrete_TypeRef(),
            value.as_CFTypeRef(),
        )
    };

    if error == kAXErrorSuccess {
        Ok(())
    } else {
        Err(error)
    }
}

fn window_frame_changed(previous: Rect, current: Rect) -> bool {
    !nearly_equal(previous.x, current.x)
        || !nearly_equal(previous.y, current.y)
        || !nearly_equal(previous.width, current.width)
        || !nearly_equal(previous.height, current.height)
}

fn settled_window_state(pid: pid_t, frame: Rect, now: Instant) -> WindowMovementState {
    WindowMovementState {
        pid,
        frame,
        last_changed_at: now
            .checked_sub(WINDOW_MOVEMENT_SETTLE_DURATION)
            .unwrap_or(now),
    }
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= WINDOW_FRAME_TOLERANCE
}

fn frontmost_window_frame_for_pid(pid: pid_t) -> Option<Rect> {
    let window_infos = core_graphics::window::copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    let window_infos =
        unsafe { CFArray::<WindowInfo>::wrap_under_get_rule(window_infos.as_concrete_TypeRef()) };

    window_infos
        .iter()
        .filter(|window_info| window_pid(window_info) == Some(pid))
        .filter(|window_info| window_layer(window_info) == Some(0))
        .filter(|window_info| window_alpha(window_info).is_some_and(|alpha| alpha > 0.0))
        .find_map(|window_info| window_bounds(&window_info))
}

fn window_pid(window_info: &WindowInfo) -> Option<pid_t> {
    dictionary_i64(window_info, unsafe { kCGWindowOwnerPID }).map(|value| value as pid_t)
}

fn window_layer(window_info: &WindowInfo) -> Option<i64> {
    dictionary_i64(window_info, unsafe { kCGWindowLayer })
}

fn window_alpha(window_info: &WindowInfo) -> Option<f64> {
    dictionary_f64(window_info, unsafe { kCGWindowAlpha })
}

fn window_bounds(window_info: &WindowInfo) -> Option<Rect> {
    let bounds = dictionary_dictionary(window_info, unsafe { kCGWindowBounds })?;

    Some(Rect {
        x: dictionary_f64(
            &bounds,
            CFString::from_static_string("X").as_concrete_TypeRef(),
        )?,
        y: dictionary_f64(
            &bounds,
            CFString::from_static_string("Y").as_concrete_TypeRef(),
        )?,
        width: dictionary_f64(
            &bounds,
            CFString::from_static_string("Width").as_concrete_TypeRef(),
        )?,
        height: dictionary_f64(
            &bounds,
            CFString::from_static_string("Height").as_concrete_TypeRef(),
        )?,
    })
}

fn dictionary_i64(dictionary: &WindowInfo, key: CFStringRef) -> Option<i64> {
    dictionary_number(dictionary, key)?.to_i64()
}

fn dictionary_f64(dictionary: &WindowInfo, key: CFStringRef) -> Option<f64> {
    dictionary_number(dictionary, key)?.to_f64()
}

fn dictionary_number(dictionary: &WindowInfo, key: CFStringRef) -> Option<CFNumber> {
    let value = dictionary_value(dictionary, key)?;

    Some(unsafe { CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as _) })
}

fn dictionary_dictionary(dictionary: &WindowInfo, key: CFStringRef) -> Option<WindowInfo> {
    let value = dictionary_value(dictionary, key)?;

    Some(unsafe { WindowInfo::wrap_under_get_rule(value.as_CFTypeRef() as _) })
}

fn dictionary_value(dictionary: &WindowInfo, key: CFStringRef) -> Option<CFType> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };

    dictionary.find(&key).map(|value| value.clone())
}

fn ax_element_attribute(
    element: &AXUIElement,
    name: &str,
) -> Result<AXUIElement, Box<dyn StdError>> {
    let attr = CFString::new(name);
    let mut value = ptr::null();

    let err = unsafe {
        AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            attr.as_concrete_TypeRef(),
            &mut value,
        )
    };

    if err != kAXErrorSuccess {
        return Err(format!(
            "AXUIElementCopyAttributeValue failed: {}",
            error_string(err)
        )
        .into());
    }

    if value.is_null() {
        return Err(format!("AXUIElementCopyAttributeValue returned null for {name}").into());
    }

    Ok(unsafe { AXUIElement::wrap_under_create_rule(value as _) })
}

/// Outcome of asking macOS for the bounds of a text range.
#[derive(Debug, Clone, Copy)]
enum TextRangeBoundsProbe {
    Success(Rect),
    Unavailable,
    InvalidRangeValue,
    AxError(i32),
}

impl TextRangeBoundsProbe {
    /// Returns true only when the probe produced non-zero text geometry.
    fn has_usable_text_metrics(self) -> bool {
        matches!(self, Self::Success(_))
    }
}

/// Converts a Core Foundation value to a Rust string when it is a `CFString`.
fn cf_type_to_string(value: &CFType) -> Option<String> {
    value
        .instance_of::<CFString>()
        .then(|| unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _) }.to_string())
}

/// Probes `AXBoundsForRange` for a specific text range on an element.
fn probe_element_rect_for_text_range(
    element: &AXUIElement,
    start_index: isize,
    length: isize,
) -> TextRangeBoundsProbe {
    let range = CFRange {
        location: start_index,
        length,
    };

    let range_value_ref = unsafe {
        AXValueCreate(
            kAXValueTypeCFRange,
            &range as *const CFRange as *const c_void,
        )
    };

    if range_value_ref.is_null() {
        return TextRangeBoundsProbe::InvalidRangeValue;
    }

    let range_value = unsafe { CFType::wrap_under_create_rule(range_value_ref as _) };
    let attr = CFString::new(kAXBoundsForRangeParameterizedAttribute);
    let mut value = ptr::null();

    let error = unsafe {
        AXUIElementCopyParameterizedAttributeValue(
            element.as_concrete_TypeRef(),
            attr.as_concrete_TypeRef(),
            range_value.as_CFTypeRef(),
            &mut value,
        )
    };

    if error == kAXErrorNoValue || error == kAXErrorParameterizedAttributeUnsupported {
        return TextRangeBoundsProbe::Unavailable;
    }

    if error != kAXErrorSuccess {
        return TextRangeBoundsProbe::AxError(error);
    }

    if value.is_null() {
        return TextRangeBoundsProbe::Unavailable;
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let value_type = unsafe { AXValueGetType(ax_value) };

    if value_type != kAXValueTypeCGRect {
        return TextRangeBoundsProbe::Unavailable;
    }

    let mut rect = MaybeUninit::<NSRect>::uninit();

    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGRect,
            rect.as_mut_ptr() as *mut c_void,
        )
    };

    if !ok {
        return TextRangeBoundsProbe::Unavailable;
    }

    let rect = unsafe { rect.assume_init() };

    let rect = Rect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    };

    if rect_has_usable_text_metrics(rect) {
        TextRangeBoundsProbe::Success(rect)
    } else {
        TextRangeBoundsProbe::Unavailable
    }
}

/// Returns whether a text rectangle has enough geometry to render a highlight.
fn rect_has_usable_text_metrics(rect: Rect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}

/// Walks an app's accessibility tree to determine whether activation is ready.
fn verify_accessibility_activation(app: &AXUIElement) -> AccessibilityActivationVerification {
    let walker = TreeWalker::new();
    let probe = AccessibilityActivationProbe::new();

    walker.walk(app, &probe);

    probe.result()
}

/// Tree visitor used to verify that the activated app exposes usable text bounds.
struct AccessibilityActivationProbe {
    found_supported_text_element: Cell<bool>,
    found_text_range_bounds: Cell<bool>,
}

impl AccessibilityActivationProbe {
    /// Creates an empty activation probe for one accessibility-tree walk.
    fn new() -> Self {
        Self {
            found_supported_text_element: Cell::new(false),
            found_text_range_bounds: Cell::new(false),
        }
    }

    /// Summarizes what the probe found during the tree walk.
    fn result(&self) -> AccessibilityActivationVerification {
        if self.found_text_range_bounds.get() {
            AccessibilityActivationVerification::FoundTextRangeBounds
        } else if self.found_supported_text_element.get() {
            AccessibilityActivationVerification::FoundSupportedTextElement
        } else {
            AccessibilityActivationVerification::NoSupportedTextElement
        }
    }
}

impl TreeVisitor for AccessibilityActivationProbe {
    /// Checks each AX element for supported text and usable range bounds.
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_supported_text_element(element)
        {
            self.found_supported_text_element.set(true);

            let string = cf_type_to_string(&value).unwrap_or_default();
            let bounds_usable = if string.is_empty() {
                false
            } else {
                probe_element_rect_for_text_range(element, 0, 1).has_usable_text_metrics()
                    || element_rect_for_text_range_with_fallback(element, &string, 0, 1).is_some()
            };

            if bounds_usable {
                self.found_text_range_bounds.set(true);
                return TreeWalkerFlow::Exit;
            }
        }

        TreeWalkerFlow::Continue
    }

    fn exit_element(&self, _element: &AXUIElement) {}
}

struct RectCollector<'a> {
    rects: RefCell<Vec<ActionableLint>>,
    lint_text: RefCell<&'a mut LintCallback<'a>>,
}

impl TreeVisitor for RectCollector<'_> {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_supported_text_element(element)
        {
            let string =
                unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _).to_string() };

            let mut rects = self.rects.borrow_mut();
            let organized_lints = (self.lint_text.borrow_mut())(&string);

            for (rule_name, lints) in organized_lints {
                for lint in lints {
                    if let Some(rect) = element_rect_for_text_range_with_fallback(
                        element,
                        &string,
                        lint.span.start,
                        lint.span.len(),
                    ) {
                        let element = element.clone();
                        let source_text = string.clone();
                        let suggestion_source_text = string.clone();
                        let suggestion_lint = lint.clone();

                        rects.push(ActionableLint::new(
                            rect,
                            rule_name.clone(),
                            lint,
                            source_text,
                            move |suggestion| {
                                apply_suggestion_to_element(
                                    element,
                                    suggestion_source_text,
                                    suggestion_lint,
                                    suggestion,
                                );
                            },
                        ));
                    }
                }
            }
        }

        TreeWalkerFlow::Continue
    }
    fn exit_element(&self, _element: &AXUIElement) {}
}

impl<'a> RectCollector<'a> {
    fn new(lint_text: &'a mut LintCallback) -> Self {
        Self {
            rects: RefCell::new(Vec::new()),
            lint_text: RefCell::new(lint_text),
        }
    }

    fn unwrap_rects(self) -> Vec<ActionableLint> {
        self.rects.into_inner()
    }
}

fn apply_suggestion_to_element(
    element: AXUIElement,
    source_text: String,
    lint: Lint,
    suggestion: Suggestion,
) {
    let mut chars = source_text.chars().collect::<Vec<_>>();
    suggestion.apply(lint.span, &mut chars);
    let updated = chars.into_iter().collect::<String>();
    let value = CFString::new(&updated);

    if let Err(error) = element.set_value(value.as_CFType()) {
        eprintln!("Unable to apply suggestion: {error}");
    }
}

/// Returns whether an accessibility element has a text role Harper can lint.
fn is_supported_text_element(el: &AXUIElement) -> bool {
    if let Ok(role) = el.role() {
        return is_supported_text_role(&role.to_string());
    }

    false
}

/// Returns whether an AX role represents editable text supported by the highlighter.
fn is_supported_text_role(role: &str) -> bool {
    matches!(role, "AXTextArea" | "AXTextField")
}

/// Collects `AXStaticText` descendants with their string values.
///
/// Chromium-based apps (e.g. Slack) report degenerate `AXBoundsForRange`
/// rects on the editable `AXTextArea` wrapper but expose usable bounds on the
/// `AXStaticText` leaf nodes inside it.
struct StaticTextCollector {
    texts: RefCell<Vec<(AXUIElement, String)>>,
}

impl StaticTextCollector {
    fn new() -> Self {
        Self {
            texts: RefCell::new(Vec::new()),
        }
    }

    fn into_texts(self) -> Vec<(AXUIElement, String)> {
        self.texts.into_inner()
    }
}

impl TreeVisitor for StaticTextCollector {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(role) = element.role()
            && role == "AXStaticText"
            && let Ok(value) = element.value()
            && let Some(string) = cf_type_to_string(&value)
            && !string.is_empty()
        {
            self.texts.borrow_mut().push((element.clone(), string));
        }

        TreeWalkerFlow::Continue
    }

    fn exit_element(&self, _element: &AXUIElement) {}
}

/// Finds `needle` within `haystack` starting at char offset `from`.
fn find_char_subslice(haystack: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    let end = haystack.len() - needle.len();
    (from..=end).find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// Resolves text-range bounds, falling back to `AXStaticText` descendants
/// when the element itself reports unusable bounds (Chromium-based apps).
fn element_rect_for_text_range_with_fallback(
    element: &AXUIElement,
    full_text: &str,
    start_index: usize,
    length: usize,
) -> Option<Rect> {
    if let Ok(Some(rect)) =
        element_rect_for_text_range(element, start_index as isize, length as isize)
    {
        return Some(rect);
    }

    let walker = TreeWalker::new();
    let collector = StaticTextCollector::new();
    walker.walk(element, &collector);

    let full_chars: Vec<char> = full_text.chars().collect();
    let mut cursor = 0usize;

    for (child, child_text) in collector.into_texts() {
        let child_chars: Vec<char> = child_text.chars().collect();
        // Locate this child's text within the parent value, starting at the
        // cursor, to absorb any separators the parent inserts between children.
        let Some(child_start) = find_char_subslice(&full_chars, &child_chars, cursor) else {
            continue;
        };
        let child_end = child_start + child_chars.len();

        if start_index >= child_start
            && start_index + length <= child_end
            && let Ok(Some(rect)) = element_rect_for_text_range(
                &child,
                (start_index - child_start) as isize,
                length as isize,
            )
        {
            return Some(rect);
        }

        cursor = child_end;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_text_roles_include_text_area_and_text_field() {
        assert!(is_supported_text_role("AXTextArea"));
        assert!(is_supported_text_role("AXTextField"));
    }

    #[test]
    fn supported_text_roles_reject_unrelated_roles() {
        assert!(!is_supported_text_role("AXButton"));
        assert!(!is_supported_text_role("AXStaticText"));
        assert!(!is_supported_text_role(""));
    }

    #[test]
    fn finds_char_subslice_respecting_offset() {
        let haystack: Vec<char> = "one\ntwo two".chars().collect();
        let needle: Vec<char> = "two".chars().collect();

        assert_eq!(find_char_subslice(&haystack, &needle, 0), Some(4));
        assert_eq!(find_char_subslice(&haystack, &needle, 5), Some(8));
        assert_eq!(find_char_subslice(&haystack, &needle, 9), None);
        assert_eq!(find_char_subslice(&haystack, &[], 0), None);
        assert_eq!(find_char_subslice(&[], &needle, 0), None);
    }

    #[test]
    fn chromium_activation_uses_chromium_settle_duration() {
        assert_eq!(
            CHROMIUM_ACCESSIBILITY_SETTLE_DURATION,
            Duration::from_secs(3)
        );
    }

    #[test]
    fn verification_retry_slows_after_fast_attempts() {
        assert_eq!(
            accessibility_activation_verification_retry_interval(
                ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS
            ),
            ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL
        );
        assert_eq!(
            accessibility_activation_verification_retry_interval(
                ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS + 1
            ),
            ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL
        );
    }

    #[test]
    fn text_range_bounds_probe_requires_non_zero_geometry() {
        let usable = TextRangeBoundsProbe::Success(Rect {
            x: 10.0,
            y: 20.0,
            width: 1.0,
            height: 12.0,
        });
        let unavailable = TextRangeBoundsProbe::Unavailable;

        assert!(usable.has_usable_text_metrics());
        assert!(!unavailable.has_usable_text_metrics());
        assert!(rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }));
        assert!(!rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 1.0,
        }));
    }
}

fn element_rect_for_text_range(
    element: &AXUIElement,
    start_index: isize,
    length: isize,
) -> Result<Option<Rect>, Error> {
    match probe_element_rect_for_text_range(element, start_index, length) {
        TextRangeBoundsProbe::Success(rect) => Ok(Some(rect)),
        TextRangeBoundsProbe::InvalidRangeValue => Err(Error::Ax(kAXErrorIllegalArgument)),
        TextRangeBoundsProbe::AxError(error) => Err(Error::Ax(error)),
        TextRangeBoundsProbe::Unavailable => Ok(None),
    }
}
