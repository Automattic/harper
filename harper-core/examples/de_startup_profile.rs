//! Times each step `harper-ls` takes before the first German diagnostics appear.
//!
//! Run with: `cargo run --example de_startup_profile -p harper-core --release --features de`

use std::sync::Arc;
use std::time::Instant;

use harper_core::language::{Language, detect_language, dictionary, new_curated_for_language};
use harper_core::linting::Linter;
use harper_core::spell::{FstDictionary, MergedDictionary, MutableDictionary};
use harper_core::{Document, parsers::Markdown, parsers::MarkdownOptions};

fn lap<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let out = f();
    eprintln!("{label:<40} {:>8.2?}", t.elapsed());
    out
}

fn main() {
    let text = "Das ist ein Satz mit einem Fehler. Die Katze sitzt auf dem Tisch und der Hund schläft im Garten. Er haben gestern einen Apfel gegessen. ".repeat(3);
    let lang = harper_core::language::parse_language("de").expect("de");
    let english = lap("FstDictionary::curated() (English)", FstDictionary::curated);
    lap("detect_language", || {
        detect_language(&text, &*english, Language::default())
    });
    let de = lap("dictionary(German) first use", || dictionary(lang));
    let mut merged = MergedDictionary::new();
    lap("add_dictionary(German)  [hashes]", || {
        merged.add_dictionary(de.clone())
    });
    lap("add_dictionary(user, empty)", || {
        merged.add_dictionary(Arc::new(MutableDictionary::new()))
    });
    let mut again = MergedDictionary::new();
    lap("add_dictionary(German) 2nd time", || {
        again.add_dictionary(de.clone())
    });
    let merged = Arc::new(merged);
    let mut group = lap("new_curated_for_language", || {
        new_curated_for_language(merged.clone(), lang)
    });
    let source: Vec<char> = text.chars().collect();
    let doc = lap("Document::new (markdown)", || {
        Document::new(&text, &Markdown::new(MarkdownOptions::default()), &*merged)
    });
    let _ = source;
    let n = lap("lint", || group.lint(&doc).len());
    lap("lint (2nd)", || group.lint(&doc).len());
    eprintln!("{n} lints");
}
