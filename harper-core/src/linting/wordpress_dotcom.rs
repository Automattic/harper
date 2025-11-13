use crate::expr::{ExprExt, SequenceExpr};
use crate::patterns::WhitespacePattern;
use crate::punctuation::Punctuation;
use crate::{CharString, CharStringExt, Span, TokenKind, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

/// Make sure you properly capitalize `WordPress.com`.
pub struct WordPressDotcom {
    patterns: Vec<SequenceExpr>,
}

impl Default for WordPressDotcom {
    fn default() -> Self {
        let split_pattern = SequenceExpr::aco("WordPress")
            .then_optional(WhitespacePattern)
            .then_strict(TokenKind::Punctuation(Punctuation::Period))
            .then_optional(WhitespacePattern)
            .t_aco("com");

        Self {
            patterns: vec![split_pattern],
        }
    }
}

impl Linter for WordPressDotcom {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let correct: CharString = "WordPress.com".chars().collect();
        let correct_lower: CharString = "wordpress.com".chars().collect();
        let mut lints = Vec::new();

        for hostname in document.iter_hostnames() {
            let text = document.get_span_content(&hostname.span);

            if correct.as_slice() != text && text.to_lower().as_ref() == correct_lower.as_slice() {
                lints.push(Lint {
                    span: hostname.span,
                    lint_kind: LintKind::Style,
                    suggestions: vec![Suggestion::ReplaceWith(correct.to_vec())],
                    message: "The WordPress hosting provider should be stylized as `WordPress.com`"
                        .to_owned(),
                    priority: 31,
                });
            }
        }

        let source = document.get_source();

        let tokens = document.get_tokens();

        for pattern in &self.patterns {
            for match_span in pattern.iter_matches(tokens, source) {
                let first_token = &tokens[match_span.start];
                let final_token = &tokens[match_span.end - 1];
                let span = Span::new(first_token.span.start, final_token.span.end);
                let text = document.get_span_content(&span);

                if text == correct.as_slice() {
                    continue;
                }

                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Style,
                    suggestions: vec![Suggestion::ReplaceWith(correct.to_vec())],
                    message: "The WordPress hosting provider should be stylized as `WordPress.com`"
                        .to_owned(),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ensures correct capitalization of WordPress.com. This rule verifies that the official stylization of WordPress.com is used when referring to the hosting provider."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::WordPressDotcom;

    #[test]
    fn simple() {
        assert_suggestion_result("wordpress.com", WordPressDotcom::default(), "WordPress.com");
    }

    #[test]
    fn sentence() {
        assert_suggestion_result(
            "wordpress.com is a great hosting provider",
            WordPressDotcom::default(),
            "WordPress.com is a great hosting provider",
        );
    }

    #[test]
    fn lowercase_tokens_are_fixed() {
        assert_suggestion_result(
            "Wordpress.com is a great hosting provider",
            WordPressDotcom::default(),
            "WordPress.com is a great hosting provider",
        );
    }

    #[test]
    fn spaced_dot_recombines() {
        assert_suggestion_result(
            "WordPress . com is a great hosting provider",
            WordPressDotcom::default(),
            "WordPress.com is a great hosting provider",
        );
    }
}
