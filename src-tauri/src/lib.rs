use self::highlighter::Highlighter;
use self::highlighter_process::HighlighterProcess;
use clap::{Parser, Subcommand};
use harper_core::{
    Dialect, Document,
    linting::{LintGroup, Linter},
    spell::FstDictionary,
};
use std::{thread, time::Duration};
use tokio::runtime::Builder;

pub mod color;
pub mod communication;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args = Args::parse();

    match args.command {
        Some(Command::Highlighter) => run_highlighter(),
        None => run_tauri(),
    }
}

pub fn run_tauri() {
    thread::spawn(move || {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on((async move || {
            let mut highlighter_process =
                HighlighterProcess::spawn().expect("failed to spawn highlighter process");

            let mut server = highlighter_process
                .create_server()
                .expect("failed to create server");

            loop {
                server.receive_request().await.unwrap();
            }
        })())
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn run_highlighter() {
    #[cfg(target_os = "macos")]
    let broker = mac_broker::MacBroker::new();

    #[cfg(not(target_os = "macos"))]
    let broker = os_broker::NoopBroker;

    let mut linter = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    let lint_text = move |text: &str| {
        let doc = Document::new_markdown_default_curated(text);

        linter.lint(&doc)
    };

    if let Err(error) = Highlighter::new(broker, lint_text)
        .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
        .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}
