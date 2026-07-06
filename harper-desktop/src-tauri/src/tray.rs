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

    let tray_icon = TrayIconBuilder::new()
        .icon(menu_bar_icon(highlighter_service.is_running())?)
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
        loop {
            sleep(Duration::from_millis(250)).await;

            {
                let highlighter_service: State<HighlighterService> = app.state();

                let Ok(new_icon) = menu_bar_icon(highlighter_service.is_running()) else {
                    error!("Unable to generate new menu bar icon.");
                    continue;
                };

                let _ = tray_icon
                    .set_icon(Some(new_icon))
                    .inspect_err(|err| error!("Unable to set new icon: {err}"));
            }
        }
    });

    Ok(())
}

fn service_status_color(is_running: bool) -> [u8; 4] {
    match is_running {
        true => [34, 197, 94, 255],
        false => [239, 68, 68, 255],
    }
}

pub fn menu_bar_icon(is_running: bool) -> tauri::Result<Image<'static>> {
    let icon = Image::from_bytes(include_bytes!("../icons/menu-bar-icon.png"))?;
    let width = icon.width();
    let height = icon.height();
    let mut rgba = icon.rgba().to_vec();

    draw_status_line(&mut rgba, width, height, service_status_color(is_running));

    Ok(Image::new_owned(rgba, width, height))
}

fn draw_status_line(rgba: &mut [u8], width: u32, height: u32, color: [u8; 4]) {
    let line_height = (height.min(width) / 14).max(3);
    let start_y = height.saturating_sub(line_height);

    for y in start_y..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            rgba[index..index + 4].copy_from_slice(&color);
        }
    }
}
