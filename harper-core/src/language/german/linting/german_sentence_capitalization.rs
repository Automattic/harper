use crate::language::german::linting::german_foreign_stretch;
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Punctuation, Token, TokenKind, TokenStringExt, document::Document, spell::Dictionary};

/// German abbreviations whose full stop does not end a sentence.
///
/// The tokenizer has no way to tell `bzw.` from the end of a clause, so the word
/// after one looks like the first word of a new sentence and gets reported for
/// being lower case. On encyclopedic prose this was the *only* thing this rule
/// ever fired on.
const SENTENCE_INTERNAL_ABBREVIATIONS: &[&str] = &[
    // General
    "bzw",
    "ggf",
    "usw",
    "usf",
    "vgl",
    "ca",
    "evtl",
    "insb",
    "bspw",
    "sog",
    "etc",
    "ebd",
    "dgl",
    "inkl",
    "exkl",
    "zzgl",
    "abzgl",
    "max",
    "min",
    "bzgl",
    "vs",
    "zit",
    "sn",
    // The single letters of "z. B.", "d. h.", "u. a.", "i. d. R.", "n. Chr."
    "z",
    "d",
    "h",
    "u",
    "a",
    "s",
    "o",
    "i",
    "e",
    "b",
    "n",
    "v",
    "r",
    "chr",
    // Bibliographic
    "nr",
    "abb",
    "tab",
    "hg",
    "hrsg",
    "jh",
    "jhd",
    "jt",
    "geb",
    "gest",
    "verh",
    "gedr",
    "erw",
    "überarb",
    "aufl",
    "bd",
    "bde",
    "kap",
    "anm",
    "übers",
    "bearb",
    "mitarb",
    "red",
    "ff",
    "sp",
    // Titles and degrees
    "st",
    "sankt",
    "prof",
    "dr",
    "med",
    "phil",
    "rer",
    "nat",
    "jur",
    "ing",
    "dipl",
    "theol",
    "jr",
    "sen",
    "habil",
    // Languages, which German prose cites constantly
    "dt",
    "engl",
    "frz",
    "lat",
    "griech",
    "altgriech",
    "ital",
    "span",
    "port",
    "russ",
    "poln",
    "pol",
    "tschech",
    "slow",
    "kroat",
    "serb",
    "ung",
    "rum",
    "bulg",
    "türk",
    "arab",
    "hebr",
    "pers",
    "chin",
    "jap",
    "kor",
    "nl",
    "ndl",
    "schwed",
    "norw",
    "dän",
    "finn",
    "isl",
    "ir",
    "schott",
    "wal",
    "bret",
    "kelt",
    "germ",
    "ahd",
    "mhd",
    "nhd",
    "got",
    "aram",
    "sanskr",
    // Taxonomy, which encyclopedic prose cites just as often
    "spec",
    "subsp",
    "var",
    "cf",
    "syn",
    "fam",
    "gen",
];

/// A linter that checks to make sure the first word of each sentence is
/// capitalized in German text.
pub struct GermanSentenceCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSentenceCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Does the "sentence" starting at `start` in fact continue one, because the
    /// full stop before it belongs to an abbreviation or an ordinal?
    ///
    /// *"Ludwig II. von Savoyen"*, *"Karl IV. den erblichen Titel"*, *"das
    /// Messkabel und ggf. die Verstärkung"* — in each case the period is part of
    /// the token before it, and the word after is mid-sentence.
    fn continues_previous_sentence(document: &Document, start: usize) -> bool {
        let tokens = document.get_tokens();

        let Ok(index) = tokens.binary_search_by_key(&start, |t| t.span.start) else {
            return false;
        };

        // A sentence break needs a space after the full stop. Without one this is
        // something like the host name "cassini.ehess", split at its own dot.
        if let Some(previous) = index.checked_sub(1).map(|i| &tokens[i])
            && matches!(previous.kind, TokenKind::Punctuation(Punctuation::Period))
        {
            return true;
        }

        let mut preceding = tokens[..index]
            .iter()
            .rev()
            .filter(|t| !t.kind.is_whitespace());

        if !matches!(
            preceding.next().map(|t| &t.kind),
            Some(TokenKind::Punctuation(Punctuation::Period))
        ) {
            return false;
        }

        let Some(before_period) = preceding.next() else {
            return false;
        };

        Self::is_abbreviation_or_ordinal(before_period, document)
    }

    /// Does the sentence that starts at `start` in fact start inside a quoted
    /// foreign title?
    ///
    /// *"Is This What We Want? **als** Protest gegen …"* — the question mark
    /// belongs to the English album title, so the German word after it is
    /// mid-sentence and correctly lower case. The same evidence settles it as
    /// for a lower-case noun, so the same detector decides.
    ///
    /// No determiner test is passed in: this candidate opens a sentence, so
    /// what precedes it is punctuation rather than a German article.
    fn starts_inside_foreign_text(document: &Document, start: usize) -> bool {
        let tokens: Vec<&Token> = document
            .get_tokens()
            .iter()
            .filter(|t| !t.kind.is_whitespace())
            .collect();

        let Some(index) = tokens.iter().position(|t| t.span.start == start) else {
            return false;
        };

        german_foreign_stretch::in_foreign_stretch(&tokens, index, document, |_| false)
    }

    /// Ordinals ("II.", "1905.") and the abbreviations listed above.
    fn is_abbreviation_or_ordinal(token: &Token, document: &Document) -> bool {
        if matches!(token.kind, TokenKind::Number(_) | TokenKind::Decade) {
            return true;
        }

        if !token.kind.is_word() {
            return false;
        }

        let chars = document.get_span_content(&token.span);

        // A Roman numeral: regnal numbers, century and volume numbers.
        if !chars.is_empty()
            && chars
                .iter()
                .all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
        {
            return true;
        }

        let lower: String = chars.iter().flat_map(|c| c.to_lowercase()).collect();
        SENTENCE_INTERNAL_ABBREVIATIONS.contains(&lower.as_str())
    }
}

