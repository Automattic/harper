use harper_core::linting::{Lint, Suggestion};
use std::{cell::RefCell, collections::BTreeMap};
use windows::{
    Win32::Foundation::{HWND, RPC_E_CHANGED_MODE},
    Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, SAFEARRAY,
    },
    Win32::System::Ole::{SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound},
    Win32::System::Variant::VARIANT,
    Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
        IUIAutomationTextPattern, IUIAutomationValuePattern, TextPatternRangeEndpoint_End,
        TextPatternRangeEndpoint_Start, TextUnit_Character, TreeScope_Subtree,
        UIA_ControlTypePropertyId, UIA_DocumentControlTypeId, UIA_EditControlTypeId,
        UIA_TextPatternId, UIA_ValuePatternId,
    },
    Win32::UI::HiDpi::GetDpiForWindow,
    core::BSTR,
};

use crate::rect::{ActionableLint, Rect};

pub type LintCallback<'a> = dyn FnMut(&str) -> BTreeMap<String, Vec<Lint>> + 'a;

thread_local! {
    static AUTOMATION: RefCell<Option<IUIAutomation>> = const { RefCell::new(None) };
}

fn with_automation<F, T>(f: F) -> Option<T>
where
    F: FnOnce(&IUIAutomation) -> T,
{
    AUTOMATION.with(|cell| {
        let mut guard = cell.borrow_mut();
        if guard.is_none() {
            *guard = init_automation().ok();
        }
        guard.as_ref().map(f)
    })
}

fn init_automation() -> windows::core::Result<IUIAutomation> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .or_else(|e| {
            if e.code() == RPC_E_CHANGED_MODE {
                Ok(())
            } else {
                Err(e)
            }
        })?;

    unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
}

/// Returns the UIA automation instance and the currently focused element.
pub fn focused_element() -> Option<(IUIAutomation, IUIAutomationElement)> {
    with_automation(|automation| {
        let element = unsafe { automation.GetFocusedElement() }.ok()?;
        Some((automation.clone(), element))
    })
    .flatten()
}

pub fn collect_rects(
    root: &IUIAutomationElement,
    automation: &IUIAutomation,
    lint_text: &mut LintCallback<'_>,
) -> Vec<ActionableLint> {
    let Ok(condition) = edit_control_condition(automation) else {
        return Vec::new();
    };

    let Ok(elements) = (unsafe { root.FindAll(TreeScope_Subtree, &condition) }) else {
        return Vec::new();
    };

    let count = unsafe { elements.Length() }.unwrap_or(0);
    let mut rects = Vec::new();

    for i in 0..count {
        let Ok(element) = (unsafe { elements.GetElement(i) }) else {
            continue;
        };
        collect_rects_for_element(&element, lint_text, &mut rects);
    }

    rects
}

