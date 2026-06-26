use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt,
};
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

use crate::os_broker::AccessibilityPermissionStatus;

pub(super) fn accessibility_permission_status() -> AccessibilityPermissionStatus {
    if unsafe { AXIsProcessTrusted() } {
        AccessibilityPermissionStatus::Granted
    } else {
        AccessibilityPermissionStatus::NotGranted
    }
}

pub(super) fn request_accessibility_permission() -> AccessibilityPermissionStatus {
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
