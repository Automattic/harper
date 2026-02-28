use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct ThePointFor {
    expr: Box<dyn Expr>,
}

impl Default for ThePointFor {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_of(vec![Box::new(WordSet::new(&[
                    // "that's" leads to false positives: "that's the point for me"
                    "is", "was", "what's", "whats",
                ]))])
                .t_ws()
                .t_aco("the")
                .t_ws()
                .t_aco("point")
                .t_ws()
                .t_aco("for"),
            ),
        }
    }
}

impl ExprLinter for ThePointFor {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `the point for` to `the point of`"
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Avoid flagging things like "p0 is the point for which we want to find the closest point to the line"
        if let Some((_, after)) = ctx
            && after.len() >= 2
            && after[0].kind.is_whitespace()
            && after[1]
                .span
                .get_content(src)
                .eq_ignore_ascii_case_str("which")
        {
            return None;
        }

        let forspan = toks.last()?.span;
        Some(Lint {
            span: forspan,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                forspan.get_content(src),
            )],
            message: "Did you mean `the point of`?".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ThePointFor;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_is() {
        assert_suggestion_result(
            "What is the point for this check?",
            ThePointFor::default(),
            "What is the point of this check?",
        );
    }

    #[test]
    #[ignore = "'that' is disabled to avoid false positives until the heuristics can be improved"]
    fn fix_thats_the_point_no_apostrophe() {
        assert_suggestion_result(
            "You should also keep an eye on the issue list because thats the point for being public",
            ThePointFor::default(),
            "You should also keep an eye on the issue list because thats the point of being public",
        );
    }

    #[test]
    fn fix_was_the_point() {
        assert_suggestion_result(
            "But avoiding to learn qraphql to find out the proper query was the point for asking for this convenience command.",
            ThePointFor::default(),
            "But avoiding to learn qraphql to find out the proper query was the point of asking for this convenience command.",
        );
    }

    #[test]
    fn fix_whats_the_point() {
        assert_suggestion_result(
            "What's the point for IRepository?",
            ThePointFor::default(),
            "What's the point of IRepository?",
        );
    }

    #[test]
    fn fix_whats_the_point_no_apostrophe() {
        // Whats the point for using a reader like feedly, if the articles open in their native website
        assert_suggestion_result(
            "## I dont get RSS. Whats the point for using a reader like feedly, if the articles open in their native website",
            ThePointFor::default(),
            "## I dont get RSS. Whats the point of using a reader like feedly, if the articles open in their native website",
        );
    }

    #[test]
    fn avoid_flagging_the_point_for_which() {
        assert_no_lints(
            "p0 is the point for which we want to find the closest point to the line",
            ThePointFor::default(),
        );
    }
}
