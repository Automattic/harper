use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    Highlighter {
        #[arg(long)]
        no_parent: bool,
    },
    // Runs the application only showing the popup window for an example suggestion. Useful for
    // making visual modifications to the popup window.
    PopupDebug,
}
