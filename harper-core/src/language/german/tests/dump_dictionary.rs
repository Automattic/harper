//! Dumps the expanded German dictionary with its metadata, for the scripts
//! that compare one build of `dictionary.dict` with another.
//!
//! ```bash
//! HARPER_DUMP_GERMAN_DICT=/tmp/de.tsv cargo test -p harper-core \
//!     --features multilingual --lib dump_german_dictionary -- --ignored
//! ```
//!
//! One line per word form: the spelling, a tab, the metadata as JSON.
//!
//! `probe_german_dictionary` asks the dictionary Harper actually spell-checks
//! with, compound acceptance included, about the words in a file:
//!
//! ```bash
//! HARPER_PROBE_IN=words.txt HARPER_PROBE_OUT=probe.tsv cargo test -p harper-core \
//!     --features multilingual --lib probe_german_dictionary -- --ignored
//! ```
//!
//! One line per input word: the word, a tab, and the metadata as JSON (empty
//! when the word is not accepted).

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::language::german::spell::curated_german_dictionary;
    use crate::spell::Dictionary;

    #[test]
    #[ignore = "writes a large file; run on demand"]
    fn dump_german_dictionary() {
        let Ok(path) = std::env::var("HARPER_DUMP_GERMAN_DICT") else {
            return;
        };
        let dict = curated_german_dictionary();
        let mut out = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        for word in dict.words_iter() {
            let Some(metadata) = dict.get_word_metadata(word) else {
                continue;
            };
            let word: String = word.iter().collect();
            let json = serde_json::to_string(&*metadata).unwrap();
            writeln!(out, "{word}\t{json}").unwrap();
        }
    }

    #[test]
    #[ignore = "reads and writes files named by the environment"]
    fn probe_german_dictionary() {
        let (Ok(input), Ok(output)) = (
            std::env::var("HARPER_PROBE_IN"),
            std::env::var("HARPER_PROBE_OUT"),
        ) else {
            return;
        };
        let dict = crate::language::german::spell::compound_aware_german_dictionary();
        let mut out = std::io::BufWriter::new(std::fs::File::create(output).unwrap());
        for word in std::fs::read_to_string(input).unwrap().lines() {
            let json = dict
                .get_word_metadata_str(word)
                .map(|metadata| serde_json::to_string(&*metadata).unwrap())
                .unwrap_or_default();
            writeln!(out, "{word}\t{json}").unwrap();
        }
    }

    /// Prints what the base dictionary says about every word of `HARPER_DEBUG_TEXT`,
    /// the way the linters see it.
    #[test]
    #[ignore = "reads the text from the environment"]
    fn show_german_tokens() {
        use crate::Document;
        use crate::language::german::parsers::PlainGerman;

        let Ok(text) = std::env::var("HARPER_DEBUG_TEXT") else {
            return;
        };
        let dict = curated_german_dictionary();
        let document = Document::new(&text, &PlainGerman, &dict);
        if std::env::var("HARPER_DEBUG_LINT").is_ok() {
            use crate::language::german::dialects::GermanDialect;
            use crate::language::german::linting::new_curated_german;
            use crate::linting::Linter;

            let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
            for lint in linter.lint(&document) {
                println!(
                    "LINT {:?} {:?}: {}",
                    document.get_span_content_str(&lint.span),
                    lint.lint_kind,
                    lint.message
                );
            }
            return;
        }
        for token in document.tokens() {
            let word: String = document.get_span_content_str(&token.span);
            if let Some(metadata) = token.kind.as_word().and_then(|m| m.as_ref()) {
                let mut pos = Vec::new();
                if metadata.noun.is_some() {
                    pos.push("noun");
                }
                if metadata.verb.is_some() {
                    pos.push("verb");
                }
                if metadata.adjective.is_some() {
                    pos.push("adj");
                }
                if metadata.adverb.is_some() {
                    pos.push("adv");
                }
                if metadata.conjunction.is_some() {
                    pos.push("conj");
                }
                if metadata.determiner.is_some() {
                    pos.push("det");
                }
                if metadata.pronoun.is_some() {
                    pos.push("pron");
                }
                if metadata.preposition {
                    pos.push("prep");
                }
                let m = serde_json::to_string(&metadata.morphology).unwrap();
                println!("{word}\t{} {m}", pos.join("+"));
            } else {
                println!("{word}\t-");
            }
        }
    }

    /// Prints how the compound checker takes the words of `HARPER_DEBUG_WORDS` apart.
    #[test]
    #[ignore = "reads the words from the environment"]
    fn show_german_decompositions() {
        let Ok(words) = std::env::var("HARPER_DEBUG_WORDS") else {
            return;
        };
        let dict = crate::language::german::spell::compound_aware_german_dictionary();
        for word in words.split_whitespace() {
            let chars: Vec<char> = word.chars().collect();
            println!("DECOMP {word}: {:?}", dict.decomposition(&chars));
        }
    }
}
