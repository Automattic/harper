use clap::Parser;

/// Harper Language Statistics Tool
/// Analyzes dictionary size, annotation coverage, and other metrics.
///
/// The per-language statistics logic lives in each language module's
/// `stats` module (e.g. `harper_core::language::german::stats`). The dispatch
/// is generated from the languages present in harper-core, so adding a
/// language with a `stats.rs` needs no change here.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Language directory to analyze, e.g. `english` or `german`
    #[arg(required = true)]
    language: String,

    /// Show detailed annotation breakdown
    #[arg(short, long, default_value_t = false)]
    detailed: bool,
}

fn main() {
    let args = Args::parse();

    if !harper_core::language::language_stats(&args.language, args.detailed) {
        eprintln!(
            "No statistics for {:?}. This build has: {}",
            args.language,
            harper_core::language::languages_with_stats().join(", ")
        );
        std::process::exit(1);
    }
}
