use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct CeaseTo {
    expr: SequenceExpr,
}

impl Default for CeaseTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                "seize", "seized", "seizes", "seizing", "sieze", "siezed", "siezes", "siezing",
            ])
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then_verb_lemma(),
        }
    }
}

impl ExprLinter for CeaseTo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let seize = toks.first()?;
        let seize = seize.get_ch(src).get(4..)?;

        let cease: Vec<char> = ['c', 'e', 'a', 's']
            .into_iter()
            .chain(seize.iter().copied())
            .collect();

        let span = seize.span;

        let suggestions = vec![Suggestion::replace_with_match_case(
            cease,
            span.get_content(src),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Did you mean `cease to` (stop)? If you meant `seize to` (e.g., assets seized to cover a debt), you can ignore this.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects when `seize to` is likely a typo for `cease to`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CeaseTo;

    // Genuine errors

    #[test]
    fn seize_to_exist() {
        assert_suggestion_result(
            "The moment you add webbrowser control, all other controls on the Page seize to exist.",
            CeaseTo::default(),
            "The moment you add webbrowser control, all other controls on the Page cease to exist.",
        )
    }

    #[test]
    fn seize_to_impress() {
        assert_suggestion_result(
            "Considering the project is considered feature complete, you seize to impress me with new functionality, that just works.",
            CeaseTo::default(),
            "Considering the project is considered feature complete, you cease to impress me with new functionality, that just works.",
        )
    }

    #[test]
    fn sieze_to_be() {
        assert_suggestion_result(
            "When do the photons emerging from the Eiffel tower sieze to be copyrighted?",
            CeaseTo::default(),
            "When do the photons emerging from the Eiffel tower cease to be copyrighted?",
        )
    }

    #[test]
    fn seized_to_be() {
        assert_suggestion_result(
            "Though I love Chrome as a browser, at this particular point GWT Plugin / Chrome combo seized to be an asset and became a liability :(.",
            CeaseTo::default(),
            "Though I love Chrome as a browser, at this particular point GWT Plugin / Chrome combo ceased to be an asset and became a liability :(.",
        );
    }

    #[test]
    fn seized_to_function() {
        assert_suggestion_result(
            "upgrading Apache and PHP to the latest versions, PHPMyAdmin seized to function",
            CeaseTo::default(),
            "upgrading Apache and PHP to the latest versions, PHPMyAdmin ceased to function",
        )
    }

    #[test]
    fn seizes_to_be() {
        assert_suggestion_result(
            "Sure, the sandbox cannot be super restrictive, otherwise the terminal seizes to be useful.",
            CeaseTo::default(),
            "Sure, the sandbox cannot be super restrictive, otherwise the terminal ceases to be useful.",
        );
    }

    #[test]
    fn seizes_to_work() {
        assert_suggestion_result(
            "Recently, my Azure OpenAI configuartion seizes to work.",
            CeaseTo::default(),
            "Recently, my Azure OpenAI configuartion ceases to work.",
        )
    }

    #[test]
    fn never_siezes_to_amaze() {
        assert_suggestion_result(
            "Number of ppl in our industry who don’t seem to be aware of the Brooks’ Law never siezes to amaze me.",
            CeaseTo::default(),
            "Number of ppl in our industry who don’t seem to be aware of the Brooks’ Law never ceases to amaze me.",
        )
    }

    #[test]
    fn seizing_to_be() {
        assert_suggestion_result(
            "sharing much of their genetics, and seizing to be very similar genetically",
            CeaseTo::default(),
            "sharing much of their genetics, and ceasing to be very similar genetically",
        )
    }

    #[test]
    fn seizing_to_function() {
        assert_suggestion_result(
            "Task feature timeout_seconds seizing to function as intended after update",
            CeaseTo::default(),
            "Task feature timeout_seconds ceasing to function as intended after update",
        );
    }

    #[test]
    fn siezing_to_respond() {
        assert_suggestion_result(
            "and now i cant save without the game siezing to respond after 30 seconds",
            CeaseTo::default(),
            "and now i cant save without the game ceasing to respond after 30 seconds",
        )
    }

    // Legitimate uses of "seize to" - checks against false positives

    #[test]
    #[ignore = "can't clock legit verb yet"]
    fn dont_flag_seized_to_be_made() {
        // TODO: maybe try looking for "have" in the before context?
        assert_no_lints(
            "In order to accomplish this the tasks have their ownership seized to be made the property of the user running the script.",
            CeaseTo::default(),
        )
    }

    #[test]
    #[ignore = "can't clock legit verb yet"]
    fn dont_flag_collateral_seized_to_cover_a_debt() {
        assert_no_lints(
            "When a borrower's collateral on Chain A is seized to cover a debt on Chain B, Chain A's CrossChainRouter sends a LiquidationSuccess message",
            CeaseTo::default(),
        )
    }

    #[test]
    #[ignore = "can't clock legit verb yet"]
    fn dont_flag_wealth_siezed_to_save_ppl() {
        assert_no_lints(
            "software devlopers, who make vastly more than the average person should have their wealth siezed to save those people?",
            CeaseTo::default(),
        )
    }

    #[test]
    fn dont_flag_nut_siezing_to_the_screw() {
        assert_no_lints(
            "Most of the problem is the nut siezing to the screw.",
            CeaseTo::default(),
        )
    }

    // Different mistakes that can trigger false positives

    #[test]
    #[ignore = "can't clock typo"]
    fn typo_for_sizes() {
        assert_no_lints(
            "Would work, if the body width is already set at 99% of screen size, which makes all possible seizes to be limited to the 99% of screen width",
            CeaseTo::default(),
        );
    }

    // Known false positives where seize is used as a noun

    #[test]
    fn dont_flag_maps_seize_to() {
        assert_no_lints(
            "Win32ControlUnitMgr maps Seize to SeizeInput(hwnd_, false)",
            CeaseTo::default(),
        )
    }

    #[test]
    #[ignore = "can't clock seize used as noun yet"]
    fn executes_seize_to_take() {
        assert_no_lints(
            "Lender-miner immediately executes seize to take the borrower's collateral.",
            CeaseTo::default(),
        )
    }

    // Too hard to understand to know if it's a mistake or legit

    #[test]
    fn cant_grok_may_seize_to_any() {
        assert_no_lints(
            "defaults to always-allow when unset, so an unconfigured token may seize to any destination",
            CeaseTo::default(),
        )
    }
}
