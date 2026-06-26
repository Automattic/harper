use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{Error, TreeVisitor, TreeWalker, TreeWalkerFlow};
use accessibility_sys::{
    AXUIElementCopyParameterizedAttributeValue, AXValueCreate, AXValueGetType, AXValueGetValue,
    AXValueRef, kAXBoundsForRangeParameterizedAttribute, kAXErrorIllegalArgument, kAXErrorNoValue,
    kAXErrorParameterizedAttributeUnsupported, kAXErrorSuccess, kAXValueTypeCFRange,
    kAXValueTypeCGRect,
};
use core::{ffi::c_void, mem::MaybeUninit};
use core_foundation::base::{CFRange, CFType, TCFType};
use core_foundation::string::CFString;
use harper_core::linting::{Lint, Suggestion};
use objc2_foundation::NSRect;
use std::{cell::RefCell, ptr};

use crate::rect::{ActionableLint, Rect};

/// Outcome of asking macOS for the bounds of a text range.
pub(super) enum TextRangeBoundsProbe {
    Success(Rect),
    Unavailable,
    InvalidRangeValue,
    AxError(i32),
}

impl TextRangeBoundsProbe {
    /// Returns true only when the probe produced non-zero text geometry.
    pub(super) fn has_usable_text_metrics(self) -> bool {
        matches!(self, Self::Success(_))
    }
}

/// Converts a Core Foundation value to a Rust string when it is a `CFString`.
pub(super) fn cf_type_to_string(value: &CFType) -> Option<String> {
    value
        .instance_of::<CFString>()
        .then(|| unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _) }.to_string())
}

/// Probes `AXBoundsForRange` for a specific text range on an element.
pub(super) fn probe_element_rect_for_text_range(
    element: &AXUIElement,
    start_index: isize,
    length: isize,
) -> TextRangeBoundsProbe {
    let range = CFRange {
        location: start_index,
        length,
    };

    let range_value_ref = unsafe {
        AXValueCreate(
            kAXValueTypeCFRange,
            &range as *const CFRange as *const c_void,
        )
    };

    if range_value_ref.is_null() {
        return TextRangeBoundsProbe::InvalidRangeValue;
    }

    let range_value = unsafe { CFType::wrap_under_create_rule(range_value_ref as _) };
    let attr = CFString::new(kAXBoundsForRangeParameterizedAttribute);
    let mut value = ptr::null();

    let error = unsafe {
        AXUIElementCopyParameterizedAttributeValue(
            element.as_concrete_TypeRef(),
            attr.as_concrete_TypeRef(),
            range_value.as_CFTypeRef(),
            &mut value,
        )
    };

    if error == kAXErrorNoValue || error == kAXErrorParameterizedAttributeUnsupported {
        return TextRangeBoundsProbe::Unavailable;
    }

    if error != kAXErrorSuccess {
        return TextRangeBoundsProbe::AxError(error);
    }

    if value.is_null() {
        return TextRangeBoundsProbe::Unavailable;
    }

    let value = unsafe { CFType::wrap_under_create_rule(value) };
    let ax_value = value.as_CFTypeRef() as AXValueRef;
    let value_type = unsafe { AXValueGetType(ax_value) };

    if value_type != kAXValueTypeCGRect {
        return TextRangeBoundsProbe::Unavailable;
    }

    let mut rect = MaybeUninit::<NSRect>::uninit();

    let ok = unsafe {
        AXValueGetValue(
            ax_value,
            kAXValueTypeCGRect,
            rect.as_mut_ptr() as *mut c_void,
        )
    };

    if !ok {
        return TextRangeBoundsProbe::Unavailable;
    }

    let rect = unsafe { rect.assume_init() };

    let rect = Rect {
        x: rect.origin.x,
        y: rect.origin.y,
        width: rect.size.width,
        height: rect.size.height,
    };

    if rect_has_usable_text_metrics(rect) {
        TextRangeBoundsProbe::Success(rect)
    } else {
        TextRangeBoundsProbe::Unavailable
    }
}

