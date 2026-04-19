use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct CloseTightKnit {
    expr: SequenceExpr,
}

impl Default for CloseTightKnit {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["close", "closely", "tight", "tightly"])
                .t_ws_h()
                .t_aco("nit"),
        }
    }
}

impl ExprLinter for CloseTightKnit {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (sep_tok, nit_tok) = (&toks[1], &toks[2]);

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "knit",
            nit_tok.get_ch(src),
        )];
        let message = format!(
            "A `nit` is a louse egg. The correct idiom is `tight{}knit`.",
            if sep_tok.kind.is_hyphen() { '-' } else { ' ' }
        );

        Some(Lint {
            span: nit_tok.span,
            lint_kind: LintKind::Malapropism,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `close-nit` and `tight-nit` to `close-knit` and `tight-knit`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::CloseTightKnit;

    #[test]
    fn fix_tight_nit_space() {
        assert_suggestion_result(
            "To keep components theme-able and tight nit, an ideal scenario",
            CloseTightKnit::default(),
            "To keep components theme-able and tight knit, an ideal scenario",
        );
    }

    #[test]
    fn fix_tight_nit_hyphenated() {
        assert_suggestion_result(
            "We're a small, tight-nit, experienced team from Google, Optimizely, Uber, & DraftKings",
            CloseTightKnit::default(),
            "We're a small, tight-knit, experienced team from Google, Optimizely, Uber, & DraftKings",
        );
    }

    #[test]
    fn fix_close_nit_space() {
        assert_suggestion_result(
            "I get its all a close nit set of Github folks driving both projects but...",
            CloseTightKnit::default(),
            "I get its all a close knit set of Github folks driving both projects but...",
        );
    }

    #[test]
    fn fix_close_nit_hyphen() {
        assert_suggestion_result(
            "Within a group of more senior engineers working on close-nit teams (like MX) there's a lot of trust in the author during the review process.",
            CloseTightKnit::default(),
            "Within a group of more senior engineers working on close-knit teams (like MX) there's a lot of trust in the author during the review process.",
        );
    }

    #[test]
    fn fix_tightly_nit_space() {
        assert_suggestion_result(
            "Initially public keys and aliases are tightly nit, changes in key values break expectations of EVM authorization checks",
            CloseTightKnit::default(),
            "Initially public keys and aliases are tightly knit, changes in key values break expectations of EVM authorization checks",
        );
    }

    #[test]
    fn fix_tightly_nit_hyphen() {
        assert_suggestion_result(
            "... will result, in my opinion and from what I have witnessed, in a more tightly-nit community",
            CloseTightKnit::default(),
            "... will result, in my opinion and from what I have witnessed, in a more tightly-knit community",
        );
    }

    #[test]
    fn fix_closely_nit_space() {
        assert_suggestion_result(
            "For a small family, the boys have a nice network of closely nit family members.",
            CloseTightKnit::default(),
            "For a small family, the boys have a nice network of closely knit family members.",
        );
    }

    #[test]
    fn fix_closely_nit_hyphen() {
        assert_suggestion_result(
            "However this time around we are aiming for a much smaller more \"closely-nit\" town.",
            CloseTightKnit::default(),
            "However this time around we are aiming for a much smaller more \"closely-knit\" town.",
        );
    }
}
