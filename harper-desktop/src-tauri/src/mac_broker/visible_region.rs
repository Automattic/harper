use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility_sys::{
    AXUIElementCopyAttributeValue, AXValueGetType, AXValueGetTypeID, AXValueGetValue, AXValueRef,
    AXValueType, kAXErrorSuccess, kAXPositionAttribute, kAXScrollAreaRole, kAXSizeAttribute,
    kAXValueTypeCGPoint, kAXValueTypeCGSize, kAXWindowRole,
};
use core::{ffi::c_void, mem::MaybeUninit};
use core_foundation::base::{CFType, TCFType};
use core_foundation::string::CFString;
use objc2_foundation::{NSPoint, NSSize};
use std::{iter, ptr};

use crate::rect::Rect;

/// Roles whose frame bounds the part of their descendants that is drawn on screen.
const CLIPPING_ROLES: [&str; 3] = [kAXScrollAreaRole, "AXWebArea", kAXWindowRole];

/// Guards the ancestor walk against cyclic or unexpectedly deep accessibility trees.
const MAX_ANCESTOR_DEPTH: usize = 64;

/// Finds the screen region in which a text element's contents are actually visible.
///
/// `AXBoundsForRange` reports where text is laid out, even when that text is scrolled out of view
/// or sits inside a collapsed container. The visible region is the element's own frame
/// intersected with the frames of its enclosing scroll areas, web areas, and window.
///
/// Returns `None` when no frame could be read, in which case nothing should be clipped.
pub fn element_visible_region(element: &AXUIElement) -> Option<Rect> {
    let frames = iter::once(element.clone())
        .chain(clipping_ancestors(element))
        .filter_map(|element| element_frame(&element));

    intersect_frames(frames)
}

/// Clips a text rect to the visible region, returning `None` when none of it is visible.
pub fn clip_to_visible_region(rect: Rect, visible_region: Option<Rect>) -> Option<Rect> {
    let Some(visible_region) = visible_region else {
        return Some(rect);
    };

    let clipped = intersection(rect, visible_region);
    has_area(clipped).then_some(clipped)
}

/// Yields the ancestors of `element` whose frames clip what is drawn inside them.
fn clipping_ancestors(element: &AXUIElement) -> impl Iterator<Item = AXUIElement> {
    iter::successors(element.parent().ok(), |ancestor| ancestor.parent().ok())
        .take(MAX_ANCESTOR_DEPTH)
        .filter(|ancestor| {
            ancestor
                .role()
                .is_ok_and(|role| CLIPPING_ROLES.contains(&role.to_string().as_str()))
        })
}

/// Reads an element's screen frame.
///
/// Frames without area are treated as unreadable, since they carry no usable clipping
/// information and would otherwise hide every highlight.
fn element_frame(element: &AXUIElement) -> Option<Rect> {
    let position: NSPoint = ax_value_attribute(element, kAXPositionAttribute, kAXValueTypeCGPoint)?;
    let size: NSSize = ax_value_attribute(element, kAXSizeAttribute, kAXValueTypeCGSize)?;
    let frame = Rect::new(position.x, position.y, size.width, size.height);

    has_area(frame).then_some(frame)
}

/// Reads an `AXValue`-wrapped attribute, such as a point or size, from an element.
///
/// `T` must be the Core Graphics type that matches `value_type`.
fn ax_value_attribute<T>(element: &AXUIElement, name: &str, value_type: AXValueType) -> Option<T> {
    let attribute = CFString::new(name);
    let mut value = ptr::null();

    let error = unsafe {
        AXUIElementCopyAttributeValue(
            element.as_concrete_TypeRef(),
            attribute.as_concrete_TypeRef(),
            &mut value,
        )
    };

    if error != kAXErrorSuccess || value.is_null() {
        return None;
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };

    if value.type_of() != unsafe { AXValueGetTypeID() } {
        return None;
    }

    let ax_value = value.as_CFTypeRef() as AXValueRef;

    if unsafe { AXValueGetType(ax_value) } != value_type {
        return None;
    }

    let mut result = MaybeUninit::<T>::uninit();
    let ok = unsafe { AXValueGetValue(ax_value, value_type, result.as_mut_ptr() as *mut c_void) };

    ok.then(|| unsafe { result.assume_init() })
}

