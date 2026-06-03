use harper_brill::UPOS;

use crate::Token;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::InflectionOfBe;
use crate::patterns::UPOSSet;

use super::expr_linter::Chunk;
use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct FindFine {
    expr: SequenceExpr,
}

impl Default for FindFine {
    fn default() -> Self {
        let expr = SequenceExpr::with(InflectionOfBe::default())
            .t_ws()
            .t_aco("find")
            // Don't flag when `find` is followed by something that could be its
            // object, e.g. "what you want to do is find someone to replace you".
            .then_unless(SequenceExpr::whitespace().then(UPOSSet::new(&[
                UPOS::NOUN,
                UPOS::PROPN,
                UPOS::PRON,
                UPOS::DET,
            ])));

        Self { expr }
    }
}

impl ExprLinter for FindFine {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_word = matched_tokens.get(2)?;

        Some(Lint {
            span: offending_word.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "fine",
                offending_word.get_ch(source),
            )],
            message: "Did you mean `fine`?".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Fixes the common typo where writers write `find` when they mean `fine`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::FindFine;

    #[test]
    fn issue_2115() {
        assert_suggestion_result(
            "I was using oil.nvim from an year and everything was find for me but I was missing a very key feature",
            FindFine::default(),
            "I was using oil.nvim from an year and everything was fine for me but I was missing a very key feature",
        );
        assert_suggestion_result(
            "I made several observations throughout the evening and everything was find.",
            FindFine::default(),
            "I made several observations throughout the evening and everything was fine.",
        );
        assert_suggestion_result(
            "I am find not using GPU at all for open3d.",
            FindFine::default(),
            "I am fine not using GPU at all for open3d.",
        );
    }

    #[test]
    fn dont_flag_is_find_someone_3255() {
        assert_lint_count(
            "Generally speaking what you want to try and do is find someone to replace you before you leave.",
            FindFine::default(),
            0,
        );
    }
}
