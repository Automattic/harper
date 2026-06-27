use std::io::Read;

use flate2::read::GzDecoder;

use super::FstDictionary;
use crate::{CharString, DictWordMetadata};

fn parse_word_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    Some(trimmed.split('/').next().unwrap_or(trimmed))
}

/// Parse a string to a dictionary
fn fst_dictionary_from_word_list(content: &str) -> FstDictionary {
    let words: Vec<(CharString, DictWordMetadata)> = content
        .lines()
        .filter_map(parse_word_line)
        .map(|word| {
            let chars: CharString = word.chars().collect();
            (chars, DictWordMetadata::default())
        })
        .collect();
    FstDictionary::new(words)
}

/// Extract the dictionary from a gzipped data
pub fn fst_dictionary_from_gzip_bytes(compressed: &[u8]) -> FstDictionary {
    let mut decoder = GzDecoder::new(compressed);
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .expect("Failed to decompress embedded dictionary");

    fst_dictionary_from_word_list(&text)
}
