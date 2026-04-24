use harper_core::spell::{Dictionary, FstDictionary};
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;

// Pre-built word list and dictionary handle for the WASM bench harness —
// keeps parse and dictionary lookup out of the timed region.
#[wasm_bindgen]
pub struct PreparedWords {
    words: Vec<Vec<char>>,
    dict: Arc<FstDictionary>,
}

#[wasm_bindgen]
impl PreparedWords {
    #[wasm_bindgen(constructor)]
    pub fn new(words: &str) -> PreparedWords {
        PreparedWords {
            words: words
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.chars().collect())
                .collect(),
            dict: FstDictionary::curated(),
        }
    }

    // Runs fuzzy_match on each word. Returns the total result count so the
    // optimizer cannot discard the work.
    pub fn bench_fuzzy_match(&self, max_edit_distance: u8, max_results: usize) -> usize {
        let mut total = 0usize;
        for word in &self.words {
            total += self
                .dict
                .fuzzy_match(word, max_edit_distance, max_results)
                .len();
        }
        total
    }
}
