use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct Misspell {
    expr: Box<dyn Expr>,
}

impl Default for Misspell {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["miss"]).t_ws_h().then_word_set(&[
            "spell",
            "spelled",
            "spelling",
            "spells",
            "spellings",
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for Misspell {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let misspell_variant = matched_tokens.last()?;

        let variant_chars = misspell_variant.span.get_content(source);
        let variant_lower = variant_chars
            .iter()
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();

        let replacement = match variant_lower.as_str() {
            "spell" => "misspell",
            "spelled" => "misspelled",
            "spelling" => "misspelling",
            "spells" => "misspells",
            "spellings" => "misspellings",
            _ => return None,
        };

        let suggestions = vec![Suggestion::replace_with_match_case(
            replacement.chars().collect(),
            span.get_content(source),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::BoundaryError,
            suggestions,
            message: "Write `misspell` and its inflections as a single word.".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures `misspell` and its inflected forms are written as a single word."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Misspell;

    #[test]
    fn base_form() {
        assert_suggestion_result(
            "They often miss spell names in the log.",
            Misspell::default(),
            "They often misspell names in the log.",
        );
    }

    #[test]
    fn past_tense() {
        assert_suggestion_result(
            "She miss spelled the answer on the quiz.",
            Misspell::default(),
            "She misspelled the answer on the quiz.",
        );
    }

    #[test]
    fn past_tense_hyphen() {
        assert_suggestion_result(
            "She miss-spelled the answer on the quiz.",
            Misspell::default(),
            "She misspelled the answer on the quiz.",
        );
    }

    #[test]
    fn gerund_form() {
        assert_suggestion_result(
            "His constant miss spelling frustrated the team.",
            Misspell::default(),
            "His constant misspelling frustrated the team.",
        );
    }
}
