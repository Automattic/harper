//! Checks that a determiner carries the gender of the noun it introduces.

use std::sync::Arc;

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::determiners::{
        DeterminerReading, determiner_readings, forms_with_gender, stands_alone,
    },
    language::german::grammar::noun_phrase::{self, Phrase, opens_relative_clause},
    language::german::spell::curated_german_dictionary,
    language::morphology::{Gender, MorphologyExt, NumberSet},
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::{Dictionary, FstDictionary},
};

/// Endings of a noun whose plural may be spelled like its singular (*der
/// Lehrer*, *die Lehrer*; *das Mädchen*, *die Mädchen*; *der Kuchen*). The
/// dictionary records some of these as singular only, so the spelling decides.
const SAME_AS_PLURAL: &[&str] = &["en", "er", "el"];

/// Catches an article that does not fit its noun's gender: *"**die** Hund"*,
/// *"**das** Schule"*, *"**ein** Frau"*.
///
/// This is the mistake `readings_allowed_by` in `grammar/determiners.rs`
/// deliberately does not describe: when the noun rules out every reading of the
/// determiner, the preposition rule has no case error to name, and nothing else
/// looks. It is the most taught gap in German writing and, by the same token,
/// the one where a wrong dictionary entry turns a correct phrase into a report,
/// so three conditions have to hold together.
///
/// * **The gender is recorded as exactly one.** An entry with two (*der/das
///   Teil*) narrows nothing, and a noun without one is skipped.
/// * **The noun is surely a singular**, wherever the article has a plural
///   reading. *die* is also the plural article, so *die Hund* is only wrong if
///   *Hund* cannot be a plural; a base entry marked singular that does not end
///   like a plural qualifies, *Lehrer* and *Mädchen* do not. Without this every
///   *die Kinder* would be a report. *das* and *ein* have no plural, so for
///   them the gender alone decides.
/// * **The chunker has shown the phrase to end at the noun.** The head comes
///   from the same place the preposition rule reads it, for the same reason:
///   scanning forward for the first capital crosses clause boundaries.
///
/// A determiner opening a relative clause is a pronoun (*Frauen, die Mut
/// haben*) and is left alone, as is a capitalized one in mid-sentence, which is
/// part of a name (*Die Zeit*).
///
/// Only the determiner is read. An adjective ending that contradicts the noun
/// (*ein großes Hund*) is not checked.
pub struct GermanDeterminerGender {
    /// The base dictionary, not the compound-aware one, for the reason the
    /// preposition rule gives: a decomposition invents readings.
    dictionary: Arc<FstDictionary>,
}

impl Default for GermanDeterminerGender {
    fn default() -> Self {
        Self::new()
    }
}

impl GermanDeterminerGender {
    pub fn new() -> Self {
        Self {
            dictionary: curated_german_dictionary(),
        }
    }

    /// The one gender the dictionary records for `head`, and whether the noun
    /// is surely a singular.
    ///
    /// "Surely" means a base entry marked singular whose spelling does not
    /// allow it to be a plural as well: *Hund* and *Haus* are, *Lehrer* and
    /// *Mädchen* are not, and *Schule* is not either, since the entry has both
    /// numbers.
    fn gender_of(&self, head: &str) -> Option<(Gender, bool)> {
        let chars: Vec<char> = head.chars().collect();
        let metadata = self
            .dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())?;
        let agreement = metadata.noun_agreement();

        // `unique` is `None` for an empty set and for one with several genders.
        let gender = agreement.gender.unique()?;
        let lower = head.to_lowercase();
        // An acronym inflects for nothing: *die AGB*.
        let acronym = head.chars().all(|c| !c.is_lowercase());
        let surely_singular = agreement.number == NumberSet::SINGULAR
            && !acronym
            && !SAME_AS_PLURAL.iter().any(|ending| lower.ends_with(ending));

        Some((gender, surely_singular))
    }

    /// The noun that ends `phrase`, when the chunker's choice can be trusted.
    ///
    /// The preposition rule asks for a function word or punctuation behind the
    /// head, which is the right bar for a case error and the wrong one here:
    /// *die Hund gesehen* ends on a participle and is exactly what this rule is
    /// for. What has to be ruled out instead is a head that is only the first
    /// capital of a longer run — *den Berliner Philharmonikern*, *die Deutsche
    /// Bahn* — so a capitalized word behind it rejects the phrase, as does a
    /// hyphen, which makes the head half of a compound.
    fn head_of(document: &Document, words: &[&Token], phrase: &Phrase) -> Option<String> {
        let token = words[phrase.head];
        if !matches!(token.kind, TokenKind::Word(_)) {
            return None;
        }

        let content = document.get_full_content();
        let touches_hyphen = content.get(token.span.end) == Some(&'-')
            || (token.span.start > 0 && content.get(token.span.start - 1) == Some(&'-'));
        if touches_hyphen {
            return None;
        }

        let capitalized = |token: &Token| {
            matches!(token.kind, TokenKind::Word(_))
                && document
                    .get_span_content(&token.span)
                    .first()
                    .is_some_and(|c| c.is_uppercase())
        };
        if words
            .get(phrase.head + 1)
            .is_some_and(|next| capitalized(next))
        {
            return None;
        }

        let head: String = document.get_span_content(&token.span).iter().collect();
        head.chars()
            .next()
            .is_some_and(char::is_uppercase)
            .then_some(head)
    }

    /// German name of a gender, for the message.
    fn label(gender: Gender) -> &'static str {
        match gender {
            Gender::Masculine => "maskulin",
            Gender::Feminine => "feminin",
            Gender::Neuter => "neutral",
        }
    }
}

