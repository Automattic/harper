use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct AkinTo {
    expr: All,
}

impl Default for AkinTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["akin", "of"]).but_not(Word::new_exact("Akin")),
        }
    }
}

impl ExprLinter for AkinTo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let of = toks.last()?;

        Some(Lint {
            span: of.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                of.get_ch(src),
            )],
            message: "Did you mean `akin to`?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `akin of` to `akin to`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AkinTo;

    #[test]
    fn fix_lowercase() {
        assert_suggestion_result(
            "It was a bit more on the pricier side, but it would move side to side depending on what was going on in the game, turning this experience from a simple arcade game to something more akin of an amusement ride.",
            AkinTo::default(),
            "It was a bit more on the pricier side, but it would move side to side depending on what was going on in the game, turning this experience from a simple arcade game to something more akin to an amusement ride.",
        );
    }

    #[test]
    fn dont_flag_proper_noun() {
        assert_no_lints(
            "Krystal Akin of 'Mud Flower Mindful Movement & Counseling': How We Can Leverage The Power Of Gratitude",
            AkinTo::default(),
        );
    }
}
