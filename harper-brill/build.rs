use serde_json::to_string_pretty;
use std::env;
use std::fs;
use std::path::Path;

use harper_pos_utils::FreqDictBuilder;

fn main() {
    let mut freq_dict_builder = FreqDictBuilder::new();
    freq_dict_builder.inc_from_conllu_file("./en_gum-ud-train.conllu");

    let freq_dict = freq_dict_builder.build();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("freq_dict.json");

    fs::write(dest, to_string_pretty(&freq_dict).unwrap()).unwrap();
}
