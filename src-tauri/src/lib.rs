use tauri::{
    image::Image,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

const SETTINGS_WINDOW_LABEL: &str = "settings";

fn menu_bar_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/menu-bar-icon.png")).map(Image::to_owned)
}

fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        window.show()?;
        window.set_focus()?;
        return Ok(());
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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let mut tray = TrayIconBuilder::with_id("harper-menu-bar")
                .icon(menu_bar_icon()?)
                .tooltip("Harper Desktop")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle().clone();

                        if let Err(error) = show_settings_window(&app) {
                            eprintln!("failed to show settings window: {error}");
                        }
                    }
                });

            #[cfg(target_os = "macos")]
            {
                tray = tray.icon_as_template(true);
            }

            tray.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