/// Returns whether a text rectangle has enough geometry to render a highlight.
fn rect_has_usable_text_metrics(rect: Rect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}

/// Collects lint rectangles while walking supported text elements in an AX tree.
pub(super) struct RectCollector<'a> {
    rects: RefCell<Vec<ActionableLint>>,
    lint_text: RefCell<&'a mut dyn FnMut(&str) -> std::collections::BTreeMap<String, Vec<Lint>>>,
}

impl TreeVisitor for RectCollector<'_> {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(value) = element.value()
            && is_supported_text_element(element)
        {
            let string =
                unsafe { CFString::wrap_under_get_rule(value.as_CFTypeRef() as _).to_string() };

            let mut rects = self.rects.borrow_mut();
            let organized_lints = (self.lint_text.borrow_mut())(&string);

            for (rule_name, lints) in organized_lints {
                for lint in lints {
                    if let Some(rect) = element_rect_for_text_range_with_fallback(
                        element,
                        &string,
                        lint.span.start,
                        lint.span.len(),
                    ) {
                        let element = element.clone();
                        let source_text = string.clone();
                        let suggestion_source_text = string.clone();
                        let suggestion_lint = lint.clone();

                        rects.push(ActionableLint::new(
                            rect,
                            rule_name.clone(),
                            lint,
                            source_text,
                            move |suggestion| {
                                apply_suggestion_to_element(
                                    element,
                                    suggestion_source_text,
                                    suggestion_lint,
                                    suggestion,
                                );
                            },
                        ));
                    }
                }
            }
        }

        TreeWalkerFlow::Continue
    }
    fn exit_element(&self, _element: &AXUIElement) {}
}

impl<'a> RectCollector<'a> {
    pub(super) fn new(
        lint_text: &'a mut dyn FnMut(&str) -> std::collections::BTreeMap<String, Vec<Lint>>,
    ) -> Self {
        Self {
            rects: RefCell::new(Vec::new()),
            lint_text: RefCell::new(lint_text),
        }
    }

    pub(super) fn unwrap_rects(self) -> Vec<ActionableLint> {
        self.rects.into_inner()
    }
}

fn apply_suggestion_to_element(
    element: AXUIElement,
    source_text: String,
    lint: Lint,
    suggestion: Suggestion,
) {
    let mut chars = source_text.chars().collect::<Vec<_>>();
    suggestion.apply(lint.span, &mut chars);
    let updated = chars.into_iter().collect::<String>();
    let value = CFString::new(&updated);

    if let Err(error) = element.set_value(value.as_CFType()) {
        eprintln!("Unable to apply suggestion: {error}");
    }
}

/// Returns whether an accessibility element has a text role Harper can lint.
pub(super) fn is_supported_text_element(el: &AXUIElement) -> bool {
    if let Ok(role) = el.role() {
        return is_supported_text_role(&role.to_string());
    }

    false
}

/// Returns whether an AX role represents editable text supported by the highlighter.
fn is_supported_text_role(role: &str) -> bool {
    matches!(role, "AXTextArea" | "AXTextField")
}

/// Collects `AXStaticText` descendants with their string values.
///
/// Chromium-based apps (e.g. Slack) report degenerate `AXBoundsForRange`
/// rects on the editable `AXTextArea` wrapper but expose usable bounds on the
/// `AXStaticText` leaf nodes inside it.
struct StaticTextCollector {
    texts: RefCell<Vec<(AXUIElement, String)>>,
}

impl StaticTextCollector {
    fn new() -> Self {
        Self {
            texts: RefCell::new(Vec::new()),
        }
    }

    fn into_texts(self) -> Vec<(AXUIElement, String)> {
        self.texts.into_inner()
    }
}

impl TreeVisitor for StaticTextCollector {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        if let Ok(role) = element.role()
            && role == "AXStaticText"
            && let Ok(value) = element.value()
            && let Some(string) = cf_type_to_string(&value)
            && !string.is_empty()
        {
            self.texts.borrow_mut().push((element.clone(), string));
        }

