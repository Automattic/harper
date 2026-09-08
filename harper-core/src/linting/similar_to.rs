use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
    patterns::Word,
};

pub struct SimilarTo {
    expr: SequenceExpr,
}

impl Default for SimilarTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("similar").t_ws().then_any_of([
                Box::new(Word::new("with")) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::optional(SequenceExpr::default().then_noun().t_ws())
                        .t_aco("than"),
                ),
            ]),
        }
    }
}

impl ExprLinter for SimilarTo {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                span.get_content(src),
            )],
            message: "The correct preposition to use with `similar` is `to`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `similar than` and `similar with` to `similar to`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::SimilarTo;

    #[test]
    fn fix_similar_function_than() {
        assert_suggestion_result(
            "Request: Patreon skip duplicate files && implement a similar function than \"chapter-range\"",
            SimilarTo::default(),
            "Request: Patreon skip duplicate files && implement a similar function to \"chapter-range\"",
        );
    }

    #[test]
    fn fix_similar_with_goland() {
        assert_suggestion_result(
            "Is there a plan to develop a stand-alone IDE for Rust? Similar with goland, webstorm.",
            SimilarTo::default(),
            "Is there a plan to develop a stand-alone IDE for Rust? Similar to goland, webstorm.",
        )
    }

    #[test]
    fn fix_api_similar_with_model_analyzer() {
        assert_suggestion_result(
            "Is there a API similar with model_analyzer in pycolmap?",
            SimilarTo::default(),
            "Is there a API similar with model_analyzer in pycolmap?",
        )
    }

    // Edge cases, would-be false positives

    #[test]
    fn two_similar_tables_with_similar_access() {
        assert_no_lints(
            "I have two similar tables with similar access, both I can read by CURL",
            SimilarTo::default(),
        )
    }

    #[test]
    fn similar_code_with_deleted_lines() {
        assert_no_lints(
            "This function searches similar code with deleted lines.",
            SimilarTo::default(),
        )
    }

    #[test]
    fn seeing_similar_problems_with_both_node_x_and_node_y() {
        assert_no_lints(
            "I am seeing similar problems with both node 0.10.29 and 0.10.30.",
            SimilarTo::default(),
        )
    }

    #[test]
    fn extend_credentials_and_similar_with_custom_msgs() {
        assert_no_lints(
            "Extend CustomCredentials and similar with custom messages",
            SimilarTo::default(),
        );
    }

    #[test]
    fn identifying_similar_images_with_tensorflow() {
        assert_no_lints(
            "tutorial identifying similar images with tensorflow",
            SimilarTo::default(),
        )
    }

    #[test]
    fn find_similar_issues_with_ai() {
        assert_no_lints("Find Similar Issues with AI", SimilarTo::default())
    }

    #[test]
    fn similar_rather_than_identical() {
        assert_no_lints(
            "As a result, she wanted only similar rather than identical symbols.",
            SimilarTo::default(),
        )
    }
}
