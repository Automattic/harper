use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{All, AnchorEnd, Expr, FirstMatchOf, ReflexivePronoun, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct Addicting {
    expr: SequenceExpr,
}

impl Default for Addicting {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("addicting").then_longest_of(vec![
                // matches `addicting` without anything after
                Box::new(AnchorEnd),
                // Semicolon is not handled like comma is for `Chunk` - TODO: remove when #3405 is fixed
                Box::new(SequenceExpr::default().then_semicolon()),
                // matches `addicting` <ws> [ any word but not a reflexive pronoun or object pronoun ]
                Box::new(SequenceExpr::whitespace().then(All::new(vec![
                    // positive - any word
                    Box::new(SequenceExpr::any_word()),
                    // negative - reflexive pronoun or object pronoun
                    Box::new(SequenceExpr::unless(FirstMatchOf::new(vec![
                        Box::new(ReflexivePronoun::with_common_errors()),
                        Box::new(SequenceExpr::default().then_object_pronoun()),
                    ]))),
                ]))),
            ]),
        }
    }
}

impl ExprLinter for Addicting {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = toks.first()?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case(
                "addictive".chars().collect(),
                tok.get_ch(src),
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

    #[test]
    fn fix_at_end() {
        assert_suggestion_result("It is addicting", Addicting::default(), "It is addictive");
    }

    #[test]
    fn fix_before_comma() {
        assert_suggestion_result(
            "An addicting, side-scrolling, bouncing platform game - mzmousa/fruit-catcher.",
            Addicting::default(),
            "An addictive, side-scrolling, bouncing platform game - mzmousa/fruit-catcher.",
        );
    }

    #[test]
    fn fix_at_end_with_bang() {
        assert_suggestion_result(
            "Name of the Game is \"Pappu Pakia\". It is super fun and addicting!",
            Addicting::default(),
            "Name of the Game is \"Pappu Pakia\". It is super fun and addictive!",
        );
    }

    #[test]
    fn fix_at_end_with_period() {
        assert_suggestion_result(
            "This game is kind of addicting. The fast pace and small initial tail size puts emphasis on control and reaction time.",
            Addicting::default(),
            "This game is kind of addictive. The fast pace and small initial tail size puts emphasis on control and reaction time.",
        );
    }

    #[test]
    fn fix_at_end_with_semicolon() {
        assert_suggestion_result(
            "Powerful emotionally, Extremely addicting; Best game played in a long time, lost hours of time playing game without realizing",
            Addicting::default(),
            "Powerful emotionally, Extremely addictive; Best game played in a long time, lost hours of time playing game without realizing",
        );
    }

    #[test]
    fn fix_at_end_with_semicolon_no_commas_due_to_chunk_mode() {
        assert_suggestion_result(
            "Extremely addicting; Best game played in a long time",
            Addicting::default(),
            "Extremely addictive; Best game played in a long time",
        );
    }
}
