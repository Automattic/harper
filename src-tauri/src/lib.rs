use self::highlighter::Highlighter;
use clap::{Parser, Subcommand};
use std::time::Duration;

pub mod color;
pub mod highlighter;
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

    if let Err(error) = Highlighter::new(broker)
        .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
        .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}
