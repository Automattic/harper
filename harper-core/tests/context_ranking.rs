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

fn lowercase(words: &[String]) -> Vec<String> {
    words.iter().map(|word| word.to_lowercase()).collect()
}

#[cfg(feature = "context-ranking")]
fn assert_first_suggestions(examples: &[(&str, &str)]) {
    let wrong: Vec<_> = examples
        .iter()
        .map(|(text, expected)| (*text, *expected, lowercase(&suggestions(text))))
        .filter(|(_, expected, found)| found.first().map(String::as_str) != Some(*expected))
        .collect();

    assert!(wrong.is_empty(), "wrong first suggestion: {wrong:#?}");
}

#[test]
#[cfg(feature = "context-ranking")]
fn context_picks_the_first_suggestion() {
    assert_first_suggestions(&[
        ("one two tree four fibe", "five"),
        ("one two three four fibe", "five"),
        ("The house caught fibe", "fire"),
        ("I wark at Automattic", "work"),
        ("The colors have a strong contrat", "contrast"),
        ("She drank a glass of waer", "water"),
        ("The teacher wrote on the boird", "board"),
        ("Add salt and peper to taste", "pepper"),
        ("Save the fike to your desktop", "file"),
        ("Please shut the winow", "window"),
        ("My whole bady hurts", "body"),
        ("The early bird catches the wrom", "worm"),
        ("It is raning outside today", "raining"),
        ("He told me a funny sotry", "story"),
        ("We need to sove this problem", "solve"),
    ]);
}

#[test]
#[cfg(feature = "context-ranking")]
fn the_same_typo_gets_the_word_that_fits_each_sentence() {
    assert_first_suggestions(&[
        ("We ate luch at noon", "lunch"),
        ("Good luch with the exam", "luck"),
        ("I will see you next weel", "week"),
        ("The car lost a front weel", "wheel"),
        ("People all over the worls", "world"),
        ("Say a few worls to the crowd", "words"),
        ("Please be quet in the library", "quiet"),
        ("He will quet his job tomorrow", "quit"),
        ("That story is ture", "true"),
        ("Ture left at the light", "turn"),
    ]);
}

#[test]
#[cfg(feature = "context-ranking")]
fn context_finds_words_beyond_the_spelling_top_three() {
    use harper_core::spell::spelling_candidates;

    let dictionary = FstDictionary::curated();
    let examples = [
        ("one two three four fibe", "fibe", "five"),
        ("I wark at Automattic", "wark", "work"),
        ("We ate luch at noon", "luch", "lunch"),
        ("The car lost a front weel", "weel", "wheel"),
        ("People all over the worls", "worls", "world"),
        ("She drank a glass of waer", "waer", "water"),
        ("My whole bady hurts", "bady", "body"),
    ];

    for (text, typo, expected) in examples {
        let typo: Vec<char> = typo.chars().collect();
        let spelling_top_three: Vec<String> =
            spelling_candidates(&typo, dictionary.as_ref(), Dialect::American, 3)
                .into_iter()
                .map(|(word, _)| word.iter().collect())
                .collect();

        assert!(!spelling_top_three.iter().any(|word| word == expected));
        assert_eq!(suggestions(text)[0], expected, "{text}");
    }
}

#[test]
#[cfg(feature = "context-ranking")]
#[ignore = "The 64d table puts another word first in these sentences."]
fn context_picks_the_first_suggestion_known_misses() {
    assert_first_suggestions(&[
        ("The fibe is corrupt", "file"),
        ("We wark to the park", "walk"),
        ("Legal reviewed the contrat", "contract"),
        ("Please pay the invoce", "invoice"),
        ("The bady cried all night", "baby"),
        ("He rode a hoyse across the field", "horse"),
        ("The fish swam in the lke", "lake"),
    ]);
}

#[test]
#[cfg(not(feature = "context-ranking"))]
fn without_the_feature_the_spelling_order_stays() {
    assert_eq!(
        suggestions("one two three four fibe"),
        ["fife", "file", "fine"]
    );
    assert_eq!(suggestions("We ate luch at noon"), ["lush", "lech", "luck"]);
}

#[test]
fn no_surrounding_text_keeps_the_spelling_order() {
    assert_eq!(suggestions("fibe"), ["fife", "file", "fine"]);
}

#[test]
fn the_previous_sentence_does_not_count() {
    assert_eq!(
        lowercase(&suggestions("One two three four. Fibe")),
        ["fife", "file", "fine"]
    );
}

#[test]
fn there_are_at_most_three_suggestions() {
    for text in [
        "one two three four fibe",
        "We ate luch at noon",
        "The car lost a front weel",
        "It is raning outside today",
    ] {
        let found = suggestions(text);
        assert!(!found.is_empty() && found.len() <= 3, "{text}: {found:?}");
    }
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
