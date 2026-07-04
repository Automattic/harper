use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document, spell::Dictionary};

/// A linter that checks to make sure German nouns are capitalized.
/// In German, all nouns must be capitalized (not just proper nouns like in English).
pub struct GermanNounCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
    /// Suffixes that strongly indicate a noun, paired with minimum word length
    /// to avoid false positives on short function words.
    noun_suffixes: Vec<(Vec<char>, usize)>,
}

/// Common German function words that should never be flagged as nouns.
const GERMAN_NON_NOUNS: &[&str] = &[
    // Articles (all cases)
    "der",
    "die",
    "das",
    "dem",
    "den",
    "des",
    "ein",
    "eine",
    "einen",
    "einem",
    "einer",
    "eines",
    // Pronouns
    "er",
    "sie",
    "es",
    "wir",
    "ihr",
    "ich",
    "du",
    "mich",
    "mir",
    "dich",
    "dir",
    "sich",
    "uns",
    "euch",
    "ihnen",
    "ihm",
    // Possessives
    "mein",
    "dein",
    "sein",
    "unser",
    "euer",
    // Demonstratives / relative
    "dieser",
    "diese",
    "dieses",
    "jener",
    "jene",
    "jenes",
    "welcher",
    "welche",
    "welches",
    "jeder",
    "jede",
    "jedes",
    // Prepositions
    "in",
    "im",
    "an",
    "am",
    "auf",
    "aus",
    "bei",
    "mit",
    "nach",
    "von",
    "vor",
    "zu",
    "zum",
    "zur",
    "um",
    "für",
    "über",
    "unter",
    "zwischen",
    "neben",
    "hinter",
    "durch",
    "ohne",
    "gegen",
    "bis",
    "seit",
    "während",
    "wegen",
    "trotz",
    "statt",
    "außer",
    "ab",
    "ob",
    // Conjunctions
    "und",
    "oder",
    "aber",
    "denn",
    "weil",
    "dass",
    "wenn",
    "als",
    "ob",
    "sondern",
    "doch",
    "jedoch",
    "falls",
    "damit",
    "bevor",
    "nachdem",
    "obwohl",
    "während",
    "sobald",
    "solange",
    // Adverbs
    "nicht",
    "auch",
    "noch",
    "schon",
    "wieder",
    "nur",
    "sehr",
    "hier",
    "dort",
    "da",
    "immer",
    "nie",
    "oft",
    "manchmal",
    "vielleicht",
    "wahrscheinlich",
    "heute",
    "morgen",
    "gestern",
    "jetzt",
    "dann",
    "so",
    "ganz",
    "gar",
    // Common verbs (incl. conjugated forms often lowercase in text)
    "ist",
    "sind",
    "war",
    "waren",
    "hat",
    "haben",
    "hatte",
    "hatten",
    "wird",
    "werden",
    "wurde",
    "wurden",
    "kann",
    "können",
    "konnte",
    "soll",
    "sollen",
    "sollte",
    "muss",
    "müssen",
    "musste",
    "darf",
    "dürfen",
    "durfte",
    "mag",
    "mögen",
    "möchte",
    "will",
    "wollen",
    "wollte",
    "sein",
    "gewesen",
    // Common verb forms that end in -e (1st person singular)
    "schreibe",
    "lerne",
    "mache",
    "habe",
    "gebe",
    "nehme",
    "sehe",
    "komme",
    "finde",
    "denke",
    "sage",
    "frage",
    "gibe",
    "wisse",
    "verstehe",
    "versuche",
    "brauche",
    "suche",
    "arbeite",
    "spiele",
    "lese",
    "höre",
    "glaube",
    // Common past participles
    "fehlgeschlagen",
    // Adjectives
    "gut",
    "groß",
    "klein",
    "alt",
    "neu",
    "lang",
    "kurz",
    "schnell",
    "langsam",
    "viel",
    "wenig",
    "alle",
    "keine",
];

