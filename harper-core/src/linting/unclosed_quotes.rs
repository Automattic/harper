use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::{Punctuation, Quote, TokenKind};

#[derive(Debug, Clone, Copy, Default)]
pub struct UnclosedQuotes;

impl Linter for UnclosedQuotes {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // TODO: Try zipping quote positions
        for token in document.tokens() {
            if let TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc: None })) = token.kind
            {
                lints.push(Lint {
                    span: token.span,
                    lint_kind: LintKind::Formatting,
                    suggestions: vec![],
                    message: "This quote has no termination.".to_string(),
                    priority: 255,
                })
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Quotation marks should always be closed. Unpaired quotation marks are a hallmark of sloppy work."
    }
}

#[cfg(test)]
mod tests {
    use super::UnclosedQuotes;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    #[test]
    fn allows_dialogue_with_em_dash_interruption() {
        assert_no_lints(
            "\"It'll be our\"—she leaned to his ear—\"shared secret.\"",
            UnclosedQuotes::default(),
        );
    }

    #[test]
    fn still_flags_unclosed_quotes() {
        assert_lint_count("\"It'll be our", UnclosedQuotes::default(), 1);
    }
}
