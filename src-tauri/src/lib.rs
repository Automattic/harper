use self::highlighter::Highlighter;
use clap::{Parser, Subcommand};

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
    if let Err(error) = Highlighter::new().and_then(|mut h| {
        h.set_rects(vec![Rect::new(100., 100., 100., 100.)]);
        h.run_window_for_each_monitor()
    }) {
        eprintln!("failed to run highlighter: {error}");
    }
}
