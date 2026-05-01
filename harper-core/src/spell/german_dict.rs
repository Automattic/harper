//! German dictionary based on the LanguageTool/igerman98 Hunspell word lists.
//!
//! The word list is derived from the igerman98 dictionary (GPLv2/GPLv3),
//! expanded using Hunspell affix rules for comprehensive coverage.
//! It is embedded as gzip-compressed data and decompressed once at first use.

use std::io::Read;
use std::sync::{Arc, LazyLock};

use flate2::read::GzDecoder;

use crate::spell::fst_dictionary::FstDictionary;
use crate::{CharString, DictWordMetadata};

static GERMAN_DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(|| {
    let compressed = include_bytes!("../../german_dictionary.dict.gz");
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .expect("Failed to decompress German dictionary");

    let words: Vec<(CharString, DictWordMetadata)> = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(|word| {
            let chars: CharString = word.chars().collect();
            (chars, DictWordMetadata::default())
        })
        .collect();

    Arc::new(FstDictionary::new(words))
});

/// Returns a shared reference to the German FstDictionary.
///
/// The dictionary is loaded and built once on first access, then cached for the
/// lifetime of the process. This provides fuzzy matching, prefix search, and
/// all other `Dictionary` trait capabilities.
pub fn curated_german_dictionary() -> Arc<FstDictionary> {
    (*GERMAN_DICT).clone()
}
