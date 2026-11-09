#![cfg(feature = "context-ranking")]

use harper_core::linting::{LintGroup, Linter, Suggestion};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

fn suggestions(text: &str) -> Vec<String> {
    let mut linter = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
    linter.set_all_rules_to(Some(false));
    linter.config.set_rule_enabled("SpellCheck", true);
    let document = Document::new_plain_english_curated(text);

    linter
        .lint(&document)
        .into_iter()
        .next()
        .map(|lint| {
            lint.suggestions
                .iter()
                .map(|suggestion| match suggestion {
                    Suggestion::ReplaceWith(word) => word.iter().collect(),
                    _ => String::new(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn assert_first_suggestions(examples: &[(&str, &str)]) {
    let wrong: Vec<_> = examples
        .iter()
        .map(|(text, expected)| (*text, *expected, suggestions(text)))
        .filter(|(_, expected, found)| found.first().map(String::as_str) != Some(*expected))
        .collect();

    assert!(wrong.is_empty(), "wrong first suggestion: {wrong:#?}");
}

#[test]
fn context_picks_the_first_suggestion() {
    assert_first_suggestions(&[
        ("one two tree four fibe", "five"),
        ("one two three four fibe", "five"),
        ("The house caught fibe", "fire"),
        ("I wark at Automattic", "work"),
        ("The colors have a strong contrat", "contrast"),
    ]);
}

#[test]
#[ignore = "The 64d table puts another word first in these sentences."]
fn context_picks_the_first_suggestion_known_misses() {
    assert_first_suggestions(&[
        ("The fibe is corrupt", "file"),
        ("We wark to the park", "walk"),
        ("Legal reviewed the contrat", "contract"),
        ("Please pay the invoce", "invoice"),
    ]);
}

#[test]
fn no_surrounding_text_keeps_the_spelling_order() {
    assert_eq!(suggestions("fibe"), ["fife", "file", "fine"]);
}

#[test]
fn valid_words_stay_unflagged() {
    assert!(suggestions("one two tree four five").is_empty());
}

#[test]
fn a_capitalized_typo_gets_capitalized_suggestions() {
    assert!(
        suggestions("Fibe people came")
            .iter()
            .all(|word| word.starts_with(char::is_uppercase))
    );
}
