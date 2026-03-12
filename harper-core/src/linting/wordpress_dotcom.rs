use crate::{CharString, CharStringExt, TokenStringExt};

use super::{Lint, LintKind, Linter, Suggestion};

/// Make sure you properly capitalize `WordPress.com`.
#[derive(Default)]
pub struct WordPressDotcom;

impl Linter for WordPressDotcom {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let correct: CharString = "WordPress.com".chars().collect();
        let correct_lower = correct.to_lower();
        let mut lints = Vec::new();

        for hostname in document.iter_hostnames() {
            let text = document.get_span_content(&hostname.span);

            if correct.as_slice() != text && text.to_lower() == correct_lower {
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
        assert_suggestion_result("wordpress.com", WordPressDotcom, "WordPress.com");
    }

    #[test]
    fn sentence() {
        assert_suggestion_result(
            "wordpress.com is a great hosting provider",
            WordPressDotcom,
            "WordPress.com is a great hosting provider",
        );
    }
}
