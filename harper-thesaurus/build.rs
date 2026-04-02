#![warn(clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

const THESAURUS_PATH: &str = "thesaurus.txt";

fn main() {
    println!("cargo::rerun-if-changed={THESAURUS_PATH}");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("compressed-thesaurus.zst");

    let in_file = File::open(THESAURUS_PATH).expect("Thesaurus file exists");
    let out_file = File::create(dest_path).expect("Can create output file");
    let reader = BufReader::new(in_file);
    let writer = BufWriter::new(out_file);

    // Use a lesser compression level to speed up debug builds.
    let compression_level = match env::var("OPT_LEVEL").unwrap().as_str() {
        "3" | "2" | "s" | "z" => zstd::zstd_safe::max_c_level(), // 3.84 MiB
        _ => 4,                                                  // 7.02 MiB
    };

    zstd::stream::copy_encode(reader, writer, compression_level)
        .expect("Able to write compressed thesaurus");
}
