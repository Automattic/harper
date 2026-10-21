//! Functions to manage the main windows involved in Harper Desktop

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tracing::error;

/// Synchronize macOS Dock visibility with the application's open windows.
///
/// Minimized windows still count as open. Exclude a destroyed window explicitly because
/// it may still be registered when its destruction event is delivered. Highlighter
/// windows belong to a separate process and do not affect this policy.
#[cfg(target_os = "macos")]
pub fn sync_dock_visibility(app: &AppHandle, destroyed_label: Option<&str>) -> tauri::Result<()> {
    let windows = app.webview_windows();
    let policy = activation_policy_for_windows(windows.keys().map(String::as_str), destroyed_label);
    let restore_icon = matches!(policy, tauri::ActivationPolicy::Regular);
    app.set_activation_policy(policy)?;

    if restore_icon {
        let _ = app
            .run_on_main_thread(restore_dock_icon)
            .inspect_err(|err| error!("Could not schedule Dock icon restoration: {err}"));
    }

    Ok(())
}

/// Reapply the bundled icon after restoring the app's Dock presence.
///
/// Tauri assigns the development-mode icon only at startup; recreating the Dock
/// entry can lose that image. Run this on the main thread after selecting Regular
/// policy, loading the embedded icon rather than relying on cached native state.
#[cfg(target_os = "macos")]
fn restore_dock_icon() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        error!("Dock icon restoration must run on the main thread");
        return;
    };
    let data = NSData::with_bytes(include_bytes!("../icons/icon.icns"));
    let Some(image) = NSImage::initWithData(mtm.alloc(), &data) else {
        error!("Could not decode the bundled Dock icon");
        return;
    };

    // SAFETY: The setter receives a non-nil image, and AppKit is accessed on the main thread.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image)) };
}

/// Select the activation policy from open window labels, ignoring a destroyed window.
#[cfg(target_os = "macos")]
fn activation_policy_for_windows<'a>(
    mut labels: impl Iterator<Item = &'a str>,
    destroyed_label: Option<&str>,
) -> tauri::ActivationPolicy {
    if labels.any(|label| Some(label) != destroyed_label) {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    }
}

/// Present a registered window after restoring its macOS Dock presence.
///
/// New windows are built hidden so activation policy changes precede showing and
/// focusing them. Failed window creation therefore leaves the policy unchanged.
fn show_window(window: WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    sync_dock_visibility(window.app_handle(), None)?;

    window.show()?;
    window.set_focus()
}

/// Open the editor window, focusing it if it already exists.
pub fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("editor") {
        return show_window(window);
    }

    let window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .title("Harper")
        .inner_size(800.0, 600.0)
        .visible(false)
        .build()?;

    show_window(window)
}

/// Open the settings window, focusing it if it already exists.
pub fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        return show_window(window);
    }

    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Harper Settings")
        .inner_size(920.0, 680.0)
        .min_inner_size(780.0, 520.0)
        .center()
        .visible(false)
        .build()?;

    show_window(window)
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
