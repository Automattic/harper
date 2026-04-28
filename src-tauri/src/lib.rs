#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    {
        use accessibility::attribute::AXUIElementAttributes;
        use accessibility::ui_element::AXUIElement;
        use accessibility::{TreeVisitor, TreeWalker, TreeWalkerFlow};

        let el = AXUIElement::application(57046);

        let walker = TreeWalker::new();
        walker.walk(&el, &Printing);

        struct Printing;
        impl TreeVisitor for Printing {
            fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
                if let Ok(value) = element.value() {
                    dbg!(value);
                }

                TreeWalkerFlow::Continue
            }
            fn exit_element(&self, element: &AXUIElement) {}
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