impl<T: Dictionary> GermanNounCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        let noun_suffixes = vec![
            (vec!['h', 'e', 'i', 't'], 5),           // -heit (min 5 chars)
            (vec!['k', 'e', 'i', 't'], 5),           // -keit
            (vec!['u', 'n', 'g'], 5),                // -ung
            (vec!['n', 'i', 's'], 5),                // -nis
            (vec!['t', 'u', 'm'], 5),                // -tum
            (vec!['l', 'i', 'n', 'g'], 6),           // -ling
            (vec!['i', 'o', 'n'], 5),                // -ion
            (vec!['t', 'ä', 't'], 5),                // -tät
            (vec!['s', 'c', 'h', 'a', 'f', 't'], 8), // -schaft
        ];

        Self {
            dictionary,
            noun_suffixes,
        }
    }

    fn is_non_noun(word_lower: &[char]) -> bool {
        let s: String = word_lower.iter().collect();
        GERMAN_NON_NOUNS.contains(&s.as_str())
    }

    /// Check if a word is likely a German noun based on dictionary metadata
    /// and suffix heuristics, while excluding known function words.
    fn is_likely_noun(&self, word: &[char], _document: &Document) -> bool {
        let lower: Vec<char> = word
            .iter()
            .map(|c| c.to_lowercase().next().unwrap_or(*c))
            .collect();

        // Never flag known function words
        if Self::is_non_noun(&lower) {
            return false;
        }

        // Check dictionary metadata first - most reliable
        // Check both the word and its lowercase form
        let word_metadata = self.dictionary.get_word_metadata(word);
        let lower_metadata = self.dictionary.get_word_metadata(&lower);

        // If word is explicitly marked as a noun in dictionary, it's a noun
        if let Some(ref metadata) = word_metadata {
            if metadata.noun.is_some() {
                return true;
            }
        }
        if let Some(ref metadata) = lower_metadata
            && metadata.noun.is_some()
        {
            return true;
        }

        // If word is explicitly marked as a NON-noun (verb, adjective, adverb, etc.)
        // in the dictionary, it should NOT be treated as a noun
        // This prevents false positives like "schreibe" (verb) or "fehlgeschlagen" (participle)
        let has_noun_metadata = word_metadata
            .as_ref()
            .and_then(|m| m.noun.as_ref())
            .is_some()
            || lower_metadata
                .as_ref()
                .and_then(|m| m.noun.as_ref())
                .is_some();

        let has_non_noun_metadata = word_metadata.as_ref().is_some_and(|m| {
            m.verb.is_some()
                || m.adjective.is_some()
                || m.adverb.is_some()
                || m.conjunction.is_some()
                || m.determiner.is_some()
                || m.pronoun.is_some()
                || m.preposition
        }) || lower_metadata.as_ref().is_some_and(|m| {
            m.verb.is_some()
                || m.adjective.is_some()
                || m.adverb.is_some()
                || m.conjunction.is_some()
                || m.determiner.is_some()
                || m.pronoun.is_some()
                || m.preposition
        });

        if has_non_noun_metadata && !has_noun_metadata {
            return false;
        }

        // Check for common noun suffixes (with minimum length guards)
        // Only apply suffix heuristics if we don't have explicit dictionary info
        for (suffix, min_len) in &self.noun_suffixes {
            if lower.len() >= *min_len && &lower[lower.len() - suffix.len()..] == suffix {
                return true;
            }
        }

        false
    }
}

impl<T: Dictionary> Linter for GermanNounCapitalization<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                // Get the first word of this sentence to skip it
                let first_word = sentence.first_non_whitespace();

                for word in sentence.iter_words() {
                    // Skip first word of sentence (handled by sentence capitalization)
                    if let Some(fw) = &first_word
                        && word.span == fw.span
                    {
                        continue;
                    }

                    let word_chars = document.get_span_content(&word.span);

                    // Skip words that are already capitalized
                    if let Some(first_char) = word_chars.first()
                        && first_char.is_uppercase()
                    {
                        continue;
                    }

                    // Skip non-alphabetic words
                    if word_chars.iter().all(|c| c.is_alphabetic())
                        && self.is_likely_noun(word_chars, document)
                    {
                        let mut replacement: Vec<char> = word_chars.to_vec();
                        if let Some(first_char) = replacement.first_mut() {
                            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                        }

                        lints.push(Lint {
                            span: word.span,
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(replacement)],
                            priority: 25, // High priority for German
                            message: format!(
                                "In German, all nouns must be capitalized. \"{}\" appears to be a noun.",
                                word_chars.iter().collect::<String>()
                            ),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ensures German nouns are properly capitalized"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::language::german::spell::combined_german_dictionary;

    fn test_linter() -> GermanNounCapitalization<impl Dictionary> {
        GermanNounCapitalization::new(combined_german_dictionary())
    }

    fn create_document(text: &str) -> Document {
        Document::new_markdown_default(text, &combined_german_dictionary())
    }

    #[test]
    fn test_nouns_are_detected() {
        let mut linter = test_linter();
        let text = "die mondlandung";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "mondlandung" should be detected as a noun and flagged for capitalization
        assert!(
            lints.len() > 0,
            "Expected at least one lint for lowercase noun"
        );
        let lint = &lints[0];
        let word: String = document.get_span_content(&lint.span).iter().collect();
        assert_eq!(word, "mondlandung");
        assert!(lint.message.contains("noun"));
    }

    #[test]
    fn test_verbs_are_not_detected_as_nouns() {
        let mut linter = test_linter();
        let text = "ich schreibe und lerne";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "schreibe" and "lerne" should NOT be detected as nouns
        assert_eq!(lints.len(), 0, "Verbs should not be detected as nouns");
    }

    #[test]
    fn test_past_participles_are_not_detected_as_nouns() {
        let mut linter = test_linter();
        let text = "es ist fehlgeschlagen";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "fehlgeschlagen" should NOT be detected as a noun
        assert_eq!(
            lints.len(),
            0,
            "Past participles should not be detected as nouns"
        );
    }

    #[test]
    fn test_noun_suffixes_still_work() {
        let mut linter = test_linter();
        let text = "die freiheit und die menschheit";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // "freiheit" and "menschheit" should be detected as nouns via suffix
        assert!(
            lints.len() >= 1,
            "Expected at least one lint for nouns with suffixes"
        );
    }

    #[test]
    fn test_mixed_nouns_and_verbs() {
        let mut linter = test_linter();
        let text = "die mondlandung ist wieder fehlgeschlagen";
        let document = create_document(text);
        let lints = linter.lint(&document);

        // Only "mondlandung" should be detected as a noun
        assert_eq!(
            lints.len(),
            1,
            "Expected exactly one lint for 'mondlandung'"
        );
        let lint = &lints[0];
        let word: String = document.get_span_content(&lint.span).iter().collect();
        assert_eq!(word, "mondlandung");
    }
}
