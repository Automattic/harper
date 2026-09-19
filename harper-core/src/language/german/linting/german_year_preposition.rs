use crate::language::german::spell::lexical_classes::UNIT_ABBREVIATIONS;
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Token, TokenKind, TokenStringExt, document::Document};

/// The range a bare four-digit number is read as a year in.
const EARLIEST_YEAR: f64 = 1000.0;
const LATEST_YEAR: f64 = 2999.0;

/// Catches `in 2024`, the English year construction.
///
/// German puts the year on its own — *"2024 wurde das Gebäude saniert"* — or
/// spells the preposition out, *"im Jahr 2024"*. The bare `in` in front of it is
/// an anglicism, and a common one in business writing.
///
/// A number after `in` is not always a year, and the word on either side says so:
///
/// * *"in 2024 **Fällen**"* is a count — a noun follows, and German capitalizes
///   its nouns;
/// * *"bis in 2000 **m** Höhe"* is a measurement — a unit follows, or `bis`
///   opens a range;
/// * *"Straubing in 1945: Turning Point in Its History"* is an English book
///   title — nothing lower case and German follows the number.
#[derive(Default)]
pub struct GermanYearPreposition;

impl GermanYearPreposition {
    fn is_year(token: &Token, document: &Document) -> bool {
        if !matches!(token.kind, TokenKind::Number(_)) {
            return false;
        }

        let text: String = document.get_span_content(&token.span).iter().collect();
        if text.len() != 4 || !text.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        text.parse::<f64>()
            .is_ok_and(|value| (EARLIEST_YEAR..=LATEST_YEAR).contains(&value))
    }

    /// Is the number something other than a year — a count or a measurement?
    fn is_not_a_year_after_all(tokens: &[&Token], index: usize, document: &Document) -> bool {
        let Some(next) = tokens.get(index + 1) else {
            return true;
        };

        // A year in a German sentence is followed by the rest of that sentence,
        // which starts lower case. A capital means a noun (a count) or an
        // English title; punctuation means a title or a citation.
        if !matches!(next.kind, TokenKind::Word(_)) {
            return true;
        }

        let word: String = document.get_span_content(&next.span).iter().collect();
        if word.chars().next().is_some_and(char::is_uppercase) {
            return true;
        }

        UNIT_ABBREVIATIONS.contains(&word)
    }

    /// Does `bis` open a range in front of it? *"bis in 2000 m Höhe"*.
    fn opens_a_range(tokens: &[&Token], index: usize, document: &Document) -> bool {
        index.checked_sub(1).is_some_and(|i| {
            let word: String = document
                .get_span_content(&tokens[i].span)
                .iter()
                .flat_map(|c| c.to_lowercase())
                .collect();
            word == "bis"
        })
    }
}

impl Linter for GermanYearPreposition {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|t| !t.kind.is_whitespace())
                .collect();

            for (index, token) in tokens.iter().enumerate() {
                if !matches!(token.kind, TokenKind::Word(_)) {
                    continue;
                }

                let word: String = document
                    .get_span_content(&token.span)
                    .iter()
                    .flat_map(|c| c.to_lowercase())
                    .collect();
                if word != "in" {
                    continue;
                }

                let Some(year) = tokens.get(index + 1) else {
                    continue;
                };

                if !Self::is_year(year, document)
                    || Self::is_not_a_year_after_all(&tokens, index + 1, document)
                    || Self::opens_a_range(&tokens, index, document)
                {
                    continue;
                }

                let digits: String = document.get_span_content(&year.span).iter().collect();
                let span = crate::Span::new(token.span.start, year.span.end);
                let capitalized = document
                    .get_span_content(&token.span)
                    .first()
                    .is_some_and(|c| c.is_uppercase());
                let spelled_out = if capitalized {
                    format!("Im Jahr {digits}")
                } else {
                    format!("im Jahr {digits}")
                };

                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Grammar,
                    suggestions: vec![
                        Suggestion::ReplaceWith(digits.chars().collect()),
                        Suggestion::ReplaceWith(spelled_out.chars().collect()),
                    ],
                    priority: 29,
                    message: format!(
                        "Im Deutschen steht die Jahreszahl ohne »in«: »{digits}« oder \
                         »im Jahr {digits}«."
                    ),
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Streicht das englische »in« vor einer Jahreszahl (»in 2024« → »2024«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanYearPreposition;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanYearPreposition.lint(&document).len()
    }

    #[test]
    fn flags_the_anglicism() {
        for text in [
            "In 2024 wurde das Gebäude vollständig saniert.",
            "Der Umsatz ist in 2023 deutlich gestiegen.",
        ] {
            assert_eq!(lint_count(text), 1, "should fire on {text:?}");
        }
    }

    #[test]
    fn leaves_the_german_constructions_alone() {
        for text in [
            "2024 wurde das Gebäude vollständig saniert.",
            "Im Jahr 2024 wurde das Gebäude saniert.",
            "Das Gebäude wurde 2024 saniert.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }

    #[test]
    fn leaves_a_count_alone() {
        for text in [
            "Das trat in 2024 Fällen auf.",
            "Der Fehler zeigte sich in 1200 Messungen.",
            // A measurement, and a range that opens with "bis".
            "Man findet den Täubling bis in 2000 m Höhe.",
            // An English book title in a citation.
            "Anonymus: Straubing in 1945: Turning Point in Its History.",
        ] {
            assert_eq!(lint_count(text), 0, "should not fire on {text:?}");
        }
    }
}
