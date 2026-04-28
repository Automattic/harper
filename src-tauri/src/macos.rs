use accessibility::attribute::AXUIElementAttributes;
use accessibility::ui_element::AXUIElement;
use accessibility::{TreeVisitor, TreeWalker, TreeWalkerFlow};

pub fn main() {
    let el = AXUIElement::application(57046);

    let walker = TreeWalker::new();
    walker.walk(&el, &Printing);

    struct Printing;
    impl TreeVisitor for Printing {
        fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
            if let Ok(value) = element.value() && is_textarea(element){
                dbg!(value);
            }

            TreeWalkerFlow::Continue
        }
        fn exit_element(&self, element: &AXUIElement) {}
    }
}

fn is_textarea(el: &AXUIElement) -> bool {
    if let Ok(role) = el.role() && role == "AXTextArea"{
       true 
    }

    false
}
