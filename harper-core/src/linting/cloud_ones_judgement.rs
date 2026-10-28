use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct CloudOnesJudgement {
    expr: SequenceExpr,
}

impl Default for CloudOnesJudgement {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["clown", "clowned", "clowning", "clowns"])
                .t_ws()
                .then_possessive_determiner()
                .t_ws()
                .t_set(["judgement", "judgment", "judgements", "judgments"]),
        }
    }
}

impl ExprLinter for CloudOnesJudgement {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let span = toks.first()?.span;

        let mut corr = span.get_content(src).to_vec();
        corr[3] = ((corr[3] as u8) - (b'w' - b'u')) as char;
        corr[4] = ((corr[4] as u8) - (b'n' - b'd')) as char;

        Some(Lint {
            span,
            lint_kind: LintKind::Malapropism,
            suggestions: vec![Suggestion::ReplaceWith(corr)],
            message: "Unless this is intentional wordplay, the correct verb is `cloud`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the malapropism `clown one's judgement` to `cloud one's judgement`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::CloudOnesJudgement;

    #[test]
    fn clowned_your_judgement() {
        assert_suggestion_result(
            "I think the dark side of the force has clowned your judgement young Padawan.",
            CloudOnesJudgement::default(),
            "I think the dark side of the force has clouded your judgement young Padawan.",
        );
    }

    #[test]
    fn clown_his_judgement() {
        assert_suggestion_result(
            "...because he knows that the sarcophagus will only clown his judgement and he can actually get ahead...",
            CloudOnesJudgement::default(),
            "...because he knows that the sarcophagus will only cloud his judgement and he can actually get ahead...",
        );
    }

    #[test]
    fn clown_your_judgement() {
        assert_suggestion_result(
            "stop letting infatuation clown your judgement.",
            CloudOnesJudgement::default(),
            "stop letting infatuation cloud your judgement.",
        );
    }

    #[test]
    fn cloning_my_judgement() {
        assert_suggestion_result(
            "...cloning my judgement knew the genius would rise...",
            CloudOnesJudgement::default(),
            "...cloning my judgement knew the genius would rise...",
        );
    }

    #[test]
    fn clown_your_judgements() {
        assert_suggestion_result(
            "You Should Never Allow Your Emotions Clown Your Judgements Over Somebody",
            CloudOnesJudgement::default(),
            "You Should Never Allow Your Emotions Cloud Your Judgements Over Somebody",
        );
    }

    #[test]
    fn clown_our_judgment() {
        assert_suggestion_result(
            "know-it-all attitude is what can clown our judgment when we become more experienced",
            CloudOnesJudgement::default(),
            "know-it-all attitude is what can cloud our judgment when we become more experienced",
        );
    }

    #[test]
    fn clowns_his_judgment() {
        assert_suggestion_result(
            "Yoda accuses him of hatred and clowns his judgment, questioning why he came",
            CloudOnesJudgement::default(),
            "Yoda accuses him of hatred and clouds his judgment, questioning why he came",
        );
    }

    #[test]
    fn clowning_my_judgment() {
        assert_suggestion_result(
            "trying to make a public POTD was somehow clowning my judgment",
            CloudOnesJudgement::default(),
            "trying to make a public POTD was somehow clouding my judgment",
        );
    }

    #[test]
    fn dont_flag_coincidence() {
        assert_no_lints(
            "Same old from this clown. His judgment of talent the last few years is less than sub par.",
            CloudOnesJudgement::default(),
        );
    }
}
