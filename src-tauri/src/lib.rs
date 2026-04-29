use self::highlighter::Highlighter;
use clap::{Parser, Subcommand};
use std::time::Duration;

pub mod highlighter;
pub mod rect;

use rect::Rect;

#[cfg(target_os = "macos")]
mod macos;

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
    if let Err(error) = Highlighter::new(|| {
        #[cfg(target_os = "macos")]
        return Some(macos::get_boxes());

        None
    })
    .map(|highlighter| highlighter.with_read_interval(Duration::from_millis(16)))
    .and_then(Highlighter::run_window_for_each_monitor)
    {
        eprintln!("failed to run highlighter: {error}");
    }
}
