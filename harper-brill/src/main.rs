use std::fs::File;

use anyhow::anyhow;
use freq_dict::FreqDictBuilder;
use rs_conllu::parse_file;
use upos::UPOS;

mod freq_dict;
mod upos;

fn main() -> anyhow::Result<()> {
    let file = File::open("./en_gum-ud-train.conllu")?;
    let doc = parse_file(file);

    let mut freq_dict_builder = FreqDictBuilder::new();

    for res in doc {
        match res {
            Ok(s) => {
                for token in s.tokens {
                    if let Some(upos) = token.upos.map(UPOS::from_conllu).flatten() {
                        freq_dict_builder.inc(&token.form, &upos)
                    }
                }
            }
            Err(err) => return Err(anyhow!("Fail: {err}")),
        }
    }

    let mut table = tabled::builder::Builder::default();

    for (key, value) in freq_dict_builder.build().mapping {
        table.push_record([key, value.as_ref().to_string()]);
    }

    println!("{}", table.build());

    Ok(())
}