/// Intersects every frame, returning `None` when there are no frames.
fn intersect_frames(frames: impl IntoIterator<Item = Rect>) -> Option<Rect> {
    frames.into_iter().reduce(intersection)
}

/// Returns the area covered by both rects, with zero width or height when they do not overlap.
fn intersection(a: Rect, b: Rect) -> Rect {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);

    Rect::new(x, y, (right - x).max(0.0), (bottom - y).max(0.0))
}

fn has_area(rect: Rect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_keeps_overlapping_area() {
        let a = Rect::new(0.0, 0.0, 100.0, 50.0);
        let b = Rect::new(40.0, 20.0, 100.0, 100.0);

        assert_eq!(intersection(a, b), Rect::new(40.0, 20.0, 60.0, 30.0));
        assert_eq!(intersection(b, a), intersection(a, b));
    }

    #[test]
    fn intersection_of_disjoint_rects_has_no_area() {
        let above = Rect::new(0.0, 0.0, 100.0, 50.0);
        let below = Rect::new(0.0, 80.0, 100.0, 50.0);

        assert!(!has_area(intersection(above, below)));
    }

    #[test]
    fn intersection_of_touching_rects_has_no_area() {
        let left = Rect::new(0.0, 0.0, 50.0, 50.0);
        let right = Rect::new(50.0, 0.0, 50.0, 50.0);

        assert!(!has_area(intersection(left, right)));
    }

    #[test]
    fn intersect_frames_narrows_to_innermost_visible_area() {
        let text_area = Rect::new(100.0, 0.0, 400.0, 2000.0);
        let scroll_area = Rect::new(80.0, 100.0, 600.0, 300.0);
        let window = Rect::new(0.0, 50.0, 1000.0, 800.0);

        assert_eq!(
            intersect_frames([text_area, scroll_area, window]),
            Some(Rect::new(100.0, 100.0, 400.0, 300.0))
        );
    }

    #[test]
    fn intersect_frames_without_frames_is_unknown() {
        assert_eq!(intersect_frames([]), None);
    }

    #[test]
    fn unknown_region_does_not_clip() {
        let rect = Rect::new(10.0, 5000.0, 40.0, 16.0);

        assert_eq!(clip_to_visible_region(rect, None), Some(rect));
    }

    #[test]
    fn fully_visible_rect_is_unchanged() {
        let visible_region = Rect::new(0.0, 0.0, 800.0, 600.0);
        let rect = Rect::new(10.0, 20.0, 40.0, 16.0);

        assert_eq!(
            clip_to_visible_region(rect, Some(visible_region)),
            Some(rect)
        );
    }

    #[test]
    fn rect_scrolled_out_of_view_is_hidden() {
        let visible_region = Rect::new(0.0, 0.0, 800.0, 600.0);
        let below_fold = Rect::new(10.0, 900.0, 40.0, 16.0);
        let above_fold = Rect::new(10.0, -200.0, 40.0, 16.0);

        assert_eq!(
            clip_to_visible_region(below_fold, Some(visible_region)),
            None
        );
        assert_eq!(
            clip_to_visible_region(above_fold, Some(visible_region)),
            None
        );
    }

    #[test]
    fn rect_on_visible_edge_is_trimmed() {
        let visible_region = Rect::new(0.0, 0.0, 800.0, 600.0);
        let straddling = Rect::new(10.0, 592.0, 40.0, 16.0);

        assert_eq!(
            clip_to_visible_region(straddling, Some(visible_region)),
            Some(Rect::new(10.0, 592.0, 40.0, 8.0))
        );
    }

    #[test]
    fn empty_visible_region_hides_everything() {
        let collapsed = Rect::new(0.0, 300.0, 800.0, 0.0);
        let rect = Rect::new(10.0, 300.0, 40.0, 16.0);

        assert_eq!(clip_to_visible_region(rect, Some(collapsed)), None);
    }
}
