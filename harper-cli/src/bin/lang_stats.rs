use clap::Parser;

/// Harper Language Statistics Tool
/// Analyzes dictionary size, annotation coverage, and other metrics.
///
/// The per-language statistics logic lives in each language module's
/// `stats` module (e.g. `harper_core::language::german::stats`), keeping
/// this binary a thin, language-agnostic dispatcher.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Language to analyze (english, german, portuguese - depending on enabled features)
    #[arg(required = true)]
    language: String,

    /// Show detailed annotation breakdown
    #[arg(short, long, default_value_t = false)]
    detailed: bool,
}

fn main() {
    let args = Args::parse();

    match args.language.as_str() {
        "english" => harper_core::language::english::stats::analyze(args.detailed),
        #[cfg(feature = "de")]
        "german" => harper_core::language::german::stats::analyze(args.detailed),
        #[cfg(feature = "pt")]
        "portuguese" => harper_core::language::portuguese::stats::analyze(args.detailed),
        _ => eprintln!("Unknown language: {}", args.language),
    }
}
