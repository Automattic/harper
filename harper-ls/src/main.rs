#![doc = include_str!("../README.md")]

use std::io::stderr;

use config::Config;
use tokio::net::TcpListener;
mod backend;
mod config;
mod diagnostics;
mod dictionary_io;
mod document_state;
mod git_commit_parser;
mod ignored_lints_io;
mod io_utils;
mod pos_conv;

use backend::Backend;
use clap::Parser;
use tower_lsp_server::{LspService, Server};
use tracing::{Level, error, info};
use tracing_subscriber::FmtSubscriber;

static DEFAULT_ADDRESS: &str = "127.0.0.1:4000";

/// Start a language server to provide grammar checking inside of developer
/// environments.
///
/// Will listen on 127.0.0.1:4000 by default.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Set to listen on standard input / output rather than TCP.
    #[arg(short, long, default_value_t = false)]
    stdio: bool,
    /// Skip the debug version check.
    #[arg(long, default_value_t = false)]
    skip_version_check: bool,
}

// Setting worker threads to four means the process will use about five threads total
// This is because worker threads do not include blocking threads
#[tokio::main(worker_threads = 1)]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .map_writer(move |_| stderr)
        .with_ansi(false)
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let config = Config::default();

    if !args.skip_version_check {
        tokio::spawn(log_version_info());
    }

    let (service, socket) = LspService::new(|client| Backend::new(client, config));

    if args.stdio {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        let listener = TcpListener::bind(DEFAULT_ADDRESS).await.unwrap();
        println!("Listening on {DEFAULT_ADDRESS}");
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        Server::new(read, write, socket).serve(service).await;
    }

    Ok(())
}

/// Ping Harper's website to get the latest available version.
async fn get_latest_version() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();

    client
        .get("https://writewithharper.com/latestversion")
        .header("Harper-Version", harper_core::core_version())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await
}

/// Log the current version information to the console.
async fn log_version_info() {
    match get_latest_version().await {
        Ok(version) => info!("Latest available Harper version: {}", version),
        Err(_err) => error!("Unable to obtain latest version."),
    }

    info!(
        "Current harper-core version: {}",
        harper_core::core_version()
    );
    info!("Current harper-ls version: {}", backend::ls_version());
}
