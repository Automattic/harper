//! Conformance suite for the language plugin system.
//!
//! Every test here walks the languages this build actually contains, via
//! `all_languages()`, instead of naming any of them. A language added to
//! `harper-core/src/language/` is therefore covered by all of it the moment it
//! compiles, and a module that satisfies `LanguageModule` but is wired in
//! wrongly fails here rather than in production.
//!
//! These are the invariants the registry promises. A language-specific
//! assertion belongs in that language's own tests, not in this file.

#![cfg(feature = "language-module")]

use std::collections::{BTreeMap, BTreeSet};

use harper_core::language::{
    Language, LanguageFamily, all_languages, default_language, dictionary_for_language,
    language_aliases, new_curated, parse_language, parser_for_prose, prose_language,
    weir_rules_lint_group,
};
use harper_core::linting::Linter;
use harper_core::parsers::MarkdownOptions;
use harper_core::spell::Dictionary;
use harper_core::{Document, Span};

/// Prose formats the registry claims to serve for every language.
const PROSE_FORMATS: &[&str] = &["plaintext", "text", "markdown", "quarto", "org", "mail"];

fn families() -> Vec<LanguageFamily> {
    let mut seen = Vec::new();
    for language in all_languages() {
        if !seen.contains(&language.family()) {
            seen.push(language.family());
        }
    }
    seen
}

/// A short, deliberately plain sentence. It need not be grammatical in any
/// given language — the point is that linting it must not panic.
const SMOKE_TEXT: &str = "Alpha beta gamma. Delta epsilon zeta?";

#[test]
fn english_is_always_present() {
    let languages = all_languages();
    assert!(
        languages
            .iter()
            .any(|l| l.family() == LanguageFamily::English),
        "English must be compiled in unconditionally, got {languages:?}"
    );
}

#[test]
fn every_dialect_reports_its_own_family() {
    for language in all_languages() {
        assert_eq!(
            Some(language.family()),
            Some(LanguageFamily::from(language)),
            "{language:?} disagrees with its own family"
        );
    }
}

#[test]
fn every_alias_parses_back_to_the_language_it_names() {
    for (alias, expected) in language_aliases() {
        assert_eq!(
            parse_language(alias),
            Some(expected),
            "alias {alias:?} is listed for {expected:?} but parse_language disagrees"
        );
    }
}

/// Aliases come from hand-written `config.toml` files, so two languages can
/// easily claim the same one. The first match would silently win.
#[test]
fn no_two_languages_claim_the_same_alias() {
    let mut owner: BTreeMap<&str, Language> = BTreeMap::new();
    for (alias, language) in language_aliases() {
        if let Some(previous) = owner.insert(alias, language)
            && previous != language
        {
            panic!("alias {alias:?} is claimed by both {previous:?} and {language:?}");
        }
    }
}

#[test]
fn aliases_are_lower_case() {
    // `parse_language` lower-cases its input before matching, so an alias with
    // a capital in it could never be reached.
    for (alias, language) in language_aliases() {
        assert_eq!(
            alias,
            alias.to_ascii_lowercase(),
            "alias {alias:?} of {language:?} can never match, parse_language lower-cases first"
        );
    }
}

#[test]
fn every_family_has_a_default_dialect_of_its_own() {
    for family in families() {
        let language = default_language(family);
        assert_eq!(
            language.family(),
            family,
            "default_language({family:?}) returned {language:?}"
        );
    }
}

#[test]
fn every_family_has_a_non_empty_dictionary() {
    for family in families() {
        let dictionary = dictionary_for_language(family);
        assert!(
            dictionary.word_count() > 0,
            "{family:?} ships an empty dictionary"
        );
    }
}

/// English keeps the empty suffix; every other language needs a distinct one,
/// or two languages would share a user-dictionary file on disk.
#[test]
fn dictionary_suffixes_are_unique() {
    let mut seen: BTreeMap<&str, LanguageFamily> = BTreeMap::new();
    for family in families() {
        let suffix = family.dict_suffix();
        if let Some(previous) = seen.insert(suffix, family) {
            panic!("{family:?} and {previous:?} share the dictionary suffix {suffix:?}");
        }
        if family != LanguageFamily::English {
            assert!(!suffix.is_empty(), "{family:?} has no dictionary suffix");
        }
    }
}

#[test]
fn every_language_has_a_parser_for_every_prose_format() {
    for language in all_languages() {
        for format in PROSE_FORMATS {
            assert!(
                parser_for_prose(format, language, MarkdownOptions::default()).is_some(),
                "{language:?} has no parser for {format:?}"
            );
        }
    }
}

/// The Brill tagger and the neural chunker are English-only models and cost
/// about four times as much as the rest of parsing. Every parser handed out
/// for a non-English language must opt out of them.
#[test]
fn non_english_parsers_skip_the_english_pos_models() {
    use harper_core::parsers::Parser;

    for language in all_languages() {
        let english = language.family() == LanguageFamily::English;
        for format in PROSE_FORMATS {
            let parser = parser_for_prose(format, language, MarkdownOptions::default())
                .unwrap_or_else(|| panic!("{language:?} has no parser for {format:?}"));
            assert_eq!(
                parser.is_english(),
                english,
                "the {format:?} parser for {language:?} reports is_english() == {}",
                parser.is_english()
            );
        }
    }
}

