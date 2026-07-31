use std::time::{Duration, Instant};
use windows::{
    Win32::Foundation::{HWND, RECT},
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
    },
};

use crate::rect::Rect;

pub const WINDOW_MOVEMENT_SETTLE_DURATION: Duration = Duration::from_millis(150);
const WINDOW_FRAME_TOLERANCE: f64 = 0.5;

pub struct WindowMovementState {
    pub pid: u32,
    pub frame: Rect,
    pub last_changed_at: Instant,
}

pub fn window_frame_changed(previous: Rect, current: Rect) -> bool {
    !nearly_equal(previous.x, current.x)
        || !nearly_equal(previous.y, current.y)
        || !nearly_equal(previous.width, current.width)
        || !nearly_equal(previous.height, current.height)
}

pub fn settled_window_state(pid: u32, frame: Rect, now: Instant) -> WindowMovementState {
    WindowMovementState {
        pid,
        frame,
        last_changed_at: now
            .checked_sub(WINDOW_MOVEMENT_SETTLE_DURATION)
            .unwrap_or(now),
    }
}

fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= WINDOW_FRAME_TOLERANCE
}

pub fn frontmost_window_frame_for_pid(pid: u32) -> Option<Rect> {
    let hwnd: HWND = unsafe { GetForegroundWindow() };

    if hwnd.is_invalid() {
        return None;
    }

    let mut owner_pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut owner_pid)) };

    if owner_pid != pid {
        return None;
    }

    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect).ok()? };

    Some(Rect {
        x: rect.left as f64,
        y: rect.top as f64,
        width: (rect.right - rect.left) as f64,
        height: (rect.bottom - rect.top) as f64,
    })
}
