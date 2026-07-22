use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FishNorFowl {
    expr: SequenceExpr,
}

impl Default for FishNorFowl {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["neither", "niether"])
                .t_ws()
                .then_word_seq(&["fish", "nor"])
                .t_ws()
                .t_set(&["foul", "bird"]),
        }
    }
}

impl ExprLinter for FishNorFowl {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let last_token = toks.last()?;

        let (lint_kind, message) = if last_token
            .get_ch(src)
            .starts_with_any_ignore_ascii_case_str(&["f"])
        {
            (
                LintKind::Spelling,
                "The correct spelling is `fowl`, meaning `bird`. `Foul` is an adjective meaning `dirty` or `offensive`.",
            )
        } else {
            (
                LintKind::Nonstandard,
                "The standard idiom is `neither fish nor fowl`.",
            )
        };

        let span = last_token.span;

        Some(Lint {
            span,
            lint_kind,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "fowl",
                span.get_content(src),
            )],
            message: message.to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `neither fish nor foul` and `neither fish nor bird` to `neither fish nor fowl`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::FishNorFowl;

    #[test]
    fn fix_bird() {
        assert_suggestion_result(
            "But there's one bus in Australia that actually tries to be that little bit train and is neither fish nor bird doing so.",
            FishNorFowl::default(),
            "But there's one bus in Australia that actually tries to be that little bit train and is neither fish nor fowl doing so.",
        );
    }

    #[test]
    fn fix_foul() {
        assert_suggestion_result(
            "But I do doubt chromebooks will be big hit, they're neither fish nor foul, since tablets stole netbooks' market.",
            FishNorFowl::default(),
            "But I do doubt chromebooks will be big hit, they're neither fish nor fowl, since tablets stole netbooks' market.",
        );
    }
}
