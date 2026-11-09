use std::io::{self, BufRead, IsTerminal, Write};

use harper_core::linting::{LintGroup, Linter, Suggestion};
use harper_core::parsers::MarkdownOptions;
use harper_core::spell::{FstDictionary, spelling_candidates};
use harper_core::{Dialect, Document};

use crate::input::single_input::{SingleInput, SingleInputTrait};

const DIALECT: Dialect = Dialect::American;
const SHOWN_SUGGESTIONS: usize = 3;
const CHARS_AROUND_WORD: usize = 40;

pub fn run(input: Option<SingleInput>) -> anyhow::Result<()> {
    let dictionary = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dictionary.clone(), DIALECT);
    linter.set_all_rules_to(Some(false));
    linter.config.set_rule_enabled("SpellCheck", true);

    if let Some(input) = input {
        let (document, _) = input.load(MarkdownOptions::default(), dictionary.as_ref())?;
        print_comparison(&document, &mut linter, &dictionary);
        return Ok(());
    }

    let stdin = io::stdin();
    let prompt = || -> io::Result<()> {
        if stdin.is_terminal() {
            print!("> ");
            io::stdout().flush()?;
        }
        Ok(())
    };

    prompt()?;
    for line in stdin.lock().lines() {
        let document = Document::new_markdown_default_curated(&line?);
        print_comparison(&document, &mut linter, &dictionary);
        prompt()?;
    }

    Ok(())
}

fn print_comparison(document: &Document, linter: &mut LintGroup, dictionary: &FstDictionary) {
    let lints = linter.lint(document);

    if lints.is_empty() {
        println!("No misspelled words.");
    }

    for lint in lints {
        let word = document.get_span_content(&lint.span);
        let today: Vec<String> = spelling_candidates(word, dictionary, DIALECT, SHOWN_SUGGESTIONS)
            .into_iter()
            .map(|(candidate, _)| candidate.iter().collect::<String>().to_lowercase())
            .collect();
        let ranked: Vec<String> = lint
            .suggestions
            .iter()
            .filter_map(|suggestion| match suggestion {
                Suggestion::ReplaceWith(candidate) => {
                    Some(candidate.iter().collect::<String>().to_lowercase())
                }
                _ => None,
            })
            .collect();

        let source = document.get_source();
        let start = lint.span.start.saturating_sub(CHARS_AROUND_WORD);
        let end = (lint.span.end + CHARS_AROUND_WORD).min(source.len());
        let around: String = source[start..end]
            .iter()
            .map(|c| if c.is_whitespace() { ' ' } else { *c })
            .collect();

        println!("{}: {around}", document.get_span_content_str(&lint.span));
        println!("  Harper today: {}", today.join(", "));
        println!("  Common Pile:  {}", ranked.join(", "));
    }
}
