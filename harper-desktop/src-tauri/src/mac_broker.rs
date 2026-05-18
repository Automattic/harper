use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue,
    AXUIElementCopyParameterizedAttributeValue, AXUIElementGetPid, AXValueCreate, AXValueGetType,
    AXValueGetValue, AXValueRef, error_string, kAXBoundsForRangeParameterizedAttribute,
    kAXErrorIllegalArgument, kAXErrorNoValue, kAXErrorParameterizedAttributeUnsupported,
    kAXErrorSuccess, kAXFocusedApplicationAttribute, kAXPositionAttribute, kAXSizeAttribute,
    kAXTrustedCheckOptionPrompt, kAXValueTypeCFRange, kAXValueTypeCGPoint, kAXValueTypeCGRect,
    kAXValueTypeCGSize, pid_t,
};
use core::{ffi::c_void, mem::MaybeUninit};
use core_foundation::base::{CFRange, CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::{CGPoint, CGSize};
use harper_core::linting::{Lint, Suggestion};
use objc2_app_kit::NSRunningApplication;
use objc2_foundation::NSRect;
use std::{cell::RefCell, collections::BTreeMap, error::Error as StdError, ptr, rc::Rc};

use crate::config::{Config, Integration};
use crate::os_broker::{AccessibilityPermissionStatus, OsBroker};
use crate::rect::{ActionableLint, Rect};

/// macOS implementation of the OS data the highlighter needs.
///
/// `MacBroker` owns focus memory because clicking the overlay can make the highlighter process the
/// focused application. Remembering the last non-highlighter PID lets accessibility reads continue
/// targeting the app the user was reviewing.
pub struct MacBroker {
    last_focused: Option<pid_t>,
    integrations: Rc<RefCell<Vec<Integration>>>,
    last_geometry: Option<GeometrySnapshot>,
}

#[derive(Debug, Clone)]
struct GeometrySnapshot {
    pid: pid_t,
    window_frame: Rect,
    lints: Vec<LintSnapshot>,
}

#[derive(Debug, Clone)]
struct LintSnapshot {
    rect: Rect,
    rule_name: String,
    span_start: usize,
    span_len: usize,
    source_text: String,
}

impl MacBroker {
    pub fn new(integrations: Rc<RefCell<Vec<Integration>>>) -> Self {
        Self {
            last_focused: None,
            integrations,
            last_geometry: None,
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

    fn correct_for_window_movement(
        &mut self,
        pid: pid_t,
        window_frame: Rect,
        rects: &mut [ActionableLint],
    ) {
        let lints = lint_snapshots(rects);

        if let Some(last_geometry) = &self.last_geometry
            && should_translate_geometry(last_geometry, pid, window_frame, &lints)
        {
            let dx = window_frame.x - last_geometry.window_frame.x;
            let dy = window_frame.y - last_geometry.window_frame.y;

            for (positioned_lint, last_lint) in rects.iter_mut().zip(&last_geometry.lints) {
                positioned_lint.rect = last_lint.rect.translated(dx, dy);
            }
        }

        self.last_geometry = Some(GeometrySnapshot {
            pid,
            window_frame,
            lints: lint_snapshots(rects),
        });
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
                self.last_geometry = None;
                return Vec::new();
            }
            Err(err) => {
                self.last_geometry = None;
                eprintln!("Unable to identify focused window: {err}");
                return Vec::new();
            }
        };

        if !is_pid_approved(pid, &self.integrations.borrow()) {
            self.last_geometry = None;
            return Vec::new();
        }

        let el = AXUIElement::application(pid);
        let window_frame = focused_window_frame(&el).ok();

        let walker = TreeWalker::new();
        let collector = RectCollector::new(lint_text);

        walker.walk(&el, &collector);

        let mut rects = collector.unwrap_rects();

        if let Some(window_frame) = window_frame {
            self.correct_for_window_movement(pid, window_frame, &mut rects);
        } else {
            self.last_geometry = None;
        }

        rects
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
}

fn lint_snapshots(rects: &[ActionableLint]) -> Vec<LintSnapshot> {
    rects
        .iter()
        .map(|positioned_lint| LintSnapshot {
            rect: positioned_lint.rect,
            rule_name: positioned_lint.rule_name.clone(),
            span_start: positioned_lint.lint.span.start,
            span_len: positioned_lint.lint.span.len(),
            source_text: positioned_lint.source_text.clone(),
        })
        .collect()
}

fn should_translate_geometry(
    last_geometry: &GeometrySnapshot,
    pid: pid_t,
    window_frame: Rect,
    lints: &[LintSnapshot],
) -> bool {
    last_geometry.pid == pid
        && window_frame.same_size_as(last_geometry.window_frame)
        && window_frame_has_moved(last_geometry.window_frame, window_frame)
        && same_lint_identity(&last_geometry.lints, lints)
}

fn window_frame_has_moved(previous: Rect, current: Rect) -> bool {
    (previous.x - current.x).abs() > 0.5 || (previous.y - current.y).abs() > 0.5
}

fn same_lint_identity(left: &[LintSnapshot], right: &[LintSnapshot]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.rule_name == right.rule_name
                && left.span_start == right.span_start
                && left.span_len == right.span_len
                && left.source_text == right.source_text
        })
}

fn focused_window_frame(app: &AXUIElement) -> Result<Rect, Box<dyn StdError>> {
    let window = app.focused_window()?;
    let position = ax_point_attribute(&window, kAXPositionAttribute)?;
    let size = ax_size_attribute(&window, kAXSizeAttribute)?;

    Ok(Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    })
}

fn ax_point_attribute(element: &AXUIElement, name: &str) -> Result<CGPoint, Box<dyn StdError>> {
    let value = ax_value_attribute(element, name, kAXValueTypeCGPoint)?;
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let mut point = MaybeUninit::<CGPoint>::uninit();

    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGPoint,
            point.as_mut_ptr() as *mut c_void,
        )
    };

    if !ok {
        return Err(format!("AXValueGetValue failed for {name}").into());
    }

    Ok(unsafe { point.assume_init() })
}

fn ax_size_attribute(element: &AXUIElement, name: &str) -> Result<CGSize, Box<dyn StdError>> {
    let value = ax_value_attribute(element, name, kAXValueTypeCGSize)?;
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let mut size = MaybeUninit::<CGSize>::uninit();

    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGSize,
            size.as_mut_ptr() as *mut c_void,
        )
    };

    if !ok {
        return Err(format!("AXValueGetValue failed for {name}").into());
    }

    Ok(unsafe { size.assume_init() })
}

fn ax_value_attribute(
    element: &AXUIElement,
    name: &str,
    expected_type: u32,
) -> Result<CFType, Box<dyn StdError>> {
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
            "AXUIElementCopyAttributeValue failed for {name}: {}",
            error_string(err)
        )
        .into());
    }

    if value.is_null() {
        return Err(format!("AXUIElementCopyAttributeValue returned null for {name}").into());
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };
    let ax_value = value.as_CFTypeRef() as AXValueRef;

    if unsafe { AXValueGetType(ax_value) } != expected_type {
        return Err(
            format!("AXUIElementCopyAttributeValue returned unexpected type for {name}").into(),
        );
    }

    Ok(value)
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
