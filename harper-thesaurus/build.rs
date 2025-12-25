#![warn(clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use hashbrown::HashSet;

const THESAURUS_PATH: &str = "thesaurus.txt";
const DICT_PATH: &str = "../harper-core/dictionary.dict";

fn main() {
    println!("cargo::rerun-if-changed={THESAURUS_PATH}");
    println!("cargo::rerun-if-changed={DICT_PATH}");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compressed-thesaurus.zst");

    let in_file = File::open(THESAURUS_PATH).expect("Thesaurus file exists");
    let out_file = File::create(dest_path).expect("Can create output file");
    let reader = BufReader::new(in_file);
    let writer = BufWriter::new(out_file);

    if let Ok(dict_words) = get_dict_words() {
        // Remove entries for words that aren't in the curated dictionary, then compress.
        let mut compressed_writer = zstd::Encoder::new(writer, zstd::zstd_safe::max_c_level())
            .unwrap()
            .auto_finish();

        for entry in reader.split(b'\n').filter_map(Result::ok) {
            let word = entry
                .split(|c| *c == b',')
                .next()
                .expect("All entries are words followed by a comma");
            if dict_words.contains(str::from_utf8(word).unwrap()) {
                compressed_writer.write_all(&entry).unwrap();
                compressed_writer.write_all(b"\n").unwrap();
            }
        }
    } else {
        // Compress without any filtering.
        zstd::stream::copy_encode(reader, writer, zstd::zstd_safe::max_c_level())
            .expect("Able to write compressed thesaurus");
    }
}

/// Get all unique words contained in the curated dictionary.
///
/// This will fail if the curated dictionary could not be read.
fn get_dict_words() -> Result<HashSet<String>, std::io::Error> {
    let dict_reader = BufReader::new(File::open(DICT_PATH)?);
    let mut out_words = HashSet::new();
    for word in dict_reader.lines().filter_map(|line| {
        if let Ok(line) = line
            && let Some(word) = line.split('/').next()
        {
            Some(word.to_owned())
        } else {
            None
        }
    }) {
        out_words.insert(word);
    }
    Ok(out_words)
}
