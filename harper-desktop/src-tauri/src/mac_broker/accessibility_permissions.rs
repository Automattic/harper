use accessibility_sys::{
    AXIsProcessTrusted, AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt,
};
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;

use crate::os_broker::AccessibilityPermissionStatus;

/// Check is the current process has permission to access the accessibility API.
pub fn accessibility_permission_status() -> AccessibilityPermissionStatus {
    if unsafe { AXIsProcessTrusted() } {
        AccessibilityPermissionStatus::Granted
    } else {
        AccessibilityPermissionStatus::NotGranted
    }
}

/// Request access to the accessibility API. Returns if it was granted.
pub fn request_accessibility_permission() -> AccessibilityPermissionStatus {
    let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let prompt_value = CFBoolean::true_value();
    let options: CFDictionary<CFString, CFBoolean> =
        CFDictionary::from_CFType_pairs(&[(prompt_key, prompt_value)]);

    accessibility_permission_status()
}
