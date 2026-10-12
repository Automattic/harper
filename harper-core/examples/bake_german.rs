//! Writes the expanded German dictionary to a pack file that start-up reads instead of expanding the word list.
//!
//! Run with: `cargo run --example bake_german -p harper-core --release --features de -- <file>`
//! or `just language-bake german <file>`.

use harper_core::language::german::spell::german_dict::bake_german_pack;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: bake_german <output file>");
    let pack = bake_german_pack();
    std::fs::write(&path, &pack).expect("the pack can be written");
    eprintln!("wrote {} ({} bytes)", path, pack.len());
}
