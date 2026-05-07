use self::highlighter::Highlighter;
use self::highlighter_service::HighlighterService;
use crate::communication::{Client, ProtocolError};
use crate::config::Config;
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, DictWordMetadata, Document, IgnoredLints,
    linting::{FlatConfig, Lint, LintGroup},
    spell::MutableDictionary,
};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tauri::{
    Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
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
const OPEN_EDITOR_MENU_ID: &str = "open-editor";
const SETTINGS_MENU_ID: &str = "settings";
const QUIT_MENU_ID: &str = "quit";

fn menu_bar_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(include_bytes!("../icons/menu-bar-icon.png")).map(Image::to_owned)
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

fn tray_menu(app: &tauri::App) -> tauri::Result<Menu<tauri::Wry>> {
    let open_editor =
        MenuItem::with_id(app, OPEN_EDITOR_MENU_ID, "Open Editor", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, SETTINGS_MENU_ID, "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&open_editor, &separator, &settings, &quit])
}

#[tauri::command]
async fn get_lint_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<FlatConfig, String> {
    Ok(config.lock().await.lint_config.clone())
}

#[tauri::command]
async fn set_lint_config(
    lint_config: FlatConfig,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.lint_config = lint_config;
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_lint_config,
            set_lint_config,
            ignore_lint,
            add_to_dictionary,
        ])
        .on_window_event(|window, event| {
            if window.label() == EDITOR_WINDOW_LABEL {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();

                    if let Err(error) = window.hide() {
                        eprintln!("failed to hide editor window: {error}");
                    }
                }
            }
        })
        .setup(|app| {
            let menu = tray_menu(app)?;
            let tray = TrayIconBuilder::with_id("harper-menu-bar")
                .menu(&menu)
                .icon(menu_bar_icon()?)
                .tooltip("Harper Desktop")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
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

            #[cfg(target_os = "macos")]
            let tray = tray.icon_as_template(true);

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
            &refresh_linter,
        ),
        Err(error) => eprintln!("failed to refresh highlighter config: {error}"),
    };

    #[cfg(target_os = "macos")]
    let broker = mac_broker::MacBroker::new();

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

        Ok(Config {
            dialect,
            mutable_dictionary,
            ignored_lints,
            lint_config,
        })
    })
}

fn apply_highlighter_config(
    config: Config,
    ignored_lints: &Rc<RefCell<IgnoredLints>>,
    user_dictionary: &Rc<RefCell<MutableDictionary>>,
    dialect: &Rc<RefCell<Dialect>>,
    linter: &Rc<RefCell<LintGroup>>,
) {
    let linter_config = config.create_linter();
    *ignored_lints.borrow_mut() = config.ignored_lints;
    *user_dictionary.borrow_mut() = config.mutable_dictionary;
    *dialect.borrow_mut() = config.dialect;
    *linter.borrow_mut() = linter_config;
}
