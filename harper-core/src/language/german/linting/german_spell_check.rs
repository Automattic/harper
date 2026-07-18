use hashbrown::HashMap;

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document, spell::Dictionary};

const MIN_COMPOUND_PART_LEN: usize = 3;
const MAX_COMPOUND_PARTS: usize = 5;
const EMPTY_INTERFIX: &[char] = &[];
const S_INTERFIX: &[char] = &['s'];
const N_INTERFIX: &[char] = &['n'];
const EN_INTERFIX: &[char] = &['e', 'n'];
const ER_INTERFIX: &[char] = &['e', 'r'];
const ES_INTERFIX: &[char] = &['e', 's'];
const GERMAN_COMPOUND_INTERFIXES: &[&[char]] = &[
    EMPTY_INTERFIX,
    S_INTERFIX,
    N_INTERFIX,
    EN_INTERFIX,
    ER_INTERFIX,
    ES_INTERFIX,
];

/// A spell checker for German text with compound word handling.
pub struct GermanSpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    fn strip_compound_interfix<'a>(
        &self,
        remainder: &'a [char],
        interfix: &[char],
    ) -> Option<&'a [char]> {
        remainder.strip_prefix(interfix)
    }

    fn is_valid_compound_segment(
        &self,
        word: &[char],
        depth: usize,
        memo: &mut HashMap<Vec<char>, bool>,
    ) -> bool {
        if word.len() < MIN_COMPOUND_PART_LEN {
            return false;
        }

        if depth >= MAX_COMPOUND_PARTS {
            return false;
        }

        if depth > 0 && self.dictionary.contains_word(word) {
            return true;
        }

        if let Some(cached) = memo.get(word) {
            return *cached;
        }

        let mut valid = false;

        for split_pos in MIN_COMPOUND_PART_LEN..=word.len() - MIN_COMPOUND_PART_LEN {
            let first_part = &word[..split_pos];

            if !self.dictionary.contains_word(first_part) {
                continue;
            }

            let remainder = &word[split_pos..];

            for interfix in GERMAN_COMPOUND_INTERFIXES {
                let Some(next_part) = self.strip_compound_interfix(remainder, interfix) else {
                    continue;
                };

                if next_part.len() < MIN_COMPOUND_PART_LEN {
                    continue;
                }

                // In German, compound noun parts are capitalized. Try both the original
                // and capitalized versions of the next part.
                let mut capitalized_next_part = next_part.to_vec();
                if let Some(first_char) = capitalized_next_part.first_mut() {
                    *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                }

                if self.dictionary.contains_word(next_part)
                    || self.dictionary.contains_word(&capitalized_next_part)
                    || self.is_valid_compound_segment(next_part, depth + 1, memo)
                    || self.is_valid_compound_segment(&capitalized_next_part, depth + 1, memo)
                {
                    valid = true;
                    break;
                }
            }

            if valid {
                break;
            }
        }

        memo.insert(word.to_vec(), valid);
        valid
    }

    /// Check if a word is a valid German compound.
    /// German freely combines nouns and often inserts linking morphemes such as
    /// `s`, `n`, `en`, `er`, `e`, or `es` between parts.
    fn try_compound_word_check(&self, word: &[char]) -> bool {
        if word.len() < MIN_COMPOUND_PART_LEN * 2 {
            return false;
        }

        let mut memo = HashMap::new();
        self.is_valid_compound_segment(word, 0, &mut memo)
    }

    /// Get spelling suggestions for a word using fuzzy matching.
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        // Use the dictionary's fuzzy matching (FST-based Levenshtein)
        let results = self.dictionary.fuzzy_match(word, 2, 5);
        let mut suggestions: Vec<Vec<char>> =
            results.into_iter().map(|r| r.word.to_vec()).collect();

        // Also try simple capitalization fix (common German error)
        if suggestions.is_empty() && word.len() > 1 {
            let mut capitalized = word.to_vec();
            if let Some(first_char) = capitalized.first_mut() {
                *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
            }
            if self.dictionary.contains_word(&capitalized) {
                suggestions.push(capitalized);
            }
        }

        suggestions.truncate(5);
        suggestions
    }
}

impl<T: Dictionary> Linter for GermanSpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                for word in sentence.iter_words() {
                    let word_chars = document.get_span_content(&word.span);

