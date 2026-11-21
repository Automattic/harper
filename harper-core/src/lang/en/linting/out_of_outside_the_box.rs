use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct OutOfOutsideTheBox {
    expr: SequenceExpr,
}

impl Default for OutOfOutsideTheBox {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(
                    SequenceExpr::any_of([
                        Box::new(SequenceExpr::aco("out").t_ws().t_aco("side")),
                        Box::new(SequenceExpr::aco("outside")),
                    ])
                    .then_optional(SequenceExpr::whitespace().t_aco("of")),
                ),
                Box::new(SequenceExpr::aco("out").t_ws().t_aco("of")),
            ])
            .t_ws()
            .then_word_seq(&["the", "box"]),
        }
    }
}

impl ExprLinter for OutOfOutsideTheBox {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // 0:outside              2:the 4:box
        // 0:outside        2:of  4:the 6:box
        // 0:out     2:side       4:the 6:box
        // 0:out     2:side 4:of  6:the 8:box
        // 0:out            2:of  4:the 6:box
        if toks.get(2)?.get_ch(src).eq_ch(&['o', 'f'])
            && preceded_by_word(ctx, |t| {
                t.get_ch(src)
                    .eq_any_ignore_ascii_case_str(&["think", "thinking", "thinks", "thought"])
            })
        {
            let span = toks.get(0..=2)?.span()?;

            return Some(Lint {
                span,
                lint_kind: LintKind::Usage,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "outside",
                    span.get_content(src),
                )],
                message: "Use `outside the box` when referring to creative or original thinking."
                    .to_owned(),
                ..Default::default()
            });
        }

        if preceded_by_word(ctx, |t| {
            t.get_ch(src).eq_any_ignore_ascii_case_str(&[
                "right", "straight", "work", "worked", "working", "works",
            ])
        }) {
            let span = toks.get_rel_slice(0, -5)?.span()?;

            return Some(Lint {
                span,
                lint_kind: LintKind::Usage,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "out of",
                    span.get_content(src),
                )],
                message: "Use `out of the box` when referring to something that works immediately without setup or customization.".to_owned(),
                ..Default::default()
            });
        }
        None
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for incorrect mixing of the idioms `think outside the box` and `out of the box`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::OutOfOutsideTheBox;

    #[test]
    fn think_out_of_the_box() {
        assert_suggestion_result(
            "think out of the box",
            OutOfOutsideTheBox::default(),
            "think outside the box",
        );
    }

    #[test]
    fn right_outside_the_box() {
        assert_suggestion_result(
            "it works right outside the box",
            OutOfOutsideTheBox::default(),
            "it works right out of the box",
        );
    }

    #[test]
    fn right_out_side_the_box() {
        assert_suggestion_result(
            "it works right out side the box",
            OutOfOutsideTheBox::default(),
            "it works right out of the box",
        );
    }

    #[test]
    fn right_outside_of_the_box() {
        assert_suggestion_result(
            "it works right outside of the box",
            OutOfOutsideTheBox::default(),
            "it works right out of the box",
        );
    }

    #[test]
    fn right_out_side_of_the_box() {
        assert_suggestion_result(
            "it works right out side of the box",
            OutOfOutsideTheBox::default(),
            "it works right out of the box",
        );
    }
}
