use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::token::Token;
use crate::token_string_ext::TokenStringExt;

pub struct MoreBetter {
    expr: Box<dyn Expr>,
}

impl Default for MoreBetter {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::any_of(vec![
                Box::new(
                    SequenceExpr::default()
                        .t_aco("more")
                        .t_ws()
                        .then_comparative_adjective(),
                ),
                Box::new(
                    SequenceExpr::default()
                        .t_aco("most")
                        .t_ws()
                        .then_superlative_adjective(),
                ),
            ])),
        }
    }
}

impl ExprLinter for MoreBetter {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let phrase_span = toks.span()?;

        let degree_str = toks.first()?.span.get_content_string(src);
        let adj_span = toks.last()?.span;

        let suggestion = Suggestion::replace_with_match_case(
            adj_span.get_content(src).to_vec(),
            phrase_span.get_content(src),
        );

        let message = format!(
            "{} is already in the {} form, the {} is redundant",
            adj_span.get_content_string(src),
            if degree_str.eq_ignore_ascii_case("more") {
                "comparative"
            } else {
                "superlative"
            },
            degree_str,
        );

        Some(Lint {
            span: phrase_span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![suggestion],
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Finds redundant paring of `more` or `most` with adjectives already in the comparative or superlative form."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{MoreBetter, tests::assert_suggestion_result};

    #[test]
    fn flag_most_biggest() {
        assert_suggestion_result("Most biggest", MoreBetter::default(), "Biggest");
    }

    #[test]
    fn flag_more_better_and_more_better() {
        assert_suggestion_result(
            "More bigger is more better",
            MoreBetter::default(),
            "Bigger is better",
        );
    }
}
