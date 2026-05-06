use self::highlighter::Highlighter;
use self::highlighter_process::HighlighterProcess;
use crate::communication::Client;
use crate::config::Config;
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, DictWordMetadata, Document, IgnoredLints,
    linting::{FlatConfig, Lint, LintGroup},
    spell::{FstDictionary, MergedDictionary, MutableDictionary},
};
use std::{cell::RefCell, rc::Rc, sync::Arc, thread, time::Duration};
use tauri::{
    Manager, State, WebviewUrl, WebviewWindowBuilder,
    image::Image,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tokio::{runtime::Builder, sync::Mutex};

pub mod color;
pub mod communication;
pub mod config;
pub mod highlighter;
pub mod highlighter_process;
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
    let server_config = config.clone();

    thread::spawn(move || {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on((async move || {
            let mut highlighter_process =
                HighlighterProcess::spawn().expect("failed to spawn highlighter process");

            let mut server = highlighter_process
                .create_server(server_config)
                .expect("failed to create server");

            loop {
                match server.receive_request().await {
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(error) => eprintln!("failed to receive highlighter request: {error}"),
                }
            }
        })())
    });

    tauri::Builder::default()
        .manage(config)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_lint_config,
            set_lint_config,
            ignore_lint,
            add_to_dictionary,
        ])
        .setup(|app| {
            let tray = TrayIconBuilder::with_id("harper-menu-bar")
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
            let tray = tray.icon_as_template(true);

            tray.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_highlighter() {
    #[cfg(target_os = "macos")]
    let broker = mac_broker::MacBroker::new();

    #[cfg(not(target_os = "macos"))]
    let broker = os_broker::NoopBroker;

    let client = Rc::new(RefCell::new(Client::current_process()));
    let sync_runtime = Rc::new(
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build highlighter protocol runtime"),
    );
    let ignored_lints = Rc::new(RefCell::new(IgnoredLints::new()));
    let user_dictionary = Rc::new(RefCell::new(MutableDictionary::new()));
    let linter = Rc::new(RefCell::new(create_linter(create_dictionary(
        user_dictionary.borrow().clone(),
    ))));

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

    let disable_client = client.clone();
    let disable_runtime = sync_runtime.clone();
    let disable_linter = linter.clone();

    let lint_text = move |text: &str| {
        let dictionary = create_dictionary(lint_user_dictionary.borrow().clone());
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
        *dictionary_linter.borrow_mut() = create_linter(create_dictionary(
            dictionary_user_dictionary.borrow().clone(),
        ))
        .with_lint_config(lint_config);

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

    if let Err(error) = Highlighter::new(
        broker,
        lint_text,
        ignore_lint,
        add_to_dictionary,
        disable_rule,
    )
    .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
    .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}

fn create_dictionary(user_dictionary: MutableDictionary) -> Arc<MergedDictionary> {
    let mut dictionary = MergedDictionary::new();
    dictionary.add_dictionary(FstDictionary::curated());
    dictionary.add_dictionary(Arc::new(user_dictionary));

    Arc::new(dictionary)
}

fn create_linter(dictionary: Arc<MergedDictionary>) -> LintGroup {
    LintGroup::new_curated(dictionary, Dialect::American)
}
