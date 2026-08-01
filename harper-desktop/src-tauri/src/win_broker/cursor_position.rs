use windows::{
    Win32::Foundation::{HWND, POINT},
    Win32::UI::HiDpi::GetDpiForWindow,
    Win32::UI::WindowsAndMessaging::{GetCursorPos, WindowFromPoint},
};

/// Returns the current global cursor position in logical screen pixels.
pub fn cursor_position() -> Option<egui::Pos2> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };

    let scale_factor = match unsafe { WindowFromPoint(point) } {
        hwnd if !hwnd.0.is_null() => {
            let dpi = unsafe { GetDpiForWindow(hwnd) };
            if dpi > 0 {
                dpi as f64 / 96.0
            } else {
                1.0
            }
        }
        _ => 1.0,
    };

    Some(egui::pos2(
        (point.x as f64 / scale_factor) as f32,
        (point.y as f64 / scale_factor) as f32,
    ))
}
