use crate::{
    CharStringExt, Lint, Lrc, Token,
    expr::{AnchorEnd, Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_index_matching},
    },
};

pub struct AWaysToGo {
    expr: FirstMatchOf,
}

impl Default for AWaysToGo {
    fn default() -> Self {
        let prolog = SequenceExpr::aco("still").t_ws();

        let core = Lrc::new(
            SequenceExpr::word_set(&["have", "had", "has", "having"])
                .t_ws()
                .then_word_seq(&["ways", "to", "go"]),
        );

        let epilog = FirstMatchOf::new([
            Box::new(AnchorEnd) as Box<dyn Expr>,
            Box::new(SequenceExpr::whitespace().then_any_of([
                Box::new(SequenceExpr::word_set(&["before", "but", "though"])),
                Box::new(SequenceExpr::word_seq(&["in", "order", "to"])),
                Box::new(SequenceExpr::word_seq(&["to", "be"])),
            ])),
        ]);

        Self {
            expr: FirstMatchOf::new([
                Box::new(prolog.then(core.clone())),
                Box::new(SequenceExpr::with(core).then(epilog)),
            ]),
        }
    }
}

impl ExprLinter for AWaysToGo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = find_the_only_token_index_matching(toks, src, |t, s| t.get_ch(s).eq_str("ways"))
            .and_then(|idx| idx.checked_sub(2))
            .and_then(|i| toks.get(i))
            .map(|t| t.span)?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::InsertAfter([' ', 'a'].to_vec())],
            message: "This idiom requires the word `a`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the idiom `a ways to go` when the indefinite article is missing."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AWaysToGo;

    #[test]
    fn and_has() {
        assert_suggestion_result(
            "This project is currently under dev and has ways to go.",
            AWaysToGo::default(),
            "This project is currently under dev and has a ways to go.",
        );
    }

    #[test]
    fn and_has_comma() {
        assert_suggestion_result(
            "It's an ongoing effort and has ways to go, but please take a look and consider contributing as well.",
            AWaysToGo::default(),
            "It's an ongoing effort and has a ways to go, but please take a look and consider contributing as well.",
        );
    }

    #[test]
    fn have_in_order_to() {
        assert_suggestion_result(
            "We have ways to go in order to reach even close.",
            AWaysToGo::default(),
            "We have a ways to go in order to reach even close.",
        );
    }

    #[test]
    fn still_has() {
        assert_suggestion_result(
            "and our darray implementation still has ways to go.",
            AWaysToGo::default(),
            "and our darray implementation still has a ways to go.",
        );
    }

    #[test]
    fn still_has_before() {
        assert_suggestion_result(
            "that thing still has ways to go before it's actually usable",
            AWaysToGo::default(),
            "that thing still has a ways to go before it's actually usable",
        );
    }

    #[test]
    fn still_has_to_be() {
        assert_suggestion_result(
            "Currently react-native-keys still has ways to go to be free of vulnerabilities.",
            AWaysToGo::default(),
            "Currently react-native-keys still has a ways to go to be free of vulnerabilities.",
        );
    }

    #[test]
    fn still_have_to_with() {
        assert_suggestion_result(
            "the fact that we still have ways to go with updating the documentation",
            AWaysToGo::default(),
            "the fact that we still have a ways to go with updating the documentation",
        );
    }

    #[test]
    fn still_have_though() {
        assert_suggestion_result(
            "I still have ways to go though so it's a challenge for sure.",
            AWaysToGo::default(),
            "I still have a ways to go though so it's a challenge for sure.",
        );
    }

    #[test]
    fn still_have_before() {
        assert_suggestion_result(
            "but it seems I'll still have ways to go before getting a fully descriptive usable schema",
            AWaysToGo::default(),
            "but it seems I'll still have a ways to go before getting a fully descriptive usable schema",
        );
    }

    #[test]
    fn still_have_but() {
        assert_suggestion_result(
            "Now to work on the email part still have ways to go but I'm actually liking the new logic.",
            AWaysToGo::default(),
            "Now to work on the email part still have a ways to go but I'm actually liking the new logic.",
        );
    }

    // Avoid false positives

    #[test]
    fn dont_flag_ways_to_go_back() {
        assert_no_lints(
            "All IDEs (and even Vim/evil) have ways to go back to the last place''",
            AWaysToGo::default(),
        );
    }

    #[test]
    fn dont_flag_were_from_a_to_b() {
        assert_no_lints(
            "In every map there were ways to go from A to B by going through intermediate factories in <= time.",
            AWaysToGo::default(),
        );
    }

    #[test]
    fn dont_flag_to_go_about() {
        assert_no_lints(
            "Also I now have ways to go about rendering audio along with the frames",
            AWaysToGo::default(),
        );
    }
}
