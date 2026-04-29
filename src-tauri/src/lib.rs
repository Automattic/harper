use self::highlighter::Highlighter;

pub mod highlighter;
pub mod rect;

use rect::Rect;

#[cfg(target_os = "macos")]
mod macos;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    {
        macos::main();
    }

    if let Err(error) = Highlighter::new().and_then(|mut h| {
        h.set_rects(vec![Rect::new(100., 100., 100., 100.)]);
        h.run_window_for_each_monitor()
    }) {
        eprintln!("failed to run highlighter: {error}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
