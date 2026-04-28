use accessibility::attribute::{AXAttribute, AXUIElementAttributes};
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;

use accessibility_sys::{
    AXValueGetValue, AXValueRef, kAXPositionAttribute, kAXSizeAttribute, kAXValueTypeCGPoint,
    kAXValueTypeCGSize,
};
use cocoa_foundation::{NSPoint, NSSize};
use core::ffi::c_void;
use core::mem::MaybeUninit;

pub fn main() {
    let el = AXUIElement::application(57046);

    let walker = TreeWalker::new();
    walker.walk(&el, &Printing);

    struct Printing;
    impl TreeVisitor for Printing {
        fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
            if let Ok(value) = element.value()
                && is_textarea(element)
            {
                dbg!(value);

                dbg!(element_rect(&element));
            }

            TreeWalkerFlow::Continue
        }
        fn exit_element(&self, element: &AXUIElement) {}
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

#[derive(Debug, Clone, Copy)]
pub struct AxRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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

pub fn element_rect(element: &AXUIElement) -> Result<Option<AxRect>, Error> {
    let Some(position) = ax_value::<NSPoint>(element, kAXPositionAttribute, kAXValueTypeCGPoint)?
    else {
        return Ok(None);
    };

    let Some(size) = ax_value::<NSSize>(element, kAXSizeAttribute, kAXValueTypeCGSize)? else {
        return Ok(None);
    };

    Ok(Some(AxRect {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
    }))
}
