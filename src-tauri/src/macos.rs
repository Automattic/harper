use accessibility::attribute::{AXAttribute, AXUIElementAttributes};
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXUIElementCopyAttributeValue, AXUIElementCopyParameterizedAttributeValue, AXUIElementGetPid,
    AXValueCreate, AXValueGetType, AXValueGetValue, AXValueRef, error_string,
    kAXBoundsForRangeParameterizedAttribute, kAXErrorIllegalArgument, kAXErrorNoValue,
    kAXErrorParameterizedAttributeUnsupported, kAXErrorSuccess, kAXFocusedApplicationAttribute,
    kAXPositionAttribute, kAXSizeAttribute, kAXValueTypeCFRange, kAXValueTypeCGPoint,
    kAXValueTypeCGRect, kAXValueTypeCGSize, pid_t,
};
use core::{ffi::c_void, mem::MaybeUninit};
use core_foundation::base::{CFRange, CFType, TCFType};
use core_foundation::string::CFString;
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use harper_core::{
    Dialect, Document,
    linting::{LintGroup, Linter},
    spell::FstDictionary,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use std::{cell::RefCell, error::Error as StdError, ptr};

use crate::rect::{PositionedLint, Rect};

pub fn cursor_position() -> Option<egui::Pos2> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    let event = CGEvent::new(source).ok()?;
    let location = event.location();

    Some(egui::pos2(location.x as f32, location.y as f32))
}

pub fn get_boxes() -> Vec<PositionedLint> {
    let pid = match focused_window_pid() {
        Ok(pid) => pid,
        Err(err) => {
            println!("Unable to identify focused window: {err}");
            return Vec::new();
        }
    };

    let el = AXUIElement::application(pid);

    let walker = TreeWalker::new();
    let collector = RectCollector::new();

    walker.walk(&el, &collector);

    collector.unwrap_rects()
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

struct RectCollector {
    rects: RefCell<Vec<PositionedLint>>,
    linter: RefCell<LintGroup>,
}

impl TreeVisitor for RectCollector {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_textarea(element)
        {
            let string =
                unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _).to_string() };

            let mut linter = self.linter.borrow_mut();
            let mut rects = self.rects.borrow_mut();

            let doc = Document::new_markdown_default_curated(&string);
            let lints = linter.lint(&doc);

            for lint in &lints {
                if let Ok(Some(rect)) = element_rect_for_text_range(
                    &element,
                    lint.span.start as isize,
                    lint.span.len() as isize,
                ) {
                    rects.push(PositionedLint::new(rect, lint.clone()));
                }
            }
        }

        TreeWalkerFlow::Continue
    }
    fn exit_element(&self, element: &AXUIElement) {}
}

impl RectCollector {
    pub fn new() -> Self {
        Self {
            rects: RefCell::new(Vec::new()),
            linter: RefCell::new(LintGroup::new_curated(
                FstDictionary::curated(),
                Dialect::American,
            )),
        }
    }

    pub fn unwrap_rects(self) -> Vec<PositionedLint> {
        self.rects.into_inner()
    }
}

impl Default for RectCollector {
    fn default() -> Self {
        Self::new()
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

fn ax_value<T>(element: &AXUIElement, name: &str, value_type: u32) -> Result<Option<T>, Error> {
    let attr = AXAttribute::<CFType>::new(&CFString::new(name));
    let value = element.attribute(&attr)?;
    let mut out = MaybeUninit::<T>::uninit();

    let ok = unsafe {
        AXValueGetValue(
            value.as_CFTypeRef() as AXValueRef,
            value_type,
            out.as_mut_ptr() as *mut c_void,
        )
    };

    Ok(ok.then(|| unsafe { out.assume_init() }))
}

pub fn element_rect(element: &AXUIElement) -> Result<Option<Rect>, Error> {
    let Some(position) = ax_value::<NSPoint>(element, kAXPositionAttribute, kAXValueTypeCGPoint)?
    else {
        return Ok(None);
    };

    let Some(size) = ax_value::<NSSize>(element, kAXSizeAttribute, kAXValueTypeCGSize)? else {
        return Ok(None);
    };

    Ok(Some(Rect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    }))
}

pub fn element_rect_for_text_range(
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

    match error {
        kAXErrorSuccess => {}
        kAXErrorNoValue | kAXErrorParameterizedAttributeUnsupported => return Ok(None),
        error => return Err(Error::Ax(error)),
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
