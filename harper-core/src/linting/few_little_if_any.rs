use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct FewLittleIfAny {
    expr: SequenceExpr,
}

impl Default for FewLittleIfAny {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["few", "little"]).then_any_of([
                Box::new(
                    SequenceExpr::default()
                        .then_optional_comma()
                        .t_ws()
                        .then_word_seq(&["if", "any"])
                        .t_ws()
                        .then_noun(),
                ),
                Box::new(
                    SequenceExpr::whitespace()
                        .then(|t: &Token, _| t.kind.is_open_round())
                        .then_word_seq(&["if", "any"])
                        .then(|t: &Token, _| t.kind.is_close_round())
                        .t_ws()
                        .then_noun(),
                ),
            ]),
        }
    }
}

impl ExprLinter for FewLittleIfAny {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let (fl, noun) = (
            matched_tokens.first()?.clone(),
            matched_tokens.last()?.clone(),
        );
        let (fl, noun) = (fl.get_ch(source), noun.get_ch(source));
        enum FL {
            Few,
            Little,
        }
        let fl = if fl.eq_ch(&['f', 'e', 'w']) {
            FL::Few
        } else if fl.eq_ch(&['l', 'i', 't', 't', 'l', 'e']) {
            FL::Little
        } else {
            return None;
        };
        if fl == FL::Few {
            if noun.kind.is_plural_noun() {
                return None;
            }
        }
        if fl == FL::Little {
            if !noun.kind.is_mass_noun() {
                return None;
            }
        }
        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "correction",
            span.get_content(source),
        )];
        let message = "Fix this erorr".to_owned();
        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
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

    use super::FewLittleIfAny;

    #[test]
    fn test_skeleton() {
        assert_suggestion_result("erorr", FewLittleIfAny::default(), "correction");
    }
}
