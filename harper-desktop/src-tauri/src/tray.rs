use std::time::Duration;

use tauri::image::Image;
use tauri::tray::TrayIconBuilder;
use tokio::time::sleep;
use tracing::error;

use tauri::menu::{Menu, MenuBuilder};
use tauri::{AppHandle, Manager, Runtime, State};
use tokio::runtime::Runtime as AsyncRuntime;

use crate::highlighter_service::HighlighterService;
use crate::windows::{open_issue_report, show_editor_window, show_settings_window};

/// Defines the layout of the tray menu.
fn tray_menu<R: Runtime, M: Manager<R>>(manager: &M) -> tauri::Result<Menu<R>> {
    MenuBuilder::new(manager)
        .text("toggle-service", "Toggle Service")
        .text("open-editor", "Open Editor")
        .text("settings", "Settings")
        .text("report-issue", "Report Issue")
        .text("quit", "Quit")
        .build()
}

pub fn set_up_tray_menu(app: &AppHandle) -> tauri::Result<()> {
    let highlighter_service: State<HighlighterService> = app.state();
    let async_runtime: State<AsyncRuntime> = app.state();

    let initial_is_running = highlighter_service.is_running();

    let tray_icon = TrayIconBuilder::new()
        .icon(menu_bar_icon(initial_is_running)?)
        .icon_as_template(cfg!(target_os = "macos"))
        .menu(&tray_menu(app)?)
        .on_menu_event(move |app, event| {
            let event_id = event.id().0.as_str();

            match event_id {
                "toggle-service" => {
                    let service: State<HighlighterService> = app.state();

                    let _ = service
                        .toggle()
                        .inspect_err(|err| error!("Could not toggle highlighter: {err}"));
                }
                "open-editor" => {
                    let _ = show_editor_window(app)
                        .inspect_err(|err| error!("Could not open the editor window: {err}"));
                }
                "settings" => {
                    let _ = show_settings_window(app)
                        .inspect_err(|err| error!("Could not open the settings window: {err}"));
                }
                "report-issue" => open_issue_report(app),
                "quit" => app.exit(0),
                _ => error!("Encountered unexpected event: `{event_id}`"),
            };
        })
        .build(app)?;

    let app = app.clone();
    async_runtime.spawn(async move {
        let mut shown_is_running = initial_is_running;

        loop {
            sleep(Duration::from_millis(250)).await;

            let is_running = {
                let highlighter_service: State<HighlighterService> = app.state();
                highlighter_service.is_running()
            };

            // Only touch the tray when the service state actually changes.
            // Re-decoding the icon and calling set_icon four times a second
            // regardless is pure waste, and on Windows it eventually starts
            // failing: after several days the shell began rejecting every
            // update with ERROR_TIMEOUT, filling the log and leaving the tray
            // icon unresponsive.
            if is_running == shown_is_running {
                continue;
            }

            let Ok(new_icon) = menu_bar_icon(is_running) else {
                error!("Unable to generate new menu bar icon.");
                continue;
            };

            match tray_icon.set_icon_with_as_template(Some(new_icon), cfg!(target_os = "macos")) {
                Ok(()) => shown_is_running = is_running,
                // Leave the remembered state alone so the next tick retries.
                Err(err) => error!("Unable to set new icon: {err}"),
            }
        }
    });

    Ok(())
}

/// Builds the logo with a dot beneath it only while the service is running.
/// On macOS the alpha channel serves as a template mask; other platforms show white.
/// Both states retain the asset's original dimensions and logo placement.
pub fn menu_bar_icon(is_running: bool) -> tauri::Result<Image<'static>> {
    let icon = Image::from_bytes(include_bytes!("../icons/menu-bar-icon.png"))?;
    let width = icon.width();
    let height = icon.height();
    let mut rgba = icon.rgba().to_vec();

    if is_running {
        draw_status_dot(&mut rgba, width, height);
    }

    Ok(Image::new_owned(rgba, width, height))
}

/// Draws a white, antialiased status dot in the transparent space below the logo.
/// Pixel coverage controls alpha so macOS can tint the dot with the template image.
fn draw_status_dot(rgba: &mut [u8], width: u32, height: u32) {
    let radius = width.min(height) as f32 / 20.0;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 - radius * 2.0;

    for y in 0..height {
        for x in 0..width {
            let distance = (x as f32 + 0.5 - center_x).hypot(y as f32 + 0.5 - center_y);
            let coverage = (radius + 0.5 - distance).clamp(0.0, 1.0);
            if coverage > 0.0 {
                let index = ((y * width + x) * 4) as usize;
                rgba[index..index + 4].copy_from_slice(&[
                    255,
                    255,
                    255,
                    (coverage * 255.0).round() as u8,
                ]);
            }
        }
    }
}
