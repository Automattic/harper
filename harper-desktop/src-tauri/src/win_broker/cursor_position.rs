use windows::{Win32::Foundation::POINT, Win32::UI::WindowsAndMessaging::GetCursorPos};

/// Returns the current global cursor position in physical screen pixels.
pub fn cursor_position() -> Option<egui::Pos2> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point).ok()? };
    Some(egui::pos2(point.x as f32, point.y as f32))
}
