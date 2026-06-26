use accessibility::ui_element::AXUIElement;
use accessibility_sys::{
    AXUIElementGetPid, error_string, kAXErrorSuccess, kAXFocusedApplicationAttribute, pid_t,
};
use core_foundation::base::TCFType;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use std::error::Error as StdError;

use super::core_foundation_utilities::ax_element_attribute;

pub fn focused_window_pid() -> Result<pid_t, Box<dyn StdError>> {
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
