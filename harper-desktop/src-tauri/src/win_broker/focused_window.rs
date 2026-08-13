use std::error::Error as StdError;
use windows::{
    Win32::Foundation::HWND,
    Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

/// Returns the process ID of the currently focused window.
///
/// Returns `None` when no window is focused (e.g., desktop).
pub fn focused_window_pid() -> Result<Option<u32>, Box<dyn StdError>> {
    let hwnd: HWND = unsafe { GetForegroundWindow() };

    if hwnd.is_invalid() {
        return Ok(None);
    }

    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };

    if pid == 0 {
        return Ok(None);
    }

    Ok(Some(pid))
}
