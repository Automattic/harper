use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct ByTheBook {
    expr: SequenceExpr,
}

impl Default for ByTheBook {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                "all",
                "done",
                "everything",
                "go",
                "it",
                "play",
                "playing",
            ])
            .t_ws()
            .t_aco("by")
            .t_ws()
            .t_aco("the")
            .t_ws()
            .t_aco("books")
            .then_any_of(vec![
                Box::new(AnchorEnd),
                Box::new(SequenceExpr::whitespace().then_conjunction()),
            ]),
        }
    }
}

impl ExprLinter for ByTheBook {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = find_the_only_token_matching(matched_tokens, source, |t, s| {
            t.get_ch(s).eq_ch(&['b', 'o', 'o', 'k', 's'])
        })?
        .span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "book",
                span.get_content(source),
            )],
            message: "Did you mean the idiom `by the book`?".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `by the books` to `by the book`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ByTheBook;

    #[test]
    fn fix_play_it_by_the_books() {
        assert_suggestion_result(
            "As a result, we can conclude that organizations do \"play it by the books\" but individually define what is inside their playbooks ...",
            ByTheBook::default(),
            "As a result, we can conclude that organizations do \"play it by the book\" but individually define what is inside their playbooks ...",
        );
    }

    #[test]
    fn fix_everything_by_the_books() {
        assert_suggestion_result(
            "The finalizer used in Replace AsyncProcess exit handler by weakref.finalize #4184 is doing everything by the books.",
            ByTheBook::default(),
            "The finalizer used in Replace AsyncProcess exit handler by weakref.finalize #4184 is doing everything by the book.",
        );
    }

    #[test]
    fn dont_flag_when_not_at_end() {
        assert_no_lints(
            "The order of books within the breadcrumb dropdowns (i.e. when you click the arrow) is done by the books' id, rather than by name",
            ByTheBook::default(),
        );
    }

    #[test]
    fn fix_play_by_the_books() {
        assert_suggestion_result(
            "We essentially play by the books and let the actual Pod with identity make the requests.",
            ByTheBook::default(),
            "We essentially play by the book and let the actual Pod with identity make the requests.",
        );
    }

    #[test]
    fn fix_playing_by_the_books() {
        assert_suggestion_result(
            "If we're playing by the books, we should also implement something that tells a user when they're trying to make a table.",
            ByTheBook::default(),
            "If we're playing by the book, we should also implement something that tells a user when they're trying to make a table.",
        );
    }

    #[test]
    fn fix_go_by_the_books() {
        // NOTE: This one might be a false positive meaning following examples in reference materials literally?
        assert_suggestion_result(
            "If you wanted to go by the books, I would agree that removing such consistently unmethylated positions is a good option.",
            ByTheBook::default(),
            "If you wanted to go by the book, I would agree that removing such consistently unmethylated positions is a good option.",
        );
    }

    #[test]
    fn fix_configured_by_the_books() {
        assert_suggestion_result(
            "a new media folder was created under config/addons_config/paperless_ng even though I configured it by the books",
            ByTheBook::default(),
            "a new media folder was created under config/addons_config/paperless_ng even though I configured it by the book",
        );
    }

    #[test]
    fn fix_all_by_the_books() {
        assert_suggestion_result(
            "we did all by the books and we did not need the options at the end",
            ByTheBook::default(),
            "we did all by the book and we did not need the options at the end",
        );
    }
}
