use self::highlighter::Highlighter;
use self::highlighter_process::HighlighterProcess;
use crate::communication::Client;
use crate::config::Config;
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, DictWordMetadata, Document, IgnoredLints,
    linting::{FlatConfig, Lint, LintGroup, Linter},
    spell::{FstDictionary, MergedDictionary, MutableDictionary},
};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::State;
use tokio::runtime::Builder;

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

#[tauri::command]
fn get_lint_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<FlatConfig, String> {
    Ok(config
        .lock()
        .map_err(|error| error.to_string())?
        .lint_config
        .clone())
}

#[tauri::command]
fn ignore_lint(ignored_lints: String, config: State<'_, Arc<Mutex<Config>>>) -> Result<(), String> {
    let ignored_lints =
        serde_json::from_str::<IgnoredLints>(&ignored_lints).map_err(|error| error.to_string())?;

    config
        .lock()
        .map_err(|error| error.to_string())?
        .ignored_lints
        .append(ignored_lints);

    Ok(())
}

#[tauri::command]
fn add_to_dictionary(word: String, config: State<'_, Arc<Mutex<Config>>>) -> Result<(), String> {
    config
        .lock()
        .map_err(|error| error.to_string())?
        .mutable_dictionary
        .append_word_str(&word, DictWordMetadata::default());

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
    let config = Arc::new(Mutex::new(Config::new()));
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
            ignore_lint,
            add_to_dictionary,
        ])
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
    let linter = Rc::new(RefCell::new(build_highlighter_linter(
        &user_dictionary.borrow(),
    )));

    let lint_ignored_lints = ignored_lints.clone();
    let lint_linter = linter.clone();

    let ignore_client = client.clone();
    let ignore_runtime = sync_runtime.clone();
    let ignore_ignored_lints = ignored_lints.clone();

    let dictionary_client = client.clone();
    let dictionary_runtime = sync_runtime.clone();
    let dictionary_user_dictionary = user_dictionary.clone();
    let dictionary_linter = linter.clone();

    let lint_text = move |text: &str| {
        let doc = Document::new_markdown_default_curated(text);
        let mut lints = lint_linter.borrow_mut().lint(&doc);

        lint_ignored_lints.borrow().remove_ignored(&mut lints, &doc);

        lints
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

        *dictionary_linter.borrow_mut() =
            build_highlighter_linter(&dictionary_user_dictionary.borrow());

        if let Err(error) =
            dictionary_runtime.block_on(dictionary_client.borrow_mut().add_to_dictionary(word))
        {
            eprintln!("failed to sync dictionary update: {error}");
        }
    };

    if let Err(error) = Highlighter::new(broker, lint_text, ignore_lint, add_to_dictionary)
        .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
        .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}

fn build_highlighter_linter(user_dictionary: &MutableDictionary) -> LintGroup {
    let mut dictionary = MergedDictionary::new();
    dictionary.add_dictionary(FstDictionary::curated());
    dictionary.add_dictionary(Arc::new(user_dictionary.clone()));

    LintGroup::new_curated(Arc::new(dictionary), Dialect::American)
}