        TreeWalkerFlow::Continue
    }

    fn exit_element(&self, _element: &AXUIElement) {}
}

/// Finds `needle` within `haystack` starting at char offset `from`.
fn find_char_subslice(haystack: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    let end = haystack.len() - needle.len();
    (from..=end).find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// Resolves text-range bounds, falling back to `AXStaticText` descendants
/// when the element itself reports unusable bounds (Chromium-based apps).
pub(super) fn element_rect_for_text_range_with_fallback(
    element: &AXUIElement,
    full_text: &str,
    start_index: usize,
    length: usize,
) -> Option<Rect> {
    if let Ok(Some(rect)) =
        element_rect_for_text_range(element, start_index as isize, length as isize)
    {
        return Some(rect);
    }

    let walker = TreeWalker::new();
    let collector = StaticTextCollector::new();
    walker.walk(element, &collector);

    let full_chars: Vec<char> = full_text.chars().collect();
    let mut cursor = 0usize;

    for (child, child_text) in collector.into_texts() {
        let child_chars: Vec<char> = child_text.chars().collect();
        // Locate this child's text within the parent value, starting at the
        // cursor, to absorb any separators the parent inserts between children.
        let Some(child_start) = find_char_subslice(&full_chars, &child_chars, cursor) else {
            continue;
        };
        let child_end = child_start + child_chars.len();

        if start_index >= child_start
            && start_index + length <= child_end
            && let Ok(Some(rect)) = element_rect_for_text_range(
                &child,
                (start_index - child_start) as isize,
                length as isize,
            )
        {
            return Some(rect);
        }

        cursor = child_end;
    }

    None
}

fn element_rect_for_text_range(
    element: &AXUIElement,
    start_index: isize,
    length: isize,
) -> Result<Option<Rect>, Error> {
    match probe_element_rect_for_text_range(element, start_index, length) {
        TextRangeBoundsProbe::Success(rect) => Ok(Some(rect)),
        TextRangeBoundsProbe::InvalidRangeValue => Err(Error::Ax(kAXErrorIllegalArgument)),
        TextRangeBoundsProbe::AxError(error) => Err(Error::Ax(error)),
        TextRangeBoundsProbe::Unavailable => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rect::Rect;

    #[test]
    fn supported_text_roles_include_text_area_and_text_field() {
        assert!(is_supported_text_role("AXTextArea"));
        assert!(is_supported_text_role("AXTextField"));
    }

    #[test]
    fn supported_text_roles_reject_unrelated_roles() {
        assert!(!is_supported_text_role("AXButton"));
        assert!(!is_supported_text_role("AXStaticText"));
        assert!(!is_supported_text_role(""));
    }

    #[test]
    fn finds_char_subslice_respecting_offset() {
        let haystack: Vec<char> = "one\ntwo two".chars().collect();
        let needle: Vec<char> = "two".chars().collect();

        assert_eq!(find_char_subslice(&haystack, &needle, 0), Some(4));
        assert_eq!(find_char_subslice(&haystack, &needle, 5), Some(8));
        assert_eq!(find_char_subslice(&haystack, &needle, 9), None);
        assert_eq!(find_char_subslice(&haystack, &[], 0), None);
        assert_eq!(find_char_subslice(&[], &needle, 0), None);
    }

    #[test]
    fn text_range_bounds_probe_requires_non_zero_geometry() {
        let usable = TextRangeBoundsProbe::Success(Rect {
            x: 10.0,
            y: 20.0,
            width: 1.0,
            height: 12.0,
        });
        let unavailable = TextRangeBoundsProbe::Unavailable;

        assert!(usable.has_usable_text_metrics());
        assert!(!unavailable.has_usable_text_metrics());
        assert!(rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }));
        assert!(!rect_has_usable_text_metrics(Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 1.0,
        }));
    }
}
