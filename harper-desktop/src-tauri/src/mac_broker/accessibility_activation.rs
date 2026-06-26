use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXUIElementCopyAttributeValue, AXUIElementSetAttributeValue, error_string,
    kAXErrorAttributeUnsupported, kAXErrorNoValue, kAXErrorNotImplemented, kAXErrorSuccess, pid_t,
};
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::string::CFString;
use std::{
    cell::Cell,
    ptr,
    time::{Duration, Instant},
};

use super::accessibility_text::{
    cf_type_to_string, element_rect_for_text_range_with_fallback, is_supported_text_element,
    probe_element_rect_for_text_range,
};

pub(super) const ACCESSIBILITY_ACTIVATION_RETRY_INTERVAL: Duration = Duration::from_secs(10);
pub(super) const ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL: Duration =
    Duration::from_millis(250);
pub(super) const ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL: Duration =
    Duration::from_secs(1);
pub(super) const ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS: u8 = 20;
pub(super) const CHROMIUM_ACCESSIBILITY_SETTLE_DURATION: Duration = Duration::from_secs(3);

/// Tracks an app accessibility activation attempt and any AX value Harper should restore later.
#[derive(Debug, Clone)]
pub(super) struct AccessibilityActivationState {
    pub(super) pid: pid_t,
    pub(super) bundle_id: String,
    pub(super) status: AccessibilityActivationStatus,
    pub(super) last_attempted_at: Instant,
    pub(super) enhanced_user_interface_restore_value: Option<bool>,
}

/// State of the focused app's accessibility activation attempt.
#[derive(Debug, Clone, Copy)]
pub(super) enum AccessibilityActivationStatus {
    Pending {
        ready_at: Instant,
        verification_attempts: u8,
    },
    Ready,
    RetryLater,
}

/// Result of checking whether an activated app exposes usable text geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AccessibilityActivationVerification {
    FoundTextRangeBounds,
    FoundSupportedTextElement,
    NoSupportedTextElement,
}

pub(super) fn accessibility_activation_verification_retry_interval(
    verification_attempts: u8,
) -> Duration {
    if verification_attempts <= ACCESSIBILITY_ACTIVATION_FAST_VERIFICATION_ATTEMPTS {
        ACCESSIBILITY_ACTIVATION_VERIFICATION_RETRY_INTERVAL
    } else {
        ACCESSIBILITY_ACTIVATION_SLOW_VERIFICATION_RETRY_INTERVAL
    }
}

/// Adds a duration to an instant without panicking on overflow.
pub(super) fn instant_after(now: Instant, duration: Duration) -> Instant {
    now.checked_add(duration).unwrap_or(now)
}

/// Restores `AXEnhancedUserInterface` when a previous value was captured.
pub(super) fn release_accessibility_activation(state: &AccessibilityActivationState) {
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
pub(super) fn is_unsupported_accessibility_activation_error(error: i32) -> bool {
    error == kAXErrorAttributeUnsupported
        || error == kAXErrorNoValue
        || error == kAXErrorNotImplemented
}

/// Sets `AXEnhancedUserInterface` while returning its old value when readable.
pub(super) fn set_enhanced_user_interface_preserving_previous(
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

/// Walks an app accessibility tree to determine whether activation exposes usable text bounds.
pub(super) fn verify_accessibility_activation(
    app: &AXUIElement,
) -> AccessibilityActivationVerification {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
