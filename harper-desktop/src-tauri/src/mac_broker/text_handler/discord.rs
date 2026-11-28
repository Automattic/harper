use super::super::accessibility_text::cf_type_to_string;
use super::{DefaultTextHandler, TextHandler, TextResult, TextTarget};
use accessibility::{attribute::AXUIElementAttributes, ui_element::AXUIElement};
use accessibility_sys::{
    AXUIElementCopyParameterizedAttributeValue, AXValueCreate, AXValueGetValue, AXValueRef,
    kAXErrorSuccess, kAXValueTypeCFRange,
};
use core_foundation::{
    base::{CFRange, CFType, TCFType},
    string::CFString,
};
use harper_core::{Span, linting::Suggestion};
use std::ptr;
/// Reads Discord's AX text but commits edits through native input to update its draft model.
pub struct DiscordTextHandler;
impl TextHandler for DiscordTextHandler {
    fn read_text(&self, target: &TextTarget) -> TextResult<String> {
        DefaultTextHandler.read_text(target)
    }
    fn write_text(
        &self,
        target: &TextTarget,
        snapshot: &str,
        span: Span<char>,
        suggestion: &Suggestion,
    ) -> TextResult<()> {
        apply_discord_edit(
            target.pid,
            &target.element,
            snapshot,
            span.start..span.end,
            suggestion,
        )
    }
}
/// Maps Harper's character span to the UTF-16 selection and text a native editor needs.
/// Rejects stale snapshots rather than editing whatever now occupies the same offsets.
fn discord_edit(
    snapshot: &str,
    current: &str,
    span: std::ops::Range<usize>,
    suggestion: &Suggestion,
) -> Option<(CFRange, String)> {
    if snapshot != current || span.start > span.end || span.end > snapshot.chars().count() {
        return None;
    }
    let (start, replacement) = match suggestion {
        Suggestion::InsertAfter(text) => (span.end, text.iter().collect()),
        Suggestion::Remove => (span.start, String::new()),
        Suggestion::ReplaceWith(text) => (span.start, text.iter().collect()),
    };
    let units = |n| snapshot.chars().take(n).map(char::len_utf16).sum::<usize>() as isize;
    let location = units(start);
    Some((
        CFRange {
            location,
            length: units(span.end) - location,
        },
        replacement,
    ))
}

/// Discord may expose layout newlines around inline emoji in AXValue but omit them
/// from selectable text. Confirm the entire prefix against AXStringForRange before
/// using either coordinate system; do not guess when neither representation matches.
fn discord_prefix_offset(element: &AXUIElement, prefix: &str) -> Option<isize> {
    for candidate in [prefix.to_owned(), prefix.replace('\n', "")] {
        let length = candidate.encode_utf16().count() as isize;
        if length == 0 {
            return Some(0);
        }
        let range = CFRange {
            location: 0,
            length,
        };
        let raw = unsafe { AXValueCreate(kAXValueTypeCFRange, (&range as *const CFRange).cast()) };
        if raw.is_null() {
            return None;
        }
        let parameter = unsafe { CFType::wrap_under_create_rule(raw.cast()) };
        let mut result = ptr::null();
        let status = unsafe {
            AXUIElementCopyParameterizedAttributeValue(
                element.as_concrete_TypeRef(),
                CFString::new(accessibility_sys::kAXStringForRangeParameterizedAttribute)
                    .as_concrete_TypeRef(),
                parameter.as_CFTypeRef(),
                &mut result,
            )
        };
        if status == kAXErrorSuccess && !result.is_null() {
            let text = unsafe { CFType::wrap_under_create_rule(result) };
            if cf_type_to_string(&text).as_deref() == Some(&candidate) {
                return Some(length);
            }
        }
        if !prefix.contains('\n') {
            break;
        }
    }
    None
}

/// Returns focus from Harper's popup, verifies the original editor and its selection,
/// then delivers native keyboard input. Never fall back to writing AXValue: it changes
/// Discord's rendered text without committing the corresponding draft edit.
fn apply_discord_edit(
    pid: i32,
    element: &AXUIElement,
    snapshot: &str,
    span: std::ops::Range<usize>,
    suggestion: &Suggestion,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::super::{ax_element_attribute, focused_window_pid::focused_window_pid};
    use accessibility::attribute::AXAttribute;
    use accessibility_sys::{kAXFocusedUIElementAttribute, kAXSelectedTextRangeAttribute};
    use core_graphics::{
        event::CGEvent,
        event_source::{CGEventSource, CGEventSourceStateID},
    };
    let app = AXUIElement::application(pid);
    let front = focused_window_pid()?;
    if front == std::process::id() as i32 {
        app.set_frontmost(true)?;
        // Chromium restores AXFocusedUIElement asynchronously after activation.
        for _ in 0..15 {
            let front = focused_window_pid()?;
            if front != std::process::id() as i32 && front != pid {
                break;
            }
            if front == pid && ax_element_attribute(&app, kAXFocusedUIElementAttribute).is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    if focused_window_pid()? != pid
        || ax_element_attribute(&app, kAXFocusedUIElementAttribute)? != *element
    {
        return Err("target is not focused".into());
    }
    let current = cf_type_to_string(&element.value()?).ok_or("not text")?;
    let (mut range, text) =
        discord_edit(snapshot, &current, span.clone(), suggestion).ok_or("stale suggestion")?;
    let start = if matches!(suggestion, Suggestion::InsertAfter(_)) {
        span.end
    } else {
        span.start
    };
    let prefix = |n| current.chars().take(n).collect::<String>();
    let location =
        discord_prefix_offset(element, &prefix(start)).ok_or("unmapped selection start")?;
    let end = discord_prefix_offset(element, &prefix(span.end)).ok_or("unmapped selection end")?;
    if end - location != range.length {
        return Err("non-text content inside edit".into());
    }
    range.location = location;
    if range.length == 0 && text.is_empty() {
        return Ok(());
    }
    let raw = unsafe { AXValueCreate(kAXValueTypeCFRange, (&range as *const CFRange).cast()) };
    if raw.is_null() {
        return Err("invalid selection".into());
    }
    let value = unsafe { CFType::wrap_under_create_rule(raw.cast()) };
    let selection = AXAttribute::new(&CFString::new(kAXSelectedTextRangeAttribute));
    element.set_attribute(&selection, value)?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
    loop {
        let selected = element.attribute(&selection)?;
        let mut actual = CFRange {
            location: 0,
            length: 0,
        };
        if !unsafe {
            AXValueGetValue(
                selected.as_CFTypeRef() as AXValueRef,
                kAXValueTypeCFRange,
                (&mut actual as *mut CFRange).cast(),
            )
        } {
            return Err("invalid selection readback".into());
        }
        if actual == range {
            break;
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!("selection {actual:?} did not settle to {range:?}").into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if focused_window_pid()? != pid
        || ax_element_attribute(&app, kAXFocusedUIElementAttribute)? != *element
    {
        return Err("editor focus changed".into());
    }
    let source =
        CGEventSource::new(CGEventSourceStateID::Private).map_err(|_| "keyboard unavailable")?;
    let key = if text.is_empty() { 51 } else { 0 };
    let event = |down| {
        CGEvent::new_keyboard_event(source.clone(), key, down)
            .map_err(|_| "keyboard event unavailable")
    };
    let down = event(true)?;
    let up = event(false)?;
    if !text.is_empty() {
        down.set_string(&text);
        up.set_string(&text);
    }
    down.post_to_pid(pid);
    up.post_to_pid(pid);
    Ok(())
}
