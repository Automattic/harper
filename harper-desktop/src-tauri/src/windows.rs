//! Functions to manage the main windows involved in Harper Desktop

use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window};
use tauri_plugin_opener::OpenerExt;
use tracing::error;

const EDITOR_WINDOW_LABEL: &str = "editor";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const USER_WINDOW_LABELS: [&str; 2] = [EDITOR_WINDOW_LABEL, SETTINGS_WINDOW_LABEL];

pub fn is_user_window(label: &str) -> bool {
    USER_WINDOW_LABELS.contains(&label)
}

fn show_existing_window(window: &WebviewWindow) -> tauri::Result<()> {
    window.unminimize()?;
    window.show()?;
    window.set_focus()
}

/// Open the editor window, focusing it if it already exists.
pub fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)?;

    if let Some(window) = app.get_webview_window(EDITOR_WINDOW_LABEL) {
        return show_existing_window(&window);
    }

    let window = WebviewWindowBuilder::new(
        app,
        EDITOR_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Harper")
    .inner_size(800.0, 600.0)
    .build()?;
    window.set_focus()?;

    Ok(())
}

/// Open the settings window, focusing it if it already exists.
pub fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)?;

    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        return show_existing_window(&window);
    }

    WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("Harper Settings")
    .inner_size(920.0, 680.0)
    .min_inner_size(780.0, 520.0)
    .center()
    .build()?;

    Ok(())
}

pub fn hide_user_window(window: &Window) -> tauri::Result<()> {
    window.hide()?;

    #[cfg(target_os = "macos")]
    if !has_visible_or_minimized_user_window(window.app_handle())? {
        window
            .app_handle()
            .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn has_visible_or_minimized_user_window(app: &tauri::AppHandle) -> tauri::Result<bool> {
    for label in USER_WINDOW_LABELS {
        if let Some(window) = app.get_webview_window(label)
            && (window.is_visible()? || window.is_minimized()?)
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Open the browser to an issue report page.
pub fn open_issue_report(app: &tauri::AppHandle) {
    let _ = app
        .opener()
        .open_url(
            "https://github.com/Automattic/harper/issues/new/choose",
            None::<&str>,
        )
        .inspect_err(|err| error!("failed to open issue report URL: {err}"));
}
