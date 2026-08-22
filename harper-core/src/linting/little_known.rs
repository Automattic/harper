/// Linter for detecting and fixing mixing up "it's little known" and "it's a little known"
use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LittleKnown {
    expr: SequenceExpr,
}

impl Default for LittleKnown {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["it's", "its"])
                .t_ws()
                .then_optional(SequenceExpr::aco("a").t_ws())
                .t_aco("little")
                .t_ws_h()
                .t_aco("known")
                .t_ws()
                .t_set(&["fact", "that"]),
        }
    }
}

impl ExprLinter for LittleKnown {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (span, lint_kind, sugg, action) = match (
            toks.len(),
            toks.last()?.get_ch(src).first()?.to_ascii_lowercase(),
        ) {
            (7, 'f') => (
                toks[0].span,
                LintKind::MissingWord,
                Suggestion::InsertAfter(vec![' ', 'a']),
                "Insert",
            ),
            (9, 't') => (
                toks[1..=2].span()?,
                LintKind::Usage,
                Suggestion::Remove,
                "Remove",
            ),
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind,
            suggestions: vec![sugg],
            message: format!("{} the indefinnite article `a`.", action),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::LittleKnown;

    #[test]
    fn fix_fact_needs_article_with_space() {
        assert_suggestion_result(
            "It's little known fact that Legolas did have a dialogue in the Frodo waking up scene",
            LittleKnown::default(),
            "It's a little known fact that Legolas did have a dialogue in the Frodo waking up scene",
        );
    }

    #[test]
    fn fix_fact_needs_article_with_hyphen() {
        assert_suggestion_result(
            "It's little-known fact among family and close friends, but it doesn't really come up.",
            LittleKnown::default(),
            "It's a little-known fact among family and close friends, but it doesn't really come up.",
        );
    }

    #[test]
    fn fix_that_shouldnt_have_article_hyphen() {
        assert_suggestion_result(
            "In the Macedonian and in the Turkish scientific public, it's a little-known that Mustafa Kemal visited Shtip in july 1909.",
            LittleKnown::default(),
            "In the Macedonian and in the Turkish scientific public, it's little-known that Mustafa Kemal visited Shtip in july 1909.",
        );
    }

    #[test]
    fn fix_that_shouldnt_have_article_space() {
        assert_suggestion_result(
            "And so it's a little known that the first post-war chancellor of Germany, Konrad Adenauer, who took charge of the democratic rebirth of Germany ...",
            LittleKnown::default(),
            "And so it's little known that the first post-war chancellor of Germany, Konrad Adenauer, who took charge of the democratic rebirth of Germany ...",
        );
    }
}
