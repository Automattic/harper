use accessibility::ui_element::AXUIElement;
use accessibility_sys::{AXUIElementCopyAttributeValue, error_string, kAXErrorSuccess};
use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::{CFString, CFStringRef};
use std::{error::Error as StdError, ptr};

pub(super) type WindowInfo = CFDictionary<CFString, CFType>;

pub(super) fn dictionary_i64(dictionary: &WindowInfo, key: CFStringRef) -> Option<i64> {
    dictionary_number(dictionary, key)?.to_i64()
}

pub(super) fn dictionary_f64(dictionary: &WindowInfo, key: CFStringRef) -> Option<f64> {
    dictionary_number(dictionary, key)?.to_f64()
}

pub(super) fn dictionary_number(dictionary: &WindowInfo, key: CFStringRef) -> Option<CFNumber> {
    let value = dictionary_value(dictionary, key)?;

    Some(unsafe { CFNumber::wrap_under_get_rule(value.as_CFTypeRef() as _) })
}

pub(super) fn dictionary_dictionary(
    dictionary: &WindowInfo,
    key: CFStringRef,
) -> Option<WindowInfo> {
    let value = dictionary_value(dictionary, key)?;

    Some(unsafe { WindowInfo::wrap_under_get_rule(value.as_CFTypeRef() as _) })
}

pub(super) fn dictionary_value(dictionary: &WindowInfo, key: CFStringRef) -> Option<CFType> {
    let key = unsafe { CFString::wrap_under_get_rule(key) };

    dictionary.find(&key).map(|value| value.clone())
}

pub(super) fn ax_element_attribute(
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
