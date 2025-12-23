#![warn(clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const THESAURUS_PATH: &str = "thesaurus.txt";
const DICT_PATH: &str = "../harper-core/dictionary.dict";

fn main() {
    println!("cargo::rerun-if-changed={DICT_PATH}");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compressed-thesaurus.zst");

    let in_file = File::open(THESAURUS_PATH).expect("Thesaurus file exists");
    let out_file = File::create(dest_path).expect("Can create output file");
    let reader = BufReader::new(in_file);
    let writer = BufWriter::new(out_file);

    // Produce compressed thesaurus.
    zstd::stream::copy_encode(reader, writer, zstd::zstd_safe::max_c_level())
        .expect("Able to write compressed thesaurus");
}
