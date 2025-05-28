use harper_pos_utils::UPOS;
use serde_json::to_string_pretty;
use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;

use harper_pos_utils::FreqDictBuilder;

use rs_conllu::parse_file;

fn main() {
    let file = File::open("./en_gum-ud-train.conllu").unwrap();
    let doc = parse_file(file);

    let mut freq_dict_builder = FreqDictBuilder::new();

    for res in doc {
        match res {
            Ok(s) => {
                for token in s.tokens {
                    if let Some(upos) = token.upos.and_then(UPOS::from_conllu) {
                        freq_dict_builder.inc(&token.form, &upos)
                    }
                }
            }
            Err(err) => panic!("Fail: {err}"),
        }
    }

    let freq_dict = freq_dict_builder.build();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("freq_dict.json");

    fs::write(dest, to_string_pretty(&freq_dict).unwrap()).unwrap();
}
