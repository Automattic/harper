use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCopyParameterizedAttributeValue, AXUIElementGetPid, AXValueCreate, AXValueGetType,
    AXValueGetValue, AXValueRef, error_string, kAXBoundsForRangeParameterizedAttribute,
    kAXErrorIllegalArgument, kAXErrorNoValue, kAXErrorParameterizedAttributeUnsupported,
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
use objc2_app_kit::NSRunningApplication;
use objc2_foundation::NSRect;
use std::{
    cell::RefCell,
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
}

#[derive(Debug, Clone)]
struct WindowMovementState {
    pid: pid_t,
    frame: Rect,
    last_changed_at: Instant,
}

impl MacBroker {
    pub fn new(integrations: Rc<RefCell<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            window_movement: None,
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
}

impl Default for MacBroker {
    fn default() -> Self {
        Self::new(Rc::new(RefCell::new(Config::curated_integrations())))
    }
}

impl OsBroker for MacBroker {
    fn get_boxes(
        &mut self,
        lint_text: &mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>,
    ) -> Vec<ActionableLint> {
        let pid = match self.target_pid() {
            Ok(Some(pid)) => pid,
            Ok(None) => {
                self.window_movement = None;
                return Vec::new();
            }
            Err(err) => {
                self.window_movement = None;
                eprintln!("Unable to identify focused window: {err}");
                return Vec::new();
            }
        };

        if !is_pid_approved(pid, &self.integrations.borrow()) {
            self.window_movement = None;
            return Vec::new();
        }

        if self.window_is_moving(pid) {
            return Vec::new();
        }

        let el = AXUIElement::application(pid);

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
    let app = ax_element_attribute(&system, kAXFocusedApplicationAttribute)?;

    let mut pid: pid_t = 0;
    let err = unsafe { AXUIElementGetPid(app.as_concrete_TypeRef(), &mut pid) };

    if err != kAXErrorSuccess {
        return Err(format!("AXUIElementGetPid failed: {}", error_string(err)).into());
    }

    Ok(pid)
}

fn is_pid_approved(pid: pid_t, integrations: &[Integration]) -> bool {
    let bundle_identifier = match bundle_identifier_for_pid(pid) {
        Ok(Some(bundle_identifier)) => bundle_identifier,
        Ok(None) => return false,
        Err(error) => {
            eprintln!("Unable to identify focused app bundle: {error}");
            return false;
        }
    };

    Config::is_integration_enabled_in(integrations, &bundle_identifier)
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

struct RectCollector<'a> {
    rects: RefCell<Vec<ActionableLint>>,
    lint_text: RefCell<&'a mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>>,
}

impl TreeVisitor for RectCollector<'_> {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_textarea(element)
        {
            let string =
                unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _).to_string() };

            let mut rects = self.rects.borrow_mut();
            let organized_lints = (self.lint_text.borrow_mut())(&string);

            for (rule_name, lints) in organized_lints {
                for lint in lints {
                    if let Ok(Some(rect)) = element_rect_for_text_range(
                        element,
                        lint.span.start as isize,
                        lint.span.len() as isize,
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
    fn new(lint_text: &'a mut dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>>) -> Self {
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

fn is_textarea(el: &AXUIElement) -> bool {
    if let Ok(role) = el.role()
        && role == "AXTextArea"
    {
        return true;
    }

    false
}

fn element_rect_for_text_range(
    element: &AXUIElement,
    start_index: isize,
    length: isize,
) -> Result<Option<Rect>, Error> {
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
        return Err(Error::Ax(kAXErrorIllegalArgument));
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

    if error == kAXErrorSuccess {
        // Continue.
    } else if error == kAXErrorNoValue || error == kAXErrorParameterizedAttributeUnsupported {
        return Ok(None);
    } else {
        return Err(Error::Ax(error));
    }

    if value.is_null() {
        return Ok(None);
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };
    let ax_value = value.as_CFTypeRef() as AXValueRef;

    if unsafe { AXValueGetType(ax_value) } != kAXValueTypeCGRect {
        return Ok(None);
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
        return Ok(None);
    }

    let rect = unsafe { rect.assume_init() };

    Ok(Some(Rect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    }))
}
