mod tray;
mod windows;

use self::highlighter::Highlighter;
use self::highlighter_service::HighlighterService;
use self::tray::set_up_tray_menu;
use crate::communication::{Client, ProtocolError};
use crate::config::{Config, Integration};
use crate::debounce::{DebounceState, DebounceStatus};
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, DictWordMetadata, Document, IgnoredLints,
    linting::{Lint, LintGroup},
    spell::MutableDictionary,
};
use serde::Serialize;
use std::io::stderr;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex as StdMutex},
};
use tauri::Manager as _;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::os_broker::{AccessibilityPermissionStatus, OsBroker};
use tokio::{
    io::{Stdin, Stdout},
    runtime::{Builder, Runtime},
    sync::Mutex,
};

pub mod color;
mod commands;
pub mod communication;
pub mod config;
mod debounce;
pub mod highlighter;
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
    Highlighter {
        #[arg(long)]
        no_parent: bool,
    },
}

#[derive(Debug, Clone, Serialize)]
struct IntegrationView {
    bundle_id: String,
    enabled: bool,
    display_name: String,
}

fn accessibility_allows_highlighter_start() -> bool {
    platform_broker().accessibility_permission_status() == AccessibilityPermissionStatus::Granted
}

fn start_highlighter_service_if_enabled_and_permitted(
    highlighter_service: &HighlighterService,
    highlighter_service_enabled: bool,
) {
    if !highlighter_service_enabled || !accessibility_allows_highlighter_start() {
        return;
    }

    if let Err(error) = highlighter_service.start() {
        eprintln!("failed to start highlighter service: {error}");
    }
}

#[cfg(target_os = "macos")]
pub(crate) type PlatformBroker = mac_broker::MacBroker;

#[cfg(not(target_os = "macos"))]
pub(crate) type PlatformBroker = os_broker::NoopBroker;

fn platform_broker() -> PlatformBroker {
    PlatformBroker::default()
}