/// Whether any singular reading of the determiner carries `gender`.
fn fits(readings: &[DeterminerReading], gender: Gender) -> bool {
    readings
        .iter()
        .any(|reading| reading.gender == Some(gender))
}

impl Linter for GermanDeterminerGender {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            let phrases = noun_phrase::phrases(&words, document);
            let first_word = words
                .iter()
                .position(|token| matches!(token.kind, TokenKind::Word(_)));

            for phrase in &phrases {
                let determiner = words[phrase.open];
                if !matches!(determiner.kind, TokenKind::Word(_)) {
                    continue;
                }
                let text: String = document.get_span_content(&determiner.span).iter().collect();

                // *Die Zeit*, *Der Spiegel*: a capital in mid-sentence is a name.
                if text.chars().next().is_some_and(char::is_uppercase)
                    && first_word != Some(phrase.open)
                {
                    continue;
                }
                let Some(readings) = determiner_readings(&text) else {
                    continue;
                };
                if stands_alone(&text) || opens_relative_clause(&words, phrase.open, document) {
                    continue;
                }

                let Some(head) = Self::head_of(document, &words, phrase) else {
                    continue;
                };
                let Some((gender, surely_singular)) = self.gender_of(&head) else {
                    continue;
                };

                // *die* and *der* are plural readings too. They only rule the
                // noun out when it cannot be one; *das* and *ein* never could.
                let candidates: Vec<DeterminerReading> = readings
                    .iter()
                    .copied()
                    .filter(|reading| reading.gender.is_some() || !surely_singular)
                    .collect();
                if fits(&candidates, gender) {
                    continue;
                }
                if candidates.iter().any(|reading| reading.gender.is_none()) {
                    continue;
                }

                let suggestions: Vec<Suggestion> = forms_with_gender(&candidates, gender)
                    .into_iter()
                    .map(|form| {
                        Suggestion::replace_with_match_case(
                            form.chars().collect(),
                            document.get_span_content(&determiner.span),
                        )
                    })
                    .collect();
                if suggestions.is_empty() {
                    continue;
                }

                lints.push(Lint {
                    span: determiner.span,
                    lint_kind: LintKind::Agreement,
                    suggestions,
                    message: format!(
                        "»{text}« passt nicht zu »{head}«. Das Nomen ist {}.",
                        Self::label(gender)
                    ),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Prüft, ob der Artikel zum Geschlecht des Nomens passt."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanDeterminerGender;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::Linter;

    fn lints(text: &str) -> Vec<(String, Vec<String>)> {
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanDeterminerGender::new()
            .lint(&document)
            .into_iter()
            .map(|lint| {
                (
                    document.get_span_content_str(&lint.span),
                    lint.suggestions
                        .iter()
                        .map(|suggestion| suggestion.to_string())
                        .collect(),
                )
            })
            .collect()
    }

    fn reported(text: &str) -> Vec<String> {
        lints(text).into_iter().map(|(word, _)| word).collect()
    }

    #[test]
    fn a_wrong_article_is_reported() {
        assert_eq!(reported("Ich habe die Hund gesehen."), ["die"]);
        assert_eq!(reported("Das Schule ist groß."), ["Das"]);
        assert_eq!(reported("Er liest der Buch."), ["der"]);
        assert_eq!(reported("Ich sehe das Mann."), ["das"]);
        assert_eq!(reported("Die Hund bellt laut."), ["Die"]);
    }

    #[test]
    fn the_correction_keeps_the_case_and_the_paradigm() {
        let found = lints("Ich habe die Hund gesehen.");
        assert!(found[0].1.iter().any(|s| s.contains("den")), "{found:?}");
    }

    #[test]
    fn a_correct_article_is_quiet() {
        for text in [
            "Der Hund bellt laut.",
            "Ich sehe den Hund.",
            "Das Buch liegt auf dem Tisch.",
            "Die Schule beginnt um acht.",
            "Ich habe einen Hund und eine Katze.",
            "Er gibt dem Kind das Buch.",
            "Der Mann liest die Zeitung.",
        ] {
            assert!(reported(text).is_empty(), "{text}: {:?}", reported(text));
        }
    }

    /// *die* is also the plural article, so it fits any noun that can be one.
    #[test]
    fn a_plural_article_is_not_a_gender_error() {
        for text in [
            "Die Kinder spielen im Garten.",
            "Die Lehrer kommen später.",
            "Die Mädchen lachen.",
            "Ich sehe die Hunde.",
        ] {
            assert!(reported(text).is_empty(), "{text}: {:?}", reported(text));
        }
    }

    /// *Frauen, die Mut haben*: the article is a relative pronoun.
    #[test]
    fn a_relative_pronoun_is_not_an_article() {
        for text in [
            "Es gibt Frauen, die Mut haben.",
            "Das ist ein Mann, den Hund und Katze lieben.",
        ] {
            assert!(reported(text).is_empty(), "{text}: {:?}", reported(text));
        }
    }

    #[test]
    fn a_capitalized_article_inside_a_name_is_left_alone() {
        assert!(reported("Ich lese Die Zeit jeden Tag.").is_empty());
    }

    /// An entry with two genders narrows nothing.
    #[test]
    fn a_noun_with_two_genders_is_not_checked() {
        let found = reported("Das Teil und der Teil gehören zusammen.");
        assert!(found.is_empty(), "{found:?}");
    }
}
