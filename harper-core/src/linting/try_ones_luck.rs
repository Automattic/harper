use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TryOnesLuck {
    expr: SequenceExpr,
}

impl Default for TryOnesLuck {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["try", "tried", "tries", "trying"])
                .t_ws()
                .t_aco("out")
                .t_ws()
                .then_possessive_determiner()
                .t_ws()
                .t_aco("luck"),
        }
    }
}

impl ExprLinter for TryOnesLuck {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `try out one’s luck` to `try one’s luck`"
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let ws_out_span = toks[1..3].span()?;
        Some(Lint {
            lint_kind: LintKind::Usage,
            span: ws_out_span,
            message: "`Try out` is a different idiom than `try one's luck`".to_string(),
            suggestions: vec![Suggestion::Remove],
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TryOnesLuck;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_tried_his() {
        assert_suggestion_result(
            "Ndabayithethwa Ndlondlo tried out his luck from distance.",
            TryOnesLuck::default(),
            "Ndabayithethwa Ndlondlo tried his luck from distance.",
        );
    }

    #[test]
    fn fix_tried_my() {
        assert_suggestion_result(
            "Tried out my luck on making other people OCs if I made them",
            TryOnesLuck::default(),
            "Tried my luck on making other people OCs if I made them",
        );
    }

    #[test]
    fn fix_tried_their() {
        assert_suggestion_result(
            "However a few homelab'ers on the ServeTheHome Forum just tried out their luck to figure out how to utilize these boards",
            TryOnesLuck::default(),
            "However a few homelab'ers on the ServeTheHome Forum just tried their luck to figure out how to utilize these boards",
        );
    }

    #[test]
    fn fix_tries_out_his() {
        assert_suggestion_result(
            "Inggo Tries Out His Luck in Politics",
            TryOnesLuck::default(),
            "Inggo Tries His Luck in Politics",
        );
    }

    #[test]
    fn fix_try_out_her() {
        assert_suggestion_result(
            "this woman had deigned to try out her luck at my place",
            TryOnesLuck::default(),
            "this woman had deigned to try her luck at my place",
        );
    }

    #[test]
    fn fix_try_out_his() {
        assert_suggestion_result(
            "Turned out to be a young American kid who decided to leave Washington DC and come try out his luck in Shanghai.",
            TryOnesLuck::default(),
            "Turned out to be a young American kid who decided to leave Washington DC and come try his luck in Shanghai.",
        );
    }

    #[test]
    fn fix_try_out_my() {
        assert_suggestion_result(
            "So I am just gonna try out my luck here with a problem that I have been facing with my prod ETL project.",
            TryOnesLuck::default(),
            "So I am just gonna try my luck here with a problem that I have been facing with my prod ETL project.",
        );
    }

    #[test]
    fn fix_try_out_our() {
        assert_suggestion_result(
            "we decided to try out our luck and challenge the mighty Himalayas",
            TryOnesLuck::default(),
            "we decided to try our luck and challenge the mighty Himalayas",
        );
    }

    #[test]
    fn fix_try_out_their() {
        assert_suggestion_result(
            "Users will come over to try out their luck, see what is success rate",
            TryOnesLuck::default(),
            "Users will come over to try their luck, see what is success rate",
        );
    }

    #[test]
    fn fix_try_out_your() {
        assert_suggestion_result(
            "Try out your luck.",
            TryOnesLuck::default(),
            "Try your luck.",
        );
    }

    #[test]
    fn fix_trying_her() {
        assert_suggestion_result(
            "A story about a girl in Chemistry trying out her luck and commitment towards this discipline in an attempt to get a Ph.D.",
            TryOnesLuck::default(),
            "A story about a girl in Chemistry trying her luck and commitment towards this discipline in an attempt to get a Ph.D.",
        );
    }

    #[test]
    fn fix_trying_his() {
        assert_suggestion_result(
            "A simple man trying out his luck in this probabilistic world.",
            TryOnesLuck::default(),
            "A simple man trying his luck in this probabilistic world.",
        );
    }

    #[test]
    fn fix_trying_my() {
        assert_suggestion_result(
            "No need to be sorry hahaha, I was just trying out my luck.",
            TryOnesLuck::default(),
            "No need to be sorry hahaha, I was just trying my luck.",
        );
    }

    #[test]
    fn fix_trying_our() {
        assert_suggestion_result(
            "It's now late 2016, almost two years of trying out our luck in the business world with Mike, I called it quits.",
            TryOnesLuck::default(),
            "It's now late 2016, almost two years of trying our luck in the business world with Mike, I called it quits.",
        );
    }

    #[test]
    fn fix_trying_their() {
        assert_suggestion_result(
            "It was a time before millions began trying out their luck, a time when just applying guaranteed you would get a winning entry.",
            TryOnesLuck::default(),
            "It was a time before millions began trying their luck, a time when just applying guaranteed you would get a winning entry.",
        );
    }

    #[test]
    fn fix_trying_out_your() {
        assert_suggestion_result(
            "You should start by trying out your luck and come back with specific questions.",
            TryOnesLuck::default(),
            "You should start by trying your luck and come back with specific questions.",
        );
    }
}
