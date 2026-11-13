use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Sentence, followed_by_token, followed_by_word},
    },
};

pub struct OutOfCuriosity {
    expr: SequenceExpr,
}

impl Default for OutOfCuriosity {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["out", "of", "curiosities"]),
        }
    }
}

impl ExprLinter for OutOfCuriosity {
    type Unit = Sentence;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        enum To {
            Singular,
            Possessive,
        }

        let to: To = if followed_by_token(ctx, |t| t.kind.is_comma()) {
            To::Singular
        } else if followed_by_word(ctx, |wt| wt.get_ch(src).eq_str("sake")) {
            To::Possessive
        } else {
            return None;
        };

        let (sugg, msg): (&str, &str) = match to {
            To::Singular => ("curiosity", "This idiom uses the singular `curiosity`."),
            To::Possessive => (
                "curiosity's",
                "It looks like you mean the possessive `curiosity's`.",
            ),
        };

        let span = toks.get(4)?.span;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            sugg,
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message: msg.to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::OutOfCuriosity;

    #[test]
    fn fix_to_singular() {
        assert_suggestion_result(
            "Just out of curiosities, do you know why it has a combo of super high and super low reviews?",
            OutOfCuriosity::default(),
            "Just out of curiosity, do you know why it has a combo of super high and super low reviews?",
        );
    }

    #[test]
    fn fix_to_possessive() {
        assert_suggestion_result(
            "Out of curiosities sake, I also tested on my Win10 machine with the WSL.",
            OutOfCuriosity::default(),
            "Out of curiosity's sake, I also tested on my Win10 machine with the WSL.",
        );
    }
}
