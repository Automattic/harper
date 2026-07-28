use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct OnesOwnAccord {
    expr: SequenceExpr,
}

impl Default for OnesOwnAccord {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("on")
                .t_ws()
                .t_set(&[
                    // Correct possessive determiners
                    "my", "our", "your", "his", "her", "its", "their", "one's",
                    // Common mistakes
                    "it's", "ones", "there", "they're", "theyre",
                ])
                .t_ws()
                .t_aco("own")
                .t_ws()
                .t_set(&["accord", "accords"]),
        }
    }
}

impl ExprLinter for OnesOwnAccord {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                span.get_content(source),
            )],
            message: "The correct preposition is `of`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects incorrect usage of `on one's own accord` and suggests `of one's own accord`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::OnesOwnAccord;

    #[test]
    fn her() {
        assert_suggestion_result(
            "Monika going to bed on her own accord?",
            OnesOwnAccord::default(),
            "Monika going to bed of her own accord?",
        );
    }

    #[test]
    fn his() {
        assert_suggestion_result(
            "The shot went down without a thought and the man soon left on his own accord.",
            OnesOwnAccord::default(),
            "The shot went down without a thought and the man soon left of his own accord.",
        );
    }

    #[test]
    fn its() {
        assert_suggestion_result(
            "It appears typescript is on its own accord looking at every dependency in my package.json",
            OnesOwnAccord::default(),
            "It appears typescript is of its own accord looking at every dependency in my package.json",
        );
    }

    #[test]
    fn its_apostrophe() {
        assert_suggestion_result(
            "will return the memory to the operating system on it's own accord to prevent linked freelists",
            OnesOwnAccord::default(),
            "will return the memory to the operating system of it's own accord to prevent linked freelists",
        );
    }

    #[test]
    fn my() {
        assert_suggestion_result(
            "I would love to be able to re-order the icons on my own accord.",
            OnesOwnAccord::default(),
            "I would love to be able to re-order the icons of my own accord.",
        );
    }

    #[test]
    fn our() {
        assert_suggestion_result(
            "We did not include polyfill-php80 on our own accord, Google Ads package did.",
            OnesOwnAccord::default(),
            "We did not include polyfill-php80 of our own accord, Google Ads package did.",
        );
    }

    #[test]
    fn their() {
        assert_suggestion_result(
            "they can also post messages to rooms on their own accord",
            OnesOwnAccord::default(),
            "they can also post messages to rooms of their own accord",
        );
    }

    #[test]
    fn their_plural() {
        assert_suggestion_result(
            "Both frontend and backend can be launched on their own accords from the relevant Dockerfile's in each directory.",
            OnesOwnAccord::default(),
            "Both frontend and backend can be launched of their own accords from the relevant Dockerfile's in each directory.",
        );
    }

    #[test]
    fn there_spello() {
        assert_suggestion_result(
            "Let them fail on there own accord because that's where they're heading.",
            OnesOwnAccord::default(),
            "Let them fail of there own accord because that's where they're heading.",
        );
    }

    #[test]
    fn your() {
        assert_suggestion_result(
            "If you want to do this you are free to on your own accord.",
            OnesOwnAccord::default(),
            "If you want to do this you are free to of your own accord.",
        );
    }

    #[test]
    fn your_plural() {
        assert_suggestion_result(
            "Name things on your own accords.",
            OnesOwnAccord::default(),
            "Name things of your own accords.",
        );
    }
}
