use accessibility::attribute::{AXAttribute, AXUIElementAttributes};
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;
use std::cell::Cell;

use accessibility_sys::{
    AXValueGetValue, AXValueRef, kAXPositionAttribute, kAXSizeAttribute, kAXValueTypeCGPoint,
    kAXValueTypeCGSize,
};
use core::ffi::c_void;
use core::mem::MaybeUninit;
use objc2_foundation::{NSPoint, NSSize};

use crate::rect::Rect;

pub fn get_boxes() -> Vec<Rect> {
    let el = AXUIElement::application(57046);

    let walker = TreeWalker::new();
    let collector = RectCollector::new();

    walker.walk(&el, &collector);

    collector.unwrap_rects();
}

struct RectCollector {
    rects: Cell<Vec<Rect>>,
}

impl TreeVisitor for RectCollector {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_textarea(element)
        {
            dbg!(value);

            if let Ok(Some(rect)) = element_rect(&element) {
                dbg!(rect);
                self.rects.update(|i| i.push(rect));
            }
        }

        TreeWalkerFlow::Continue
    }
    fn exit_element(&self, element: &AXUIElement) {}
}

impl RectCollector {
    pub fn new() -> Self {
        Self {
            rects: Cell::new(Vec::new()),
        }
    }

    pub fn unwrap_rects(self) -> Vec<Rect> {
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
