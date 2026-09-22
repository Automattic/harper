use crate::{
    Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AsHow {
    expr: SequenceExpr,
}

impl Default for AsHow {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["as", "how"]),
        }
    }
}

impl ExprLinter for AsHow {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((before, _)) = ctx
            && let [left @ .., last_word, last_ws] = before
            && last_ws.kind.is_whitespace()
        {
            if last_word.get_ch(src).eq_str("same") {
                return None;
            }

            if let [.., prev_word, prev_ws] = left
                && last_word.kind.is_adjective()
                && prev_ws.kind.is_whitespace()
                && prev_word.get_ch(src).eq_ch(&['a', 's'])
            {
                return None;
            }
        }

        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "as to how",
                span.get_content(src),
            )],
            message: "Consider rephrasing to 'as to how'".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `as how` to `as to how`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AsHow;

    #[test]
    fn advise() {
        assert_suggestion_result(
            "[BUG] Advise as how to debug error messages.",
            AsHow::default(),
            "[BUG] Advise as to how to debug error messages.",
        );
    }

    #[test]
    fn puzzled() {
        assert_suggestion_result(
            "Since I'm using OpenAPI to describe my API I'm puzzled as how to describe this response.",
            AsHow::default(),
            "Since I'm using OpenAPI to describe my API I'm puzzled as to how to describe this response.",
        );
    }

    #[test]
    fn example() {
        assert_suggestion_result(
            "A sample/example as how to use Summaries",
            AsHow::default(),
            "A sample/example as to how to use Summaries",
        );
    }

    #[test]
    fn clueless() {
        assert_suggestion_result(
            "searched the issues/questions but I'm still clueless as how to set the body of the HTTP request",
            AsHow::default(),
            "searched the issues/questions but I'm still clueless as to how to set the body of the HTTP request",
        );
    }

    #[test]
    fn examples() {
        assert_suggestion_result(
            "We would need a more concrete specification together with examples as how this would be supposed to work.",
            AsHow::default(),
            "We would need a more concrete specification together with examples as to how this would be supposed to work.",
        );
    }

    #[test]
    fn confused() {
        assert_suggestion_result(
            "Hi! I'm confused as how to save the brain bytes model",
            AsHow::default(),
            "Hi! I'm confused as to how to save the brain bytes model",
        );
    }

    #[test]
    fn tackle() {
        assert_suggestion_result(
            "it's more of a design document/me laying my thoughts out as how to tackle this problem",
            AsHow::default(),
            "it's more of a design document/me laying my thoughts out as to how to tackle this problem",
        );
    }

    // Avoid false positives

    #[test]
    fn avoid_as_well_as_how() {
        assert_no_lints("When to do Git things (as well as how)", AsHow::default());
    }

    #[test]
    fn avoid_as_well_as_they() {
        assert_no_lints(
            "Some questions on configuring capsules, as well as how they are (or are not) weighted.",
            AsHow::default(),
        );
    }

    #[test]
    fn avoid_as_well_as_how_to() {
        assert_no_lints(
            "how to contribute a feature to PolyScope as well as how to modify and remove it",
            AsHow::default(),
        );
    }

    #[test]
    fn avoid_as_much_as() {
        assert_no_lints(
            "It recognizes that how code looks matters just as much as how it works.",
            AsHow::default(),
        );
    }

    #[test]
    fn avoid_as_far_as() {
        assert_no_lints(
            "Although the renderer will be opinionated as far as how it will structure the submission data out-of-the-box",
            AsHow::default(),
        );
    }

    #[test]
    fn avoid_the_same_as() {
        assert_no_lints(
            "FILE_ALL_ACCESS is not defined the same as how Windows defines it",
            AsHow::default(),
        );
    }

    // Edge cases

    #[test]
    #[ignore = "we probably can't handle this case no matter what"]
    fn hard_to_parse() {
        assert_no_lints(
            "As how many requests is a request with a list of coordinates counted?",
            AsHow::default(),
        );
    }
}
