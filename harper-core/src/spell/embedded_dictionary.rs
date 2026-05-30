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

pub fn fst_dictionary_from_word_list(content: &str, additions: &[&str]) -> FstDictionary {
    let words: Vec<(CharString, DictWordMetadata)> = content
        .lines()
        .chain(additions.iter().copied())
        .filter_map(parse_word_line)
        .map(|word| {
            let chars: CharString = word.chars().collect();
            (chars, DictWordMetadata::default())
        })
        .collect();

    FstDictionary::new(words)
}

pub fn fst_dictionary_from_gzip_bytes(compressed: &[u8], additions: &[&str]) -> FstDictionary {
    let mut decoder = GzDecoder::new(compressed);
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .expect("Failed to decompress embedded dictionary");

    fst_dictionary_from_word_list(&text, additions)
}
