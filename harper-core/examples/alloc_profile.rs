//! Counts heap allocations made by `fuzzy_match` across the benchmark word lists.
//! Allocation is relatively more expensive in WASM than on native, so reducing
//! allocs/word helps spell-check latency on lower-end devices.
//!
//! Run with: `cargo run --example alloc_profile -p harper-core --release`

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use harper_core::spell::{Dictionary, FstDictionary};

struct CountingAllocator {
    alloc_count: AtomicUsize,
    dealloc_count: AtomicUsize,
}

impl CountingAllocator {
    fn reset(&self) {
        self.alloc_count.store(0, Ordering::Relaxed);
        self.dealloc_count.store(0, Ordering::Relaxed);
    }

    fn alloc_count(&self) -> usize {
        self.alloc_count.load(Ordering::Relaxed)
    }

    fn dealloc_count(&self) -> usize {
        self.dealloc_count.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.alloc_count.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.dealloc_count.fetch_add(1, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator {
    alloc_count: AtomicUsize::new(0),
    dealloc_count: AtomicUsize::new(0),
};

// Values duplicated across the WASM harness (harper-wasm/benches/wasm_bench.js)
// and the criterion bench (benches/spellcheck.rs) so numbers stay comparable.
const MAX_EDIT_DISTANCE: u8 = 3;
const MAX_RESULTS: usize = 200;

// Shared with the criterion bench (../benches/spellcheck.rs) to keep numbers comparable.
static MISSPELLED_MIXED: &str = include_str!("../benches/misspelled_words/mixed.md");
static MISSPELLED_LOWERCASE: &str = include_str!("../benches/misspelled_words/lowercase.md");
static MISSPELLED_CAPITALIZED: &str = include_str!("../benches/misspelled_words/capitalized.md");

fn load_word_list(source: &str) -> Vec<Vec<char>> {
    source
        .lines()
        .filter(|l| !l.is_empty())
        .map(|w| w.chars().collect())
        .collect()
}

fn profile_word_list(name: &str, words: &[Vec<char>], dict: &FstDictionary) {
    ALLOC.reset();

    for word in words {
        black_box(dict.fuzzy_match(black_box(word.as_slice()), MAX_EDIT_DISTANCE, MAX_RESULTS));
    }

    let allocs = ALLOC.alloc_count();
    let deallocs = ALLOC.dealloc_count();
    let net = allocs as i64 - deallocs as i64;
    let word_count = words.len();

    println!("{name}:");
    println!("  words:          {word_count}");
    println!("  allocs:         {allocs}");
    println!("  deallocs:       {deallocs}");
    println!("  net:            {net:+}");
    println!("  allocs/word:    {:.1}", allocs as f64 / word_count as f64);
    println!();
}

fn main() {
    // Initialize dictionary before resetting counters so startup allocs are excluded.
    let dict = FstDictionary::curated();
    // Warm the AUTOMATON_BUILDERS thread_local so its init allocs aren't counted
    // against the first measured case.
    let _ = black_box(dict.fuzzy_match(&['w', 'a', 'r', 'm'], MAX_EDIT_DISTANCE, MAX_RESULTS));

    let mixed = load_word_list(MISSPELLED_MIXED);
    let lowercase = load_word_list(MISSPELLED_LOWERCASE);
    let capitalized = load_word_list(MISSPELLED_CAPITALIZED);

    let cases = [
        ("misspelled_mixed", mixed.as_slice()),
        ("misspelled_lowercase", lowercase.as_slice()),
        ("misspelled_capitalized", capitalized.as_slice()),
    ];

    println!("--- fuzzy_match allocation profile ---");
    println!("max_edit_distance: {MAX_EDIT_DISTANCE}, max_results: {MAX_RESULTS}");
    println!();

    for (name, words) in cases {
        profile_word_list(name, words, &*dict);
    }
}
