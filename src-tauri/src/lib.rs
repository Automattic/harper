pub mod highlighter;

#[cfg(target_os = "macos")]
mod macos;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    {
        macos::main();
    }

    if let Err(error) = highlighter::Highlighter::new()
        .and_then(highlighter::Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