                    // Skip words in dictionary
                    if self.dictionary.contains_word(word_chars) {
                        continue;
                    }

                    // Try compound word splitting
                    if self.try_compound_word_check(word_chars) {
                        continue;
                    }

                    // Get spelling suggestions
                    let suggestions = self.get_suggestions(word_chars);
                    let word_str: String = word_chars.iter().collect();

                    let message = if !suggestions.is_empty() {
                        let suggestions_str: Vec<String> = suggestions
                            .iter()
                            .map(|s| s.iter().collect::<String>())
                            .collect();
                        format!(
                            "Possible spelling error: \"{}\". Did you mean: {}?",
                            word_str,
                            suggestions_str.join(", ")
                        )
                    } else {
                        format!("Unknown word: \"{}\".", word_str)
                    };

                    lints.push(Lint {
                        span: word.span,
                        lint_kind: LintKind::Spelling,
                        suggestions: suggestions
                            .into_iter()
                            .map(Suggestion::ReplaceWith)
                            .collect(),
                        priority: 20,
                        message,
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for spelling errors in German text"
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSpellCheck;
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_text(text: &str) -> Vec<String> {
        use crate::language::german::linting::new_curated_german;
        use crate::language::german::spell::combined_german_dictionary;
        let dict = combined_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);

        linter
            .lint(&document)
            .into_iter()
            .map(|lint| lint.message)
            .collect()
    }

    fn recognizes_compound(word: &str) -> bool {
        let dict = combined_german_dictionary();
        let spellcheck = GermanSpellCheck::new(dict);
        let chars: Vec<char> = word.chars().collect();

        spellcheck.try_compound_word_check(&chars)
    }

    #[test]
    fn recognizes_recursive_compounds() {
        for word in [
            "Gartenhaus",
            "Arbeitsstelle",
            "Frühstücksspeck",
            "Straßenrand",
            "Festplattenspeicher",
        ] {
            assert!(
                recognizes_compound(word),
                "{word} should be treated as a valid compound"
            );
        }
    }

    #[test]
    fn does_not_accept_misspelled_compounds() {
        for word in ["Festplattenspeicer", "Arbeitsplaz", "Straßenrant"] {
            assert!(
                !recognizes_compound(word),
                "{word} should not be treated as a valid compound"
            );
        }
    }

    #[test]
    fn recognizes_simple_compounds() {
        for word in ["Gartenhaus", "Arbeitsstelle", "Straßenrand"] {
            assert!(
                recognizes_compound(word),
                "{word} should be treated as a valid compound"
            );
        }
    }

    #[test]
    fn lint_allows_festplattenspeicher() {
        let messages = lint_text("Der Festplattenspeicher ist fast voll.");

        assert!(
            messages
                .iter()
                .all(|message| !message.contains("Festplattenspeicher")),
            "Festplattenspeicher should not be flagged: {messages:?}"
        );
    }

    #[test]
    fn lint_flags_misspelled_storage_compounds() {
        let messages = lint_text("Der Festplattenspeicer ist fast voll.");

        assert!(
            messages
                .iter()
                .any(|message| message.contains("Festplattenspeicer")),
            "Misspelled compound should still be flagged: {messages:?}"
        );
    }

    #[test]
    fn lint_allows_common_technical_compounds() {
        let messages = lint_text(
            "Die Systemvoraussetzungen sind dokumentiert. \
             Das Betriebssystem nutzt eine Konfigurationsdatei im Texteditor zur Fehlerbehebung. \
             Ihre Unterschrift und die Unterschriften sind vorhanden.",
        );

        for word in [
            "Systemvoraussetzungen",
            "Betriebssystem",
            "Konfigurationsdatei",
            "Texteditor",
            "Fehlerbehebung",
            "Unterschrift",
            "Unterschriften",
        ] {
            assert!(
                messages.iter().all(|message| !message.contains(word)),
                "{word} should not be flagged: {messages:?}"
            );
        }
    }

    #[test]
    fn curated_german_uses_german_spellcheck_instead_of_generic_spellcheck() {
        use crate::language::german::linting::new_curated_german;
        use crate::language::german::spell::curated_german_dictionary;
        let linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        assert!(linter.config.is_rule_enabled("GermanSpellCheck"));
        assert!(!linter.config.is_rule_enabled("SpellCheck"));
    }
}
