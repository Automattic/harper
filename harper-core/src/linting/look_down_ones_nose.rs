use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LookDownOnesNose {
    expr: Box<dyn Expr>,
}

impl Default for LookDownOnesNose {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["look", "looked", "looking", "looks"])
                    .t_ws()
                    .then_possessive_determiner()
                    .t_ws()
                    .then_word_set(&["nose", "noses"])
                    .t_ws()
                    .t_aco("down"),
            ),
        }
    }
}

impl ExprLinter for LookDownOnesNose {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `look one's nose down` to `look down one's nose`"
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (looktok, prontok, nosetok) = (toks.first()?, toks.get(2)?, toks.get(4)?);
        let (lookspan, pronspan, nosespan) = (looktok.span, prontok.span, nosetok.span);
        let (lookstr, pronstr, nosestr) = (
            lookspan.get_content_string(src),
            pronspan.get_content_string(src),
            nosespan.get_content_string(src),
        );

        Some(Lint {
            lint_kind: LintKind::Usage,
            span: toks.span()?,
            suggestions: vec![Suggestion::replace_with_match_case(
                format!("{lookstr} down {pronstr} {nosestr}")
                    .chars()
                    .collect(),
                toks.span()?.get_content(src),
            )],
            message: "The correct idiom is `look down one's nose`.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::LookDownOnesNose;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn look_his_nose() {
        assert_suggestion_result(
            "He seemed to look his nose down upon everything",
            LookDownOnesNose::default(),
            "He seemed to look down his nose upon everything",
        );
    }

    #[test]
    fn look_my_nose() {
        assert_suggestion_result(
            "I'm the last one to look my nose down at what people do from a moral perspective.",
            LookDownOnesNose::default(),
            "I'm the last one to look down my nose at what people do from a moral perspective.",
        );
    }

    #[test]
    fn look_their_nose() {
        assert_suggestion_result(
            "I hate how humans look their nose down on certain animals",
            LookDownOnesNose::default(),
            "I hate how humans look down their nose on certain animals",
        );
    }

    #[test]
    fn look_their_noses() {
        assert_suggestion_result(
            "Yet many look their noses down at the right as if it were a used Kleenex.",
            LookDownOnesNose::default(),
            "Yet many look down their noses at the right as if it were a used Kleenex.",
        );
    }

    #[test]
    fn look_your_nose() {
        assert_suggestion_result(
            "You look your nose down on me like I am only a breathing object",
            LookDownOnesNose::default(),
            "You look down your nose on me like I am only a breathing object",
        );
    }

    #[test]
    fn looking_her_nose() {
        assert_suggestion_result(
            "The long nose definitely helps give the feel of her looking her nose down on others",
            LookDownOnesNose::default(),
            "The long nose definitely helps give the feel of her looking down her nose on others",
        );
    }

    #[test]
    fn looking_his_nose() {
        assert_suggestion_result(
            "literally looking his nose down at the commoners",
            LookDownOnesNose::default(),
            "literally looking down his nose at the commoners",
        );
    }

    #[test]
    fn looking_their_noses() {
        assert_suggestion_result(
            "It makes no sense that the princesses are looking their noses down on her",
            LookDownOnesNose::default(),
            "It makes no sense that the princesses are looking down their noses on her",
        );
    }

    #[test]
    fn looking_your_nose() {
        assert_suggestion_result(
            "Whatever you do, don’t fall into the trap of looking your nose down at the customer.",
            LookDownOnesNose::default(),
            "Whatever you do, don’t fall into the trap of looking down your nose at the customer.",
        );
    }

    #[test]
    fn looks_her_nose() {
        assert_suggestion_result(
            "Daenerys looks her nose down at them.",
            LookDownOnesNose::default(),
            "Daenerys looks down her nose at them.",
        );
    }

    #[test]
    fn looks_his_nose() {
        assert_suggestion_result(
            "He probably looks his nose down on Wraith, who don't eat inbred humans.",
            LookDownOnesNose::default(),
            "He probably looks down his nose on Wraith, who don't eat inbred humans.",
        );
    }

    #[test]
    fn looks_their_nose() {
        assert_suggestion_result(
            "Your friend looks their nose down at them and wants to unfriend them based on how little money they have.",
            LookDownOnesNose::default(),
            "Your friend looks down their nose at them and wants to unfriend them based on how little money they have.",
        );
    }
}
