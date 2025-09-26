use crate::{
    Token,
    expr::{All, AnchorEnd, Expr, FirstMatchOf, LongestMatchOf, ReflexivePronoun, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct Addicting {
    expr: Box<dyn Expr>,
}

impl Default for Addicting {
    fn default() -> Self {
        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                // matches `addicting` without anything after
                Box::new(SequenceExpr::aco("addicting").then(AnchorEnd)),
                // matches `addicting` <ws> [ any word but not a reflexive pronoun or object pronoun ]
                Box::new(
                    SequenceExpr::aco("addicting")
                        .then_whitespace()
                        .then(All::new(vec![
                            // positive - any word
                            Box::new(SequenceExpr::any_word()),
                            // negative - reflexive pronoun or object pronoun
                            Box::new(SequenceExpr::unless(FirstMatchOf::new(vec![
                                Box::new(ReflexivePronoun::with_common_errors()),
                                Box::new(SequenceExpr::default().then_object_pronoun()),
                            ]))),
                        ])),
                ),
            ])),
        }
    }
}

impl ExprLinter for Addicting {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = toks.first()?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case(
                "addictive".chars().collect(),
                tok.span.get_content(src),
            )],
            message: "When used as an adjective, `addictive` is the traditional and more f form."
                .to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Replaces `addicting` with `addictive` when used as an adjective."
    }
}

#[cfg(test)]
mod tests {
    use super::Addicting;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_addicting() {
        assert_suggestion_result(
            "It is addicting like heroin.",
            Addicting::default(),
            "It is addictive like heroin.",
        );
    }

    #[test]
    fn dont_flag_addicting_object_pronoun() {
        assert_lint_count("It is addicting me.", Addicting::default(), 0);
    }

    #[test]
    fn dont_flag_addicting_reflexive_pronoun() {
        assert_lint_count("He is addicting himself.", Addicting::default(), 0);
    }

    #[test]
    fn fix_yet_highly_addicting() {
        assert_suggestion_result(
            "The objective of the game is simple yet highly addicting, you start out with the four basic elements.",
            Addicting::default(),
            "The objective of the game is simple yet highly addictive, you start out with the four basic elements.",
        );
    }

    #[test]
    fn dont_flag_addicting_them_on() {
        assert_no_lints(
            "Helping humans on their daily tasks instead of addicting them on social networks of all sorts.",
            Addicting::default(),
        );
    }

    #[test]
    #[ignore = "False positive since `myself` is not an object pronoun in this construction"]
    fn fix_find_things_addicting_myself() {
        assert_suggestion_result(
            "Yeah, I find taking the functional approach for these kinds of problems rather addicting myself :)",
            Addicting::default(),
            "Yeah, I find taking the functional approach for these kinds of problems rather addictive myself :)",
        );
    }

    #[test]
    fn dont_fix_coerced_into_addicting_themselves() {
        assert_no_lints(
            "The British, in another display of gunboat diplomacy, coerced countless innocent people into addicting themselves to opium.",
            Addicting::default(),
        );
    }
}
