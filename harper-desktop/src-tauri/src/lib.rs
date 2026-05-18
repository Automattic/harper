use self::highlighter::Highlighter;
use self::highlighter_service::HighlighterService;
use crate::communication::{Client, ProtocolError};
use crate::config::{Config, Integration};
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, DictWordMetadata, Document, IgnoredLints,
    linting::{FlatConfig, Lint, LintGroup},
    spell::{Dictionary, MutableDictionary},
};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tauri::{
    Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};

use crate::os_broker::{AccessibilityPermissionStatus, OsBroker};
use tokio::{
    io::{Stdin, Stdout},
    runtime::{Builder, Runtime},
    sync::Mutex,
};

pub mod color;
pub mod communication;
pub mod config;
pub mod highlighter;
pub mod highlighter_process;
pub mod highlighter_service;
pub mod lint_kind_color;
mod os_broker;
pub mod rect;

#[cfg(target_os = "macos")]
mod mac_broker;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Highlighter,
}

const EDITOR_WINDOW_LABEL: &str = "main";
const SETTINGS_WINDOW_LABEL: &str = "settings";
const TRAY_MENU_BAR_ID: &str = "harper-menu-bar";
const TOGGLE_SERVICE_MENU_ID: &str = "toggle-service";
const OPEN_EDITOR_MENU_ID: &str = "open-editor";
const SETTINGS_MENU_ID: &str = "settings";
const QUIT_MENU_ID: &str = "quit";

struct TrayMenu {
    menu: Menu<tauri::Wry>,
    service_toggle: MenuItem<tauri::Wry>,
}

fn service_menu_text(is_running: bool) -> &'static str {
    match is_running {
        true => "Stop Harper Service",
        false => "Start Harper Service",
    }
}

fn service_status_color(is_running: bool) -> [u8; 4] {
    match is_running {
        true => [34, 197, 94, 255],
        false => [239, 68, 68, 255],
    }
}

fn menu_bar_icon(is_running: bool) -> tauri::Result<Image<'static>> {
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

fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(EDITOR_WINDOW_LABEL) {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        EDITOR_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("harper-desktop")
    .inner_size(800.0, 600.0)
    .build()?;
    window.set_focus()?;

    Ok(())
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

fn tray_menu(app: &tauri::App, is_running: bool) -> tauri::Result<TrayMenu> {
    let service_toggle = MenuItem::with_id(
        app,
        TOGGLE_SERVICE_MENU_ID,
        service_menu_text(is_running),
        true,
        None::<&str>,
    )?;
    let open_editor =
        MenuItem::with_id(app, OPEN_EDITOR_MENU_ID, "Open Editor", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, SETTINGS_MENU_ID, "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&service_toggle, &open_editor, &separator, &settings, &quit],
    )?;

    Ok(TrayMenu {
        menu,
        service_toggle,
    })
}

fn update_service_tray_state(
    app: &tauri::AppHandle,
    service_toggle: &MenuItem<tauri::Wry>,
    is_running: bool,
) -> tauri::Result<()> {
    service_toggle.set_text(service_menu_text(is_running))?;

    if let Some(tray) = app.tray_by_id(TRAY_MENU_BAR_ID) {
        tray.set_icon(Some(menu_bar_icon(is_running)?))?;
    }

    Ok(())
}

fn should_hide_window_on_close(label: &str) -> bool {
    label == EDITOR_WINDOW_LABEL || label == SETTINGS_WINDOW_LABEL
}

#[tauri::command]
async fn get_lint_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<FlatConfig, String> {
    let mut lint_config = config.lock().await.lint_config.clone();
    lint_config.fill_with_curated();

    Ok(lint_config)
}

#[tauri::command]
async fn get_dialect(config: State<'_, Arc<Mutex<Config>>>) -> Result<Dialect, String> {
    Ok(config.lock().await.dialect)
}

#[tauri::command]
async fn set_dialect(
    dialect: Dialect,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.dialect = dialect;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn set_lint_config(
    lint_config: FlatConfig,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut lint_config = lint_config;
    lint_config.fill_with_curated();

    let mut config = config.lock().await;
    config.lint_config = lint_config;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_dictionary(config: State<'_, Arc<Mutex<Config>>>) -> Result<Vec<String>, String> {
    let mut words = config
        .lock()
        .await
        .mutable_dictionary
        .words_iter()
        .map(|word| word.iter().collect::<String>())
        .collect::<Vec<_>>();
    words.sort();

    Ok(words)
}

#[tauri::command]
async fn set_dictionary(
    words: Vec<String>,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut dictionary = MutableDictionary::new();
    dictionary.extend_words(words.into_iter().map(|word| {
        (
            word.chars().collect::<Vec<_>>(),
            DictWordMetadata::default(),
        )
    }));

    let mut config = config.lock().await;
    config.mutable_dictionary = dictionary;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn ignore_lint(
    ignored_lints: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let ignored_lints =
        serde_json::from_str::<IgnoredLints>(&ignored_lints).map_err(|error| error.to_string())?;

    let mut config = config.lock().await;
    config.ignored_lints.append(ignored_lints);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn add_to_dictionary(
    word: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config
        .mutable_dictionary
        .append_word_str(&word, DictWordMetadata::default());
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_integrations(
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<Vec<Integration>, String> {
    Ok(config.lock().await.integrations.clone())
}

#[tauri::command]
async fn add_integration(
    bundle_id: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.add_integration(bundle_id);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn remove_integration(
    bundle_id: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.remove_integration(&bundle_id);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn set_integration_enabled(
    bundle_id: String,
    enabled: bool,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.set_integration_enabled(&bundle_id, enabled);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_accessibility_permission_status() -> AccessibilityPermissionStatus {
    platform_broker().accessibility_permission_status()
}

#[tauri::command]
fn request_accessibility_permission() -> AccessibilityPermissionStatus {
    platform_broker().request_accessibility_permission()
}

#[tauri::command]
fn launch_app(bundle_id: String) -> Result<(), String> {
    platform_broker().launch_app_bundle(&bundle_id)
}

#[cfg(target_os = "macos")]
fn platform_broker() -> impl OsBroker {
    mac_broker::MacBroker::default()
}

#[cfg(not(target_os = "macos"))]
fn platform_broker() -> impl OsBroker {
    os_broker::NoopBroker
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args = Args::parse();

    match args.command {
        Some(Command::Highlighter) => run_highlighter(),
        None => run_tauri(),
    }
}

pub fn run_tauri() {
    let config_runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build config runtime");
    let config = match config_runtime.block_on(Config::load_from_system()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load config, using defaults: {error}");
            Config::new()
        }
    };
    let config = Arc::new(Mutex::new(config));
    let highlighter_service = HighlighterService::new(config.clone());

    if let Err(error) = highlighter_service.start() {
        eprintln!("failed to start highlighter service: {error}");
    }

    tauri::Builder::default()
        .manage(config)
        .manage(highlighter_service)
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_lint_config,
            get_dialect,
            set_dialect,
            set_lint_config,
            get_dictionary,
            set_dictionary,
            ignore_lint,
            add_to_dictionary,
            get_integrations,
            add_integration,
            remove_integration,
            set_integration_enabled,
            get_accessibility_permission_status,
            request_accessibility_permission,
            launch_app,
        ])
        .on_window_event(|window, event| {
            if should_hide_window_on_close(window.label())
                && let WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();

                if let Err(error) = window.hide() {
                    eprintln!("failed to hide {} window: {error}", window.label());
                }
            }
        })
        .setup(|app| {
            let is_service_running = app.state::<HighlighterService>().is_running();
            let menu = tray_menu(app, is_service_running)?;
            let service_toggle = menu.service_toggle.clone();

            let tray = TrayIconBuilder::with_id(TRAY_MENU_BAR_ID)
                .menu(&menu.menu)
                .icon(menu_bar_icon(is_service_running)?)
                .tooltip("Harper Desktop")
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    TOGGLE_SERVICE_MENU_ID => match app.state::<HighlighterService>().toggle() {
                        Ok(status) => {
                            if let Err(error) =
                                update_service_tray_state(app, &service_toggle, status)
                            {
                                eprintln!("failed to update service tray state: {error}");
                            }
                        }
                        Err(error) => {
                            eprintln!("failed to toggle highlighter service: {error}");

                            let is_running = app.state::<HighlighterService>().is_running();
                            if let Err(error) =
                                update_service_tray_state(app, &service_toggle, is_running)
                            {
                                eprintln!("failed to update service tray state: {error}");
                            }
                        }
                    },
                    OPEN_EDITOR_MENU_ID => {
                        if let Err(error) = show_editor_window(app) {
                            eprintln!("failed to show editor window: {error}");
                        }
                    }
                    SETTINGS_MENU_ID => {
                        if let Err(error) = show_settings_window(app) {
                            eprintln!("failed to show settings window: {error}");
                        }
                    }
                    QUIT_MENU_ID => app.exit(0),
                    _ => {}
                })
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

            tray.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_highlighter() {
    let client = Rc::new(RefCell::new(Client::current_process()));
    let sync_runtime = Rc::new(
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build highlighter protocol runtime"),
    );

    let startup_config = fetch_highlighter_config(&mut client.borrow_mut(), &sync_runtime);

    let startup_config = match startup_config {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to hydrate highlighter config, using defaults: {error}");
            Config::new()
        }
    };

    let startup_linter = startup_config.create_linter();
    let ignored_lints = Rc::new(RefCell::new(startup_config.ignored_lints));
    let user_dictionary = Rc::new(RefCell::new(startup_config.mutable_dictionary));
    let dialect = Rc::new(RefCell::new(startup_config.dialect));
    let integrations = Rc::new(RefCell::new(startup_config.integrations));
    let linter = Rc::new(RefCell::new(startup_linter));

    let lint_ignored_lints = ignored_lints.clone();
    let lint_linter = linter.clone();
    let lint_user_dictionary = user_dictionary.clone();

    let ignore_client = client.clone();
    let ignore_runtime = sync_runtime.clone();
    let ignore_ignored_lints = ignored_lints.clone();

    let dictionary_client = client.clone();
    let dictionary_runtime = sync_runtime.clone();
    let dictionary_user_dictionary = user_dictionary.clone();
    let dictionary_linter = linter.clone();
    let dictionary_dialect = dialect.clone();

    let disable_client = client.clone();
    let disable_runtime = sync_runtime.clone();
    let disable_linter = linter.clone();

    let refresh_client = client.clone();
    let refresh_runtime = sync_runtime.clone();
    let refresh_ignored_lints = ignored_lints.clone();
    let refresh_user_dictionary = user_dictionary.clone();
    let refresh_dialect = dialect.clone();
    let refresh_integrations = integrations.clone();
    let refresh_linter = linter.clone();

    let lint_text = move |text: &str| {
        let dictionary =
            Config::dictionary_from_user_dictionary(lint_user_dictionary.borrow().clone());
        let doc = Document::new_markdown_default(text, &dictionary);
        let mut organized_lints = lint_linter.borrow_mut().organized_lints(&doc);

        for lints in organized_lints.values_mut() {
            lint_ignored_lints.borrow().remove_ignored(lints, &doc);
        }

        organized_lints
    };

    let ignore_lint = move |lint: &Lint, document: &Document| {
        {
            ignore_ignored_lints
                .borrow_mut()
                .ignore_lint(lint, document);
        }

        let snapshot = ignore_ignored_lints.borrow().clone();
        if let Err(error) =
            ignore_runtime.block_on(ignore_client.borrow_mut().ignore_lint(&snapshot))
        {
            eprintln!("failed to sync ignored lints: {error}");
        }
    };

    let add_to_dictionary = move |word: &str| {
        dictionary_user_dictionary
            .borrow_mut()
            .append_word_str(word, DictWordMetadata::default());

        let lint_config = dictionary_linter.borrow().config.clone();
        let config = Config {
            mutable_dictionary: dictionary_user_dictionary.borrow().clone(),
            dialect: *dictionary_dialect.borrow(),
            ignored_lints: IgnoredLints::new(),
            lint_config,
            integrations: Vec::new(),
        };
        *dictionary_linter.borrow_mut() = config.create_linter();

        if let Err(error) =
            dictionary_runtime.block_on(dictionary_client.borrow_mut().add_to_dictionary(word))
        {
            eprintln!("failed to sync dictionary update: {error}");
        }
    };

    let disable_rule = move |rule_name: &str| match disable_runtime
        .block_on(disable_client.borrow_mut().disable_rule(rule_name))
    {
        Ok(config) => disable_linter.borrow_mut().config = config,
        Err(error) => eprintln!("failed to disable rule {rule_name}: {error}"),
    };

    let refresh_config = move || match fetch_highlighter_config(
        &mut refresh_client.borrow_mut(),
        &refresh_runtime,
    ) {
        Ok(config) => apply_highlighter_config(
            config,
            &refresh_ignored_lints,
            &refresh_user_dictionary,
            &refresh_dialect,
            &refresh_integrations,
            &refresh_linter,
        ),
        Err(error) => eprintln!("failed to refresh highlighter config: {error}"),
    };

    #[cfg(target_os = "macos")]
    let broker = mac_broker::MacBroker::new(integrations);

    #[cfg(not(target_os = "macos"))]
    let broker = os_broker::NoopBroker;

    if let Err(error) = Highlighter::new(
        broker,
        lint_text,
        ignore_lint,
        add_to_dictionary,
        disable_rule,
        refresh_config,
    )
    .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
    .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}

fn fetch_highlighter_config(
    client: &mut Client<Stdin, Stdout>,
    runtime: &Runtime,
) -> Result<Config, ProtocolError> {
    runtime.block_on(async {
        let dialect = client.get_dialect().await?;
        let mutable_dictionary = client.get_dictionary().await?;
        let ignored_lints = client.get_ignored_lints().await?;
        let lint_config = client.get_lint_config().await?;
        let integrations = client.get_integrations().await?;

        Ok(Config {
            dialect,
            mutable_dictionary,
            ignored_lints,
            lint_config,
            integrations,
        })
    })
}

fn apply_highlighter_config(
    config: Config,
    ignored_lints: &Rc<RefCell<IgnoredLints>>,
    user_dictionary: &Rc<RefCell<MutableDictionary>>,
    dialect: &Rc<RefCell<Dialect>>,
    integrations: &Rc<RefCell<Vec<Integration>>>,
    linter: &Rc<RefCell<LintGroup>>,
) {
    let linter_config = config.create_linter();
    *ignored_lints.borrow_mut() = config.ignored_lints;
    *user_dictionary.borrow_mut() = config.mutable_dictionary;
    *dialect.borrow_mut() = config.dialect;
    *integrations.borrow_mut() = config.integrations;
    *linter.borrow_mut() = linter_config;
}
