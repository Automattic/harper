use harper_core::spell::{Dictionary, FstDictionary};
use wasm_bindgen::prelude::wasm_bindgen;

/// Runs `FstDictionary::fuzzy_match` on each word in a newline-separated list.
///
/// Returns the total number of results across all words (prevents the optimizer
/// from discarding work).
#[wasm_bindgen]
pub fn bench_fuzzy_match(words: &str, max_edit_distance: u8, max_results: usize) -> usize {
    let dict = FstDictionary::curated();
    let mut total = 0usize;
    for line in words.lines() {
        if line.is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        total += dict
            .fuzzy_match(&chars, max_edit_distance, max_results)
            .len();
    }
    total
}
