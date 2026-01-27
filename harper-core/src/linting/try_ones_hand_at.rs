use crate::token_string_ext::TokenStringExt;
use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TryOnesHandAt {
    expr: Box<dyn Expr>,
}

impl Default for TryOnesHandAt {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["try", "tried", "tries", "trying"])
                    .t_ws()
                    .then_possessive_determiner()
                    .t_ws()
                    .t_aco("hands")
                    .t_ws()
                    .t_aco("at"),
            ),
        }
    }
}

impl ExprLinter for TryOnesHandAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        eprintln!("{}", toks.span()?.get_content_string(src));
        let hands_idx = 4;
        let hands_tok = toks.get(hands_idx)?;
        let hands_span = hands_tok.span;
        let hands_chars = hands_span.get_content(src);

        Some(Lint {
            span: hands_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                vec!['h', 'a', 'n', 'd'],
                hands_chars,
            )],
            message: "This idiom uses the singular `hand`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `try one's hands at` to `try one's hand at`."
    }
}

#[cfg(test)]
mod tests {
    use super::TryOnesHandAt;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_tried_my() {
        assert_suggestion_result(
            "I tried my hands at a little test to see how different parameters I get for the same deck and the same material.",
            TryOnesHandAt::default(),
            "I tried my hand at a little test to see how different parameters I get for the same deck and the same material.",
        )
    }

    #[test]
    fn fix_tried_their() {
        assert_suggestion_result(
            "If there isn't any obvious reason why no one has tried their hands at it yet, I might try implementing it.",
            TryOnesHandAt::default(),
            "If there isn't any obvious reason why no one has tried their hand at it yet, I might try implementing it.",
        )
    }

    #[test]
    fn fix_tries_his() {
        assert_suggestion_result(
            "A fellow programmer from India who tries his hands at everything he can.",
            TryOnesHandAt::default(),
            "A fellow programmer from India who tries his hand at everything he can.",
        )
    }

    #[test]
    fn fix_try_my() {
        assert_suggestion_result(
            "I am happy to try my hands at implementing one, but not being that proficient in C/C++ need some guidance on where to start.",
            TryOnesHandAt::default(),
            "I am happy to try my hand at implementing one, but not being that proficient in C/C++ need some guidance on where to start.",
        )
    }

    #[test]
    fn fix_try_our() {
        assert_suggestion_result(
            "One way to make some of the requirements for this more concrete is to try our hands at implementing a language server.",
            TryOnesHandAt::default(),
            "One way to make some of the requirements for this more concrete is to try our hand at implementing a language server.",
        )
    }

    #[test]
    fn fix_try_their() {
        assert_suggestion_result(
            "At the end the user will be able to create a list of decimal numbers to try their hands at the Diagonal Argument on their own.",
            TryOnesHandAt::default(),
            "At the end the user will be able to create a list of decimal numbers to try their hand at the Diagonal Argument on their own.",
        )
    }

    #[test]
    fn fix_try_your() {
        assert_suggestion_result(
            "You'll likely need to try your hands at a bit of Lua to make it work.",
            TryOnesHandAt::default(),
            "You'll likely need to try your hand at a bit of Lua to make it work.",
        )
    }

    #[test]
    fn fix_trying_my() {
        assert_suggestion_result(
            "I wouldn't mind trying my hands at a PR if the solution would be accepted.",
            TryOnesHandAt::default(),
            "I wouldn't mind trying my hand at a PR if the solution would be accepted.",
        )
    }
}
