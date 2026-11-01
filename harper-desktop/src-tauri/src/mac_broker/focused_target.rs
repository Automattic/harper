use accessibility::ui_element::AXUIElement;
use accessibility_sys::pid_t;

/// The last externally focused application and its exact focused accessibility element.
///
/// The owned AX reference keeps the element handle alive while the overlay has focus, but does not
/// prevent the source application from removing the element. A failed focus lookup leaves no element
/// rather than retaining an older field or remembering an application-wide traversal fallback.
#[derive(Clone)]
pub struct FocusedTarget {
    pub pid: pid_t,
    pub element: Option<AXUIElement>,
}

impl FocusedTarget {
    /// Selects the remembered element, allowing application-wide fallback only during external focus.
    pub fn traversal_root(&self, highlighter_focused: bool) -> Option<AXUIElement> {
        self.element
            .clone()
            .or_else(|| (!highlighter_focused).then(|| AXUIElement::application(self.pid)))
    }
}
