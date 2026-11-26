use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, TokenKind, TokenStringExt};

#[derive(Default)]
pub struct LeadingZeroYear;

impl Linter for LeadingZeroYear {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let source = document.get_source();
        let is_identifier_char = |c: &char| c.is_alphanumeric() || *c == '_';

        document
            .iter_numbers()
            .filter_map(|token| {
                let TokenKind::Number(number) = token.kind else {
                    return None;
                };
                if number.radix != 10
                    || number.suffix.is_some()
                    || !(1700.0..=2100.0).contains(&number.value.0)
                {
                    return None;
                }

                let span = token.span;
                let digits = span.get_content(source);
                if digits.first() != Some(&'0') || !digits.iter().all(char::is_ascii_digit) {
                    return None;
                }

                // The lexer can split a number from a following identifier suffix.
                if source[..span.start].last().is_some_and(is_identifier_char)
                    || source.get(span.end).is_some_and(is_identifier_char)
                {
                    return None;
                }

                let year = digits.iter().copied().skip_while(|&c| c == '0');
                Some(Lint {
                    span,
                    lint_kind: LintKind::Formatting,
                    message: "If this is a year, remove the leading zeros.".to_owned(),
                    suggestions: vec![Suggestion::ReplaceWith(year.collect())],
                    ..Default::default()
                })
            })
            .collect()
    }

    fn description(&self) -> &'static str {
        "Flags leading zeros in numbers that could be years from 1700 to 2100."
    }
}