#[test]
fn prose_language_is_total() {
    // `prose_language` matches exhaustively; this proves the generated arms
    // cover every dialect and not merely every family.
    for language in all_languages() {
        let _ = prose_language(&language);
    }
}

#[test]
fn every_language_has_linters() {
    for language in all_languages() {
        let group = new_curated(language);
        assert!(
            group.iter_keys().next().is_some(),
            "{language:?} has an empty curated lint group"
        );
    }
}

/// Two linters under one key means one of them silently never runs.
#[test]
fn curated_lint_keys_are_unique_within_a_language() {
    for language in all_languages() {
        let group = new_curated(language);
        let mut seen = BTreeSet::new();
        for key in group.iter_keys() {
            assert!(
                seen.insert(key.to_string()),
                "{language:?} has two linters named {key:?}"
            );
        }
    }
}

#[test]
fn every_linter_describes_itself() {
    for language in all_languages() {
        let group = new_curated(language);
        for (key, description) in group.all_descriptions() {
            assert!(
                !description.trim().is_empty(),
                "linter {key:?} of {language:?} has no description"
            );
        }
    }
}

/// A language with `.weir` files on disk must end up with that many rules in
/// its group.
///
/// Calling `weir_rules_lint_group` and dropping the result -- which is what this
/// test used to do -- passes for a language with no rules at all, and three of
/// the five have none, so it never said anything about the one that does. It is
/// the same failure mode the language README warns about for
/// `curated_lint_group`: returning an empty group compiles, wires in cleanly and
/// silently checks nothing.
#[test]
fn every_weir_rule_on_disk_reaches_its_lint_group() {
    for language in all_languages() {
        let group = weir_rules_lint_group(language);
        let loaded = group.iter_keys().count();
        let on_disk = weir_rules_on_disk(language);

        assert_eq!(
            loaded, on_disk,
            "{language:?} has {on_disk} weir rules on disk but {loaded} in its group"
        );
    }
}

/// How many weir *rules* the language's directory holds, counted from the
/// source tree rather than from anything the build generated.
///
/// Two conventions, and they are not the same one:
///
/// - English keeps its rules at `src/linting/weir_rules` — not under
///   `src/language/`, because its data predates the language module — and a
///   **subdirectory there is one rule** made of several files (`InOfItself/`).
/// - A language module puts its rules in a subdirectory named by
///   `rules_subdirectory` in `config.toml` (`weir_rules/de/`), and **every file
///   in it is a rule**.
fn weir_rules_on_disk(language: Language) -> usize {
    if matches!(language, Language::English(_)) {
        let dir = std::path::Path::new("src/linting/weir_rules");
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        return entries
            .filter_map(Result::ok)
            .filter(|e| {
                let path = e.path();
                path.is_dir() || path.extension().is_some_and(|x| x == "weir")
            })
            .count();
    }

    let dir = std::path::Path::new("src/language")
        .join(language_directory(language))
        .join("linting/weir_rules");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return 0;
    };

    let mut count = 0;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            count += std::fs::read_dir(&path)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().is_some_and(|x| x == "weir"))
                .count();
        } else if path.extension().is_some_and(|x| x == "weir") {
            count += 1;
        }
    }
    count
}

/// The directory a language lives in, lower-cased from its family name — the
/// same convention `build.rs` uses to find it.
fn language_directory(language: Language) -> String {
    format!("{:?}", language.family()).to_lowercase()
}

/// The end-to-end path: registry parser, registry dictionary, registry lint
/// group. Every lint it produces has to point inside the document, or an
/// editor would place the squiggle out of bounds.
#[test]
fn linting_produces_in_bounds_lints_for_every_language() {
    for language in all_languages() {
        for format in PROSE_FORMATS {
            let parser = parser_for_prose(format, language, MarkdownOptions::default()).unwrap();
            let dictionary = dictionary_for_language(language.family());
            let document = Document::new(SMOKE_TEXT, &parser, &dictionary);
            let mut group = new_curated(language);

            let length = document.get_full_content().len();
            for lint in group.lint(&document) {
                let Span { start, end, .. } = lint.span;
                assert!(
                    start <= end && end <= length,
                    "{language:?}/{format:?} produced lint {:?} outside the {length}-char document",
                    lint.span
                );
            }
        }
    }
}

/// An empty document is the first thing a language server sees.
#[test]
fn empty_input_lints_cleanly_for_every_language() {
    for language in all_languages() {
        let parser = parser_for_prose("plaintext", language, MarkdownOptions::default()).unwrap();
        let dictionary = dictionary_for_language(language.family());
        let document = Document::new("", &parser, &dictionary);
        let mut group = new_curated(language);
        assert!(
            group.lint(&document).is_empty(),
            "{language:?} finds a problem in an empty document"
        );
    }
}
