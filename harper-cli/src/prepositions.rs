use harper_core::{
    TokenStringExt,
    expr::{ExprExt, SequenceExpr},
    parsers::MarkdownOptions,
    patterns::WordSet,
    spell::FstDictionary,
};

use crate::input::{AnyInput, InputTrait, single_input::SingleInputTrait};
use crate::input_helpers::process_inputs_collect;

/// Scan documents for preposition collocations.
///
/// This dev tool helps analyze how words are used with prepositions by finding
/// patterns like "preposition + word" (e.g., "based on") or "word + preposition"
/// (e.g., "rely on"). Useful for linguistic research and understanding
/// prepositional usage patterns in corpora.
///
/// # Arguments
///
/// * `inputs` - Files, directories, or text to analyze (defaults to stdin if empty)
/// * `before_preps` - Prepositions to match before the target word.
///   Use `["P"]` to match any preposition, or provide specific prepositions
///   like `["in", "on", "at"]`.
/// * `words` - Target words to find collocations for. If empty, matches any word.
/// * `after_preps` - Prepositions to match after the target word.
///   Same format as `before_preps`.
///
/// # Examples
///
/// Find verbs followed by any preposition:
/// ```bash
/// harper-cli prepositions -b P --words rely,depend -- ./corpus/
/// ```
///
/// Find nouns preceded by specific prepositions:
/// ```bash
/// harper-cli prepositions -a in,on --words house,car -- ./text.txt
/// ```
///
/// Find full prepositional phrases:
/// ```bash
/// harper-cli prepositions -b P -a P -- ./document.md
/// ```
pub fn prepositions(
    inputs: Vec<AnyInput>,
    before_preps: Vec<String>,
    words: Vec<String>,
    after_preps: Vec<String>,
    parallel: bool,
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

    let results = process_inputs_collect(inputs, parallel, |expanded_input| {
        let input = expanded_input.input;
        if let Some(single) = input.try_as_single_ref() {
            match single.load(MarkdownOptions::default(), &FstDictionary::curated()) {
                Ok((doc, _)) => {
                    let src = doc.get_source();
                    for chunk in doc.iter_chunks() {
                        for mat in seq.iter_matches(chunk, src) {
                            let pre = &chunk[0..mat.start];
                            let toks = &chunk[mat.start..mat.end];
                            let suf = &chunk[mat.end..];

                            // ANSI guide
                            //     0 1 2 3 4 5 6 7
                            // black r g y b m c w

                            println!(
                                "\x1b[35m{}\x1b[0m: \x1b[34m{}\x1b[0m{}\x1b[34m{}\x1b[0m",
                                input.get_identifier(),
                                pre.span()
                                    .map(|s| s.get_content_string(src))
                                    .unwrap_or("".to_string()),
                                toks.iter()
                                    .enumerate()
                                    .map(|(i, t)| {
                                        if i % 2 == 0 {
                                            format!("\x1b[3{}m{}\x1b[0m", i / 2 + 1, t.get_str(src))
                                        } else {
                                            t.get_str(src)
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(""),
                                suf.span()
                                    .map(|s| s.get_content_string(src))
                                    .unwrap_or("".to_string())
                            );
                        }
                    }
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        Ok(())
    });

    // Check for any expansion errors
    for result in results {
        if let Err(e) = result {
            eprintln!("{}", e);
        }
    }

    Ok(())
}
