use crate::{
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
    {CharStringExt, Lint, Token, TokenStringExt},
};

pub struct GoodAt {
    expr: Box<dyn Expr>,
}

impl Default for GoodAt {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .t_aco("good")
            .t_ws()
            .then_word_set(&["at", "in"])
            .t_ws()
            .then_any_word();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for GoodAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let prep_idx = 2;
        let prep_tok = &toks[prep_idx];
        let prep_span = prep_tok.span;
        let prep_chars = prep_span.get_content(src);

        let emoji = if prep_chars.eq_ignore_ascii_case_chars(&['i', 'n']) {
            "📥"
        } else {
            "🎯"
        };
        eprintln!("{emoji} {}", format_lint_match(toks, ctx, src));
        if emoji == "🎯" {
            return None;
        }

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                "at".chars().collect(),
                prep_span.get_content(src),
            )],
            message: "Use 'good at' to describe proficiency with a skill.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Checks for `good in` used instead of `good at` to describe proficiency with a skill."
    }
}

#[cfg(test)]
mod tests {
    use super::GoodAt;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn good_in() {
        assert_suggestion_result(
            "but we found that Claude is not always very good in being frugal ( Gemini seemed better at it ) .",
            GoodAt::default(),
            "but we found that Claude is not always very good at being frugal ( Gemini seemed better at it ) .",
        );
    }
}
