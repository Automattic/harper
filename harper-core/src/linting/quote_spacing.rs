use crate::{
    expr::{ExprExt, SequenceExpr},
    linting::{Lint, LintKind, Linter},
    {Document, Punctuation, TokenKind, TokenStringExt},
};

pub struct QuoteSpacing {
    expr: SequenceExpr,
}

impl QuoteSpacing {
    pub fn new() -> Self {
        Self {
            expr: SequenceExpr::any_word().then_quote().then_any_word(),
        }
    }
}

impl Default for QuoteSpacing {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for QuoteSpacing {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for m in self.expr.iter_matches_in_doc(document) {
            let matched_tokens = m.get_content(document.get_tokens());

            let Some(_span) = matched_tokens.span() else {
                continue;
            };

            let quote_tok = &matched_tokens[1];

            // Is this an opening or closing quote?
            let quote_index = m.start + 1; // Quote is the second token in the match
            let is_closing = if let TokenKind::Punctuation(Punctuation::Quote(q)) = quote_tok.kind {
                // If twin_loc points backward, this is a closing quote
                q.twin_loc.map_or(false, |twin| twin < quote_index)
            } else {
                false
            };

            let mut suggestions = vec![
                super::Suggestion::ReplaceWith(vec![' ', '"']),
                super::Suggestion::ReplaceWith(vec!['"', ' ']),
            ];

            // For closing quotes, space likely should be after it
            if is_closing {
                suggestions.swap(0, 1);
            }

            lints.push(Lint {
                span: matched_tokens[1].span,
                lint_kind: LintKind::Formatting,
                suggestions,
                message: "A quote must be preceded or succeeded by a space.".to_owned(),
                priority: 31,
            })
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks that quotation marks are preceded or succeeded by whitespace."
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteSpacing;
    use crate::linting::tests::{
        assert_first_suggestion_result, assert_lint_count, assert_no_lints,
        assert_suggestion_result,
    };

    #[test]
    fn flags_missing_space_before_quote() {
        assert_lint_count("He said\"hello\" to me.", QuoteSpacing::default(), 1);
    }

    #[test]
    fn flags_missing_space_after_quote() {
        assert_lint_count(
            "She whispered \"hurry\"and left.",
            QuoteSpacing::default(),
            1,
        );
    }

    #[test]
    fn allows_quotes_with_spacing() {
        assert_no_lints("They shouted \"charge\" together.", QuoteSpacing::default());
    }

    #[test]
    fn allows_quotes_at_end_of_sentence() {
        assert_no_lints("They shouted \"charge.\"", QuoteSpacing::default());
    }

    #[test]
    fn fix_by_inserting_space_after() {
        assert_suggestion_result(
            "It's not a complete sentence since it lacks a verb but it would be valid as a title or answer to a question since \"dog runs\"are things which exist",
            QuoteSpacing::default(),
            "It's not a complete sentence since it lacks a verb but it would be valid as a title or answer to a question since \"dog runs\" are things which exist",
        );
    }

    #[test]
    fn ensure_first_suggestion_is_space_then_quote() {
        assert_first_suggestion_result(
            "foo\"bar\" bar",
            QuoteSpacing::default(),
            "foo \"bar\" bar",
        );
    }

    #[test]
    fn ensure_first_suggestion_is_quote_then_space() {
        assert_first_suggestion_result(
            "foo \"bar\"bar",
            QuoteSpacing::default(),
            "foo \"bar\" bar",
        );
    }

    #[test]
    #[should_panic]
    fn ensure_first_suggestion_is_not_quote_then_space() {
        assert_first_suggestion_result(
            "foo \"bar\"bar",
            QuoteSpacing::default(),
            "foo\" bar\" bar",
        );
    }

    #[test]
    fn ensure_quote_then_space_is_suggested_even_when_wrong() {
        assert_suggestion_result("foo\"bar\" bar", QuoteSpacing::default(), "foo\" bar\" bar");
    }
}
