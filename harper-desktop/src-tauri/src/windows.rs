//! Functions to manage the main windows involved in Harper Desktop

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tracing::error;

/// Open the editor window, focusing it if it already exists.
pub fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("editor") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .title("Harper")
        .inner_size(800.0, 600.0)
        .build()?;
    window.set_focus()?;

    Ok(())
}

/// Open the settings window, focusing it if it already exists.
pub fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Harper Settings")
        .inner_size(920.0, 680.0)
        .min_inner_size(780.0, 520.0)
        .center()
        .build()?;

    Ok(())
}

/// Open the browser to an issue report page.
pub fn open_issue_report(app: &AppHandle) {
    let _ = app
        .opener()
        .open_url(
            "https://github.com/Automattic/harper/issues/new/choose",
            None::<&str>,
        )
        .inspect_err(|err| error!("failed to open issue report URL: {err}"));
}
