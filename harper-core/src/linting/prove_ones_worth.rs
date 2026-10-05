use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct ProveOnesWorth {
    expr: All,
}

impl Default for ProveOnesWorth {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["prove", "proved", "proven", "proving"])
                .t_ws()
                .then_possessive_determiner()
                .t_ws()
                .t_aco("weight")
                .but_not(
                    SequenceExpr::anything() // prove
                        .t_any() // ws
                        .t_any() // their
                        .t_any() // ws
                        .t_any() // weight
                        .t_ws()
                        .t_set(&["back", "is", "loss", "on", "to", "was", "with"]),
                ),
        }
    }
}

impl ExprLinter for ProveOnesWorth {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let span = toks.last()?.span;
        let lint_kind = LintKind::WordChoice;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "worth",
            span.get_content(src),
        )];
        let message = "The correct word in this idiom is `worth`, not `weight`.".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `prove one's weight` to `prove one's worth ."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ProveOnesWorth;

    #[test]
    fn dont_flag_prove_her_weight_to() {
        assert_no_lints(
            "when she needed to prove her weight to Cersei, she used ...",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_his_weight_loss() {
        assert_no_lints(
            "Gets to prove his weight loss was all due to hard work!!",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_its_weight_is() {
        assert_no_lints(
            "given the refined hypothesis, construct the explicit witness and prove its weight is (4/9)|G|.",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_my_weight_back() {
        assert_no_lints(
            "I didn't have to prove my weight back then.",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_my_weight_on() {
        assert_no_lints(
            "making me video chat to prove my weight on a scale",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_my_weight_was() {
        assert_no_lints(
            "taking the photos to prove my weight was such a psychologically difficult thing",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_their_weight() {
        assert_no_lints(
            "surely a defendant should have the right to simply prove their weight.",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_prove_your_weight() {
        assert_no_lints(
            "When getting online prescription for semaglutide do you have to prove your weight?",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn proved_his() {
        assert_suggestion_result(
            "Morrissey had more than proved his weight as a solo artist",
            ProveOnesWorth::default(),
            "Morrissey had more than proved his worth as a solo artist",
        )
    }

    #[test]
    fn proved_its() {
        assert_suggestion_result(
            "Rust has also proved its weight as a language with the performance improvements it’s given to firefox with Project Quantum.",
            ProveOnesWorth::default(),
            "Rust has also proved its worth as a language with the performance improvements it’s given to firefox with Project Quantum.",
        );
    }

    #[test]
    fn proved_their() {
        assert_suggestion_result(
            "Across ~10 deployments (April–May 2026) the following patterns proved their weight",
            ProveOnesWorth::default(),
            "Across ~10 deployments (April–May 2026) the following patterns proved their worth",
        )
    }

    #[test]
    fn proven_its() {
        assert_suggestion_result(
            "and it absolutely has more than proven its weight as belonging in base",
            ProveOnesWorth::default(),
            "and it absolutely has more than proven its worth as belonging in base",
        )
    }

    #[test]
    fn dont_flag_proving_my_weight_loss() {
        assert_no_lints(
            "Shorts from the 80's resurrected-proving my weight loss strategy worked!!",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_proving_your_weight_to() {
        assert_no_lints(
            "Sub 250g any tips for proving your weight to police?",
            ProveOnesWorth::default(),
        )
    }

    // Known edge cases

    #[test]
    fn dont_flag_mixed_metaphor_gold() {
        // mixes up: "proven its worth" and "worth its weight in gold"
        assert_no_lints(
            "Each principle you shared has proven its weight in gold",
            ProveOnesWorth::default(),
        )
    }

    #[test]
    fn dont_flag_mixed_metaphor_silver() {
        assert_no_lints(
            "and it's already proving its weight in silver",
            ProveOnesWorth::default(),
        )
    }
}
