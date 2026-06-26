use accessibility_sys::pid_t;
use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::window::{
    kCGNullWindowID, kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer,
    kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
};
use std::time::{Duration, Instant};

use crate::rect::Rect;

use super::core_foundation_utilities::{
    WindowInfo, dictionary_dictionary, dictionary_f64, dictionary_i64,
};

pub(super) const WINDOW_MOVEMENT_SETTLE_DURATION: Duration = Duration::from_millis(150);
const WINDOW_FRAME_TOLERANCE: f64 = 0.5;

pub(super) struct WindowMovementState {
    pub(super) pid: pid_t,
    pub(super) frame: Rect,
    pub(super) last_changed_at: Instant,
}

pub(super) fn window_frame_changed(previous: Rect, current: Rect) -> bool {
    !nearly_equal(previous.x, current.x)
        || !nearly_equal(previous.y, current.y)
        || !nearly_equal(previous.width, current.width)
        || !nearly_equal(previous.height, current.height)
}

pub(super) fn settled_window_state(pid: pid_t, frame: Rect, now: Instant) -> WindowMovementState {
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

pub(super) fn frontmost_window_frame_for_pid(pid: pid_t) -> Option<Rect> {
    let window_infos = core_graphics::window::copy_window_info(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    let window_infos =
        unsafe { CFArray::<WindowInfo>::wrap_under_get_rule(window_infos.as_concrete_TypeRef()) };

    window_infos
        .iter()
        .filter(|window_info| window_pid(window_info) == Some(pid))
        .filter(|window_info| window_layer(window_info) == Some(0))
        .filter(|window_info| window_alpha(window_info).is_some_and(|alpha| alpha > 0.0))
        .find_map(|window_info| window_bounds(&window_info))
}

fn window_pid(window_info: &WindowInfo) -> Option<pid_t> {
    dictionary_i64(window_info, unsafe { kCGWindowOwnerPID }).map(|value| value as pid_t)
}

fn window_layer(window_info: &WindowInfo) -> Option<i64> {
    dictionary_i64(window_info, unsafe { kCGWindowLayer })
}

fn window_alpha(window_info: &WindowInfo) -> Option<f64> {
    dictionary_f64(window_info, unsafe { kCGWindowAlpha })
}

fn window_bounds(window_info: &WindowInfo) -> Option<Rect> {
    let bounds = dictionary_dictionary(window_info, unsafe { kCGWindowBounds })?;

    Some(Rect {
        x: dictionary_f64(
            &bounds,
            CFString::from_static_string("X").as_concrete_TypeRef(),
        )?,
        y: dictionary_f64(
            &bounds,
            CFString::from_static_string("Y").as_concrete_TypeRef(),
        )?,
        width: dictionary_f64(
            &bounds,
            CFString::from_static_string("Width").as_concrete_TypeRef(),
        )?,
        height: dictionary_f64(
            &bounds,
            CFString::from_static_string("Height").as_concrete_TypeRef(),
        )?,
    })
}