fn warm_app_search_cache(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let broker = app.state::<StdMutex<PlatformBroker>>();
        let result = broker
            .lock()
            .map_err(|error| format!("failed to read platform broker: {error}"))
            .and_then(|broker| broker.search_apps("").map(|_| ()));

        if let Err(error) = result {
            eprintln!("failed to warm app search cache: {error}");
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let subscriber = FmtSubscriber::builder()
        .map_writer(move |_| stderr)
        .with_ansi(false)
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set up tracing subscriber.");

    let args = Args::parse();

    match args.command {
        Some(Command::Highlighter { no_parent }) => run_highlighter(!no_parent),
        None => run_tauri(),
    }
}

pub fn run_tauri() {
    let config_runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build config runtime");
    let is_first_launch = match Config::main_config_exists() {
        Ok(exists) => !exists,
        Err(error) => {
            eprintln!("failed to check config existence: {error}");
            false
        }
    };

    let config = if is_first_launch {
        let config = Config::new();

        if let Err(error) = config_runtime.block_on(config.save_to_system()) {
            eprintln!("failed to save initial config: {error}");
        }

        config
    } else {
        match config_runtime.block_on(Config::load_from_system()) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("failed to load config, using defaults: {error}");
                Config::new()
            }
        }
    };

    let highlighter_service_enabled = config.highlighter_service_enabled;
    let config = Arc::new(Mutex::new(config));
    let highlighter_service = HighlighterService::new(config.clone());
    start_highlighter_service_if_enabled_and_permitted(
        &highlighter_service,
        highlighter_service_enabled,
    );

    tauri::Builder::default()
        .manage(config)
        .manage(highlighter_service)
        .manage(StdMutex::new(platform_broker()))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(commands::application_message_handler())
        .setup(move |app| {
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            set_up_tray_menu(app.handle())?;

            warm_app_search_cache(app.handle().clone());

            if is_first_launch {
                windows::show_settings_window(app.handle())?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Run as a highlighter process.
/// Can configure whether to run standalone, or with a parent Tauri process
pub fn run_highlighter(has_parent: bool) {
    let client = Rc::new(RefCell::new(Client::current_process()));
    let sync_runtime = Rc::new(
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build highlighter protocol runtime"),
    );

    let startup_config = if has_parent {
        fetch_highlighter_config(&mut client.borrow_mut(), &sync_runtime)
    } else {
        Ok(Config::default())
    };

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
    let integrations = Arc::new(StdMutex::new(startup_config.integrations));
    let debounce_ms = Rc::new(RefCell::new(startup_config.debounce_ms));
    let linter = Rc::new(RefCell::new(startup_linter));

    let lint_ignored_lints = ignored_lints.clone();
    let lint_linter = linter.clone();
    let lint_user_dictionary = user_dictionary.clone();
    let lint_debounce_ms = debounce_ms.clone();
    let lint_debounce_state = Rc::new(RefCell::new(DebounceState::default()));

    let ignore_client = client.clone();
    let ignore_runtime = sync_runtime.clone();
    let ignore_ignored_lints = ignored_lints.clone();

    let dictionary_client = client.clone();
    let dictionary_runtime = sync_runtime.clone();
    let dictionary_user_dictionary = user_dictionary.clone();
    let dictionary_linter = linter.clone();
    let dictionary_dialect = dialect.clone();
    let dictionary_debounce_ms = debounce_ms.clone();

    let disable_client = client.clone();
    let disable_runtime = sync_runtime.clone();
    let disable_linter = linter.clone();

    let refresh_client = client.clone();
    let refresh_runtime = sync_runtime.clone();
    let refresh_ignored_lints = ignored_lints.clone();
    let refresh_user_dictionary = user_dictionary.clone();
    let refresh_dialect = dialect.clone();
    let refresh_integrations = integrations.clone();
    let refresh_debounce_ms = debounce_ms.clone();
    let refresh_linter = linter.clone();

    let lint_text = move |text: &str| {
        let debounce_ms = *lint_debounce_ms.borrow();
        let mut debounce_state = lint_debounce_state.borrow_mut();

        match debounce_state.status(text, debounce_ms) {
            DebounceStatus::Cached(lints) => return lints,
            DebounceStatus::Ready => {}
        }

        let dictionary =
            Config::dictionary_from_user_dictionary(lint_user_dictionary.borrow().clone());
        let doc = Document::new_markdown_default(text, &dictionary);
        let mut organized_lints = lint_linter.borrow_mut().organized_lints(&doc);

        for lints in organized_lints.values_mut() {
            lint_ignored_lints.borrow().remove_ignored(lints, &doc);
        }

        debounce_state.store_lints(text, debounce_ms, &organized_lints);

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
            debounce_ms: *dictionary_debounce_ms.borrow(),
            auto_update: true,
            last_update_check: None,
            highlighter_service_enabled: true,
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

    let refresh_config = move || {
        if !has_parent {
            return;
        }

        match fetch_highlighter_config(&mut refresh_client.borrow_mut(), &refresh_runtime) {
            Ok(config) => apply_highlighter_config(
                config,
                &refresh_ignored_lints,
                &refresh_user_dictionary,
                &refresh_dialect,
                &refresh_integrations,
                &refresh_debounce_ms,
                &refresh_linter,
            ),
            Err(error) => eprintln!("failed to refresh highlighter config: {error}"),
        }
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
        let debounce_ms = client.get_debounce_ms().await?;

        Ok(Config {
            dialect,
            mutable_dictionary,
            ignored_lints,
            lint_config,
            integrations,
            debounce_ms,
            auto_update: true,
            last_update_check: None,
            highlighter_service_enabled: true,
        })
    })
}

fn apply_highlighter_config(
    config: Config,
    ignored_lints: &Rc<RefCell<IgnoredLints>>,
    user_dictionary: &Rc<RefCell<MutableDictionary>>,
    dialect: &Rc<RefCell<Dialect>>,
    integrations: &Arc<StdMutex<Vec<Integration>>>,
    debounce_ms: &Rc<RefCell<u64>>,
    linter: &Rc<RefCell<LintGroup>>,
) {
    let linter_config = config.create_linter();
    *ignored_lints.borrow_mut() = config.ignored_lints;
    *user_dictionary.borrow_mut() = config.mutable_dictionary;
    *dialect.borrow_mut() = config.dialect;
    match integrations.lock() {
        Ok(mut integrations) => *integrations = config.integrations,
        Err(error) => eprintln!("failed to update integrations: {error}"),
    }
    *debounce_ms.borrow_mut() = config.debounce_ms;
    *linter.borrow_mut() = linter_config;
}
