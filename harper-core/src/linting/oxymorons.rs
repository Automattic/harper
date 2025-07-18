use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::FixedPhrase;
use crate::linting::{ExprLinter, Lint, LintKind};
use crate::{Token, TokenStringExt};

/// A linter that flags oxymoronic phrases.
pub struct Oxymorons {
    expr: Box<dyn Expr>,
}

impl Oxymorons {
    pub fn new() -> Self {
        // List of phrases that are considered oxymoronic.
        let phrases = vec![
            "amateur expert",
            "increasingly less",
            "advancing backwards?",
            "alludes explicitly to",
            "explicitly alludes to",
            "totally obsolescent",
            "completely obsolescent",
            "generally always",
            "usually always",
            "build down",
            "conspicuous absence",
            "exact estimate",
            "found missing",
            "intense apathy",
            "mandatory choice",
            "nonworking mother",
            "organized mess",
        ];

        // Build a vector of exact-match patterns for each oxymoron.
        let exprs: Vec<Box<dyn Expr>> = phrases
            .into_iter()
            .map(|s| Box::new(FixedPhrase::from_phrase(s)) as Box<dyn Expr>)
            .collect();

        let expr = Box::new(FirstMatchOf::new(exprs));
        Self { expr }
    }
}

impl Default for Oxymorons {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for Oxymorons {
    /// Returns the underlying pattern.
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched_text: String = span.get_content(source).iter().collect();
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: Vec::new(),
            message: format!("'{matched_text}' is an oxymoron."),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Flags oxymoronic phrases (e.g. `amateur expert`, `increasingly less`, etc.)."
    }
}

#[cfg(test)]
mod tests {
    use super::Oxymorons;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_amateur_expert() {
        assert_lint_count("The amateur expert gave his opinion.", Oxymorons::new(), 1);
    }

    #[test]
    fn detects_increasingly_less() {
        assert_lint_count(
            "The solution was increasingly less effective.",
            Oxymorons::new(),
            1,
        );
    }

    #[test]
    fn detects_advancing_backwards() {
        assert_lint_count("The project is advancing backwards?", Oxymorons::new(), 1);
    }

    #[test]
    fn detects_alludes_explicitly_to() {
        assert_lint_count(
            "The report alludes explicitly to several issues.",
            Oxymorons::new(),
            1,
        );
    }

    #[test]
    fn detects_explicitly_alludes_to() {
        assert_lint_count(
            "The report explicitly alludes to several issues.",
            Oxymorons::new(),
            1,
        );
    }

    #[test]
    fn does_not_flag_clean_text() {
        assert_lint_count("The expert provided clear advice.", Oxymorons::new(), 0);
    }

    #[test]
    fn lowercase_match() {
        assert_lint_count(
            "the amateur expert is often unreliable.",
            Oxymorons::new(),
            1,
        );
    }

    #[test]
    fn phrase_with_extra_whitespace() {
        assert_lint_count("An organized    mess was found.", Oxymorons::new(), 1);
    }

    #[test]
    fn phrase_split_by_line_break() {
        assert_lint_count(
            "nonworking\nmother is not a term to be used.",
            Oxymorons::new(),
            1,
        );
    }
}