fn collect_rects_for_element(
    element: &IUIAutomationElement,
    lint_text: &mut LintCallback<'_>,
    rects: &mut Vec<ActionableLint>,
) {
    let scale_factor = match unsafe { element.CurrentNativeWindowHandle() } {
        Ok(hwnd) if !hwnd.0.is_null() => {
            let dpi = unsafe { GetDpiForWindow(hwnd) };
            if dpi > 0 {
                dpi as f64 / 96.0
            } else {
                1.0
            }
        }
        _ => 1.0,
    };

    let text = match element_text(element) {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    let organized_lints = lint_text(&text);

    for (rule_name, lints) in organized_lints {
        for lint in lints {
            let Some(rect) = text_range_rect(element, lint.span.start, lint.span.len(), scale_factor) else {
                continue;
            };

            let element_clone = element.clone();
            let source_text = text.clone();
            let suggestion_source_text = text.clone();
            let suggestion_lint = lint.clone();

            rects.push(ActionableLint::new(
                rect,
                rule_name.clone(),
                lint,
                source_text,
                move |suggestion| {
                    apply_suggestion_to_element(
                        &element_clone,
                        suggestion_source_text.clone(),
                        suggestion_lint.clone(),
                        suggestion,
                    );
                },
            ));
        }
    }
}

/// Reads the text value of an element, trying TextPattern first then ValuePattern.
fn element_text(element: &IUIAutomationElement) -> Option<String> {
    if let Ok(pattern) =
        unsafe { element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }
    {
        if let Ok(range) = unsafe { pattern.DocumentRange() } {
            if let Ok(bstr) = unsafe { range.GetText(-1) } {
                let s = bstr.to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }

    if let Ok(pattern) =
        unsafe { element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) }
    {
        if let Ok(bstr) = unsafe { pattern.CurrentValue() } {
            let s = bstr.to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }

    None
}

/// Returns screen-space bounds for a character range using TextPattern.
fn text_range_rect(element: &IUIAutomationElement, start: usize, length: usize, scale_factor: f64) -> Option<Rect> {
    let pattern =
        unsafe { element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) }
            .ok()?;

    let doc_range = unsafe { pattern.DocumentRange() }.ok()?;
    let text = unsafe { doc_range.GetText(-1) }.ok()?.to_string();

    let char_count = text.chars().count();
    if start >= char_count || length == 0 {
        return None;
    }
    let end = (start + length).min(char_count);

    let range = unsafe { doc_range.Clone() }.ok()?;

    unsafe {
        range.MoveEndpointByUnit(
            TextPatternRangeEndpoint_Start,
            TextUnit_Character,
            start as i32,
        )
    }
    .ok()?;

    unsafe {
        range.MoveEndpointByUnit(
            TextPatternRangeEndpoint_End,
            TextUnit_Character,
            -(char_count as i32 - end as i32),
        )
    }
    .ok()?;

    let sa = unsafe { range.GetBoundingRectangles() }.ok()?;
    let values = unsafe { read_safearray_f64(sa) };

    if values.len() < 4 {
        return None;
    }

    let (x, y, w, h) = (values[0], values[1], values[2], values[3]);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }

    Some(Rect::new(
        x / scale_factor,
        y / scale_factor,
        w / scale_factor,
        h / scale_factor,
    ))
}

/// Reads a SAFEARRAY of VT_R8 (f64) values into a Vec.
unsafe fn read_safearray_f64(sa: *mut SAFEARRAY) -> Vec<f64> {
    if sa.is_null() {
        return Vec::new();
    }

    let lb = match unsafe { SafeArrayGetLBound(sa, 1) } {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let ub = match unsafe { SafeArrayGetUBound(sa, 1) } {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    if ub < lb {
        return Vec::new();
    }

    let count = (ub - lb + 1) as usize;
    let mut values = vec![0.0f64; count];

    for i in 0..count {
        let idx = lb + i as i32;
        let _ = unsafe { SafeArrayGetElement(sa, &idx, values[i..].as_mut_ptr() as *mut _) };
    }

    values
}

fn apply_suggestion_to_element(
    element: &IUIAutomationElement,
    source_text: String,
    lint: Lint,
    suggestion: Suggestion,
) {
    let mut chars: Vec<char> = source_text.chars().collect();
    suggestion.apply(lint.span, &mut chars);
    let updated: String = chars.into_iter().collect();

    if let Ok(pattern) =
        unsafe { element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) }
    {
        let bstr = BSTR::from(updated.as_str());
        let _ = unsafe { pattern.SetValue(&bstr) };
    }
}

fn edit_control_condition(
    automation: &IUIAutomation,
) -> windows::core::Result<IUIAutomationCondition> {
    let edit_cond = unsafe {
        automation.CreatePropertyCondition(
            UIA_ControlTypePropertyId,
            &VARIANT::from(UIA_EditControlTypeId.0 as i32),
        )?
    };
    let doc_cond = unsafe {
        automation.CreatePropertyCondition(
            UIA_ControlTypePropertyId,
            &VARIANT::from(UIA_DocumentControlTypeId.0 as i32),
        )?
    };
    unsafe { automation.CreateOrCondition(&edit_cond, &doc_cond) }
}
