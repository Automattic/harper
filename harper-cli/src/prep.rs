use harper_core::{
    TokenStringExt,
    expr::{ExprExt, SequenceExpr},
    parsers::MarkdownOptions,
    patterns::WordSet,
    spell::FstDictionary,
};

use crate::input::{
    AnyInput, InputTrait,
    multi_input::MultiInput,
    single_input::{SingleInput, SingleInputTrait, StdinInput},
};

pub fn prep(
    inputs: Vec<AnyInput>,
    before_preps: Vec<String>,
    words: Vec<String>,
    after_preps: Vec<String>,
) -> anyhow::Result<()> {
    let mut seq = SequenceExpr::default();

    seq = match before_preps.as_slice() {
        [p] if p == "P" => seq.then_preposition().t_ws(),
        [] => seq,
        preps => {
            let mut ws = WordSet::new(&[]);
            for w in preps {
                ws.add(w);
            }
            seq.then(ws).t_ws()
        }
    };

    seq = if words.is_empty() {
        seq.then_any_word()
    } else {
        let mut ws = WordSet::new(&[]);
        for w in &words {
            ws.add(w);
        }
        seq.then(ws)
    };

    seq = match after_preps.as_slice() {
        [p] if p == "P" => seq.t_ws().then_preposition(),
        [] => seq,
        preps => {
            let mut ws = WordSet::new(&[]);
            for w in preps {
                ws.add(w);
            }
            seq.t_ws().then(ws)
        }
    };

    let inputs = if inputs.is_empty() {
        vec![SingleInput::from(StdinInput).into()]
    } else {
        inputs
    };

    let mut expanded = Vec::new();
    for input in inputs {
        if let Some(dir) = input
            .try_as_multi_ref()
            .and_then(MultiInput::try_as_dir_ref)
        {
            let mut files: Vec<_> = dir.iter_files()?.collect();
            files.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));
            for file in files {
                expanded.push(SingleInput::from(file).into());
            }
        } else {
            expanded.push(input);
        }
    }

    for input in expanded {
        println!("Identifier: {}", input.get_identifier());

        if let Some(single) = input.try_as_single_ref() {
            match single.load(MarkdownOptions::default(), &FstDictionary::curated()) {
                Ok((doc, _)) => {
                    for chunk in doc.iter_chunks() {
                        for mat in seq.iter_matches(chunk, doc.get_source()) {
                            let pre = &chunk[0..mat.start];
                            let toks = &chunk[mat.start..mat.end];
                            let suf = &chunk[mat.end..];

                            // toks len is 1, 3, or 5
                            // for 1 word, 2 words, 3 words
                            // between word tokens is a whitespace token
                            // words are even indeces, given them colours
                            // spaces are odd indices, don't give them colours
                            let coloured = toks
                                .iter()
                                .enumerate()
                                .map(|(i, t)| {
                                    if i % 2 == 0 {
                                        format!(
                                            "\x1b[3{}m{}\x1b[0m",
                                            i / 2 + 1,
                                            t.span.get_content_string(doc.get_source())
                                        )
                                    } else {
                                        t.span.get_content_string(doc.get_source())
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("");

                            // 1 2 4 = r g b; 3 = y; 5 6 = m c
                            // black r g y b m c white
                            println!(
                                "\x1b[34m{}\x1b[0m{}\x1b[34m{}\x1b[0m",
                                pre.span()
                                    .map(|s| s.get_content_string(doc.get_source()))
                                    .unwrap_or("".to_string()),
                                coloured,
                                suf.span()
                                    .map(|s| s.get_content_string(doc.get_source()))
                                    .unwrap_or("".to_string())
                            );
                        }
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    Ok(())
}