impl<T: Dictionary> Linter for GermanSentenceCapitalization<T> {
    /// A linter that checks to make sure the first word of each sentence is
    /// capitalized.
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            // Allows short, label-like comments in code.
            if paragraph.iter_sentences().count() == 1 {
                let only_sentence = paragraph.iter_sentences().next().unwrap();

                if !only_sentence
                    .iter_chunks()
                    .map(|c| c.iter_words().count())
                    .any(|c| c > 5)
                {
                    continue;
                }
            }

            for sentence in paragraph.iter_sentences() {
                // Basic sentence length check
                if sentence.iter_words().count() < 3 {
                    continue;
                }

                if let Some(first_word) = sentence.first_non_whitespace()
                    && first_word.kind.is_word()
                {
                    let word_chars = document.get_span_content(&first_word.span);

                    if let Some(first_char) = word_chars.first()
                        && first_char.is_alphabetic()
                        && !first_char.is_uppercase()
                        && !Self::continues_previous_sentence(document, first_word.span.start)
                        && !Self::starts_inside_foreign_text(document, first_word.span.start)
                    {
                        let target_span = first_word.span;
                        let mut replacement_chars =
                            document.get_span_content(&target_span).to_vec();
                        if let Some(first_char) = replacement_chars.first_mut() {
                            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                        }

                        lints.push(Lint {
                            span: target_span,
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
                            priority: 30,
                            message: "Sentences must start with a capital letter".to_string(),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks that sentences start with capital letters in German text"
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSentenceCapitalization;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let mut linter = GermanSentenceCapitalization::new(dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);
        linter.lint(&document).len()
    }

    #[test]
    fn an_abbreviation_does_not_end_a_sentence() {
        for text in [
            "Die Dämpfung des Messkabels und ggf. die Verstärkung eines Vorverstärkers zählen.",
            "Die Droga ekspresowa S12 (pol. für Schnellstraße S12) ist eine lange Straße.",
            "Matthisius wurde 1558 zum Dr. theol. promoviert und anschließend berufen.",
            "Bacteroides spec. gehören insbesondere als Darmkeime zur normalen Flora.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }

    #[test]
    fn an_ordinal_does_not_end_a_sentence() {
        for text in [
            "Sein Onkel Ludwig II. von Savoyen war sein Vormund und Regent.",
            "Der Kaiser Karl IV. den erblichen Titel eines Reichsgrafen verlieh das Recht.",
            "Von Aristoteles bis ins 19. Jahrhundert wurde die Physik anders betrieben.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }

    #[test]
    fn a_dot_without_a_space_is_not_a_sentence_break() {
        assert_eq!(
            lint_count("Die Zahlen basieren auf den Daten von cassini.ehess und INSEE."),
            0
        );
    }

    #[test]
    fn a_genuinely_lowercase_sentence_is_still_flagged() {
        assert_eq!(
            lint_count("Das ist ein Satz. dieser hier beginnt klein und ist wirklich falsch."),
            1
        );
    }
}
