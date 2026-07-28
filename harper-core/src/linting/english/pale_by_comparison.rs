use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::InflectionOfBe,
};

pub struct PaleByComparison {
    expr: SequenceExpr,
}

impl Default for PaleByComparison {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::with(InflectionOfBe::default())
                .t_ws()
                .t_aco("pale")
                .t_ws()
                .t_set(&["by", "in"])
                .t_ws()
                .t_aco("comparison"),
        }
    }
}

impl ExprLinter for PaleByComparison {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (be_idx, pale_idx) = (0, 2);
        let be_tok = &toks[be_idx];
        let be_pale_toks = &toks[be_idx..=pale_idx];

        let be_chars = be_tok.get_ch(src);

        let be_form_to_verb_form = &[
            ("am", "pale"),
            ("are", "pale"),
            ("be", "pale"),
            ("been", "paled"),
            ("being", "paling"),
            ("is", "pales"),
            ("was", "paled"),
            ("were", "paled"),
        ];

        let (_, verb_form) = be_form_to_verb_form
            .iter()
            .find(|&&(be, _)| be_chars.eq_str(be))?;

        let span = be_pale_toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case(
                verb_form.chars().collect(),
                span.get_content(src),
            )],
            message: "In this idiom, the word `pale` is correctly a verb, not an adjective."
                .to_owned(),
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

    use super::PaleByComparison;

    #[test]
    fn fix_are_pale_in() {
        assert_suggestion_result(
            "Admittedly, my R skills are pale in comparison to many of you guys, but this is a project I could undertake.",
            PaleByComparison::default(),
            "Admittedly, my R skills pale in comparison to many of you guys, but this is a project I could undertake.",
        );
    }

    #[test]
    fn fix_be_pale_in() {
        assert_suggestion_result(
            "their production would be pale in comparison to what we currently produce.",
            PaleByComparison::default(),
            "their production would pale in comparison to what we currently produce.",
        );
    }

    #[test]
    fn fix_is_pale_by() {
        assert_suggestion_result(
            "The computed dollar value is pale by comparison.",
            PaleByComparison::default(),
            "The computed dollar value pales by comparison.",
        );
    }

    #[test]
    fn fix_is_pale_in() {
        assert_suggestion_result(
            "Humans have problem to grokk exponential growth which is pale in comparison to Ackermann function.",
            PaleByComparison::default(),
            "Humans have problem to grokk exponential growth which pales in comparison to Ackermann function.",
        );
    }

    #[test]
    fn fix_was_pale_in() {
        assert_suggestion_result(
            "if you took the same actions to remove the violence and sexual fantasy out of Shakespeare, which was pale in comparison",
            PaleByComparison::default(),
            "if you took the same actions to remove the violence and sexual fantasy out of Shakespeare, which paled in comparison",
        );
    }

    #[test]
    fn fix_was_pale_by() {
        assert_suggestion_result(
            "What Microsoft did in the 90s was pale by comparison—they just bundled some extra software.",
            PaleByComparison::default(),
            "What Microsoft did in the 90s paled by comparison—they just bundled some extra software.",
        );
    }

    #[test]
    fn fix_were_pale_in() {
        assert_suggestion_result(
            "It's not just that other experiences were pale in comparison to what she'd been doing, it's likely she was chemically incapable of feeling",
            PaleByComparison::default(),
            "It's not just that other experiences paled in comparison to what she'd been doing, it's likely she was chemically incapable of feeling",
        );
    }
}
