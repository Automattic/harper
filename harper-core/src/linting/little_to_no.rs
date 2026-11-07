use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LittleToNo {
    expr: SequenceExpr,
}

impl Default for LittleToNo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("no")
                .t_ws_h()
                .t_aco("to")
                .t_ws_h()
                .t_aco("little"),
        }
    }
}

impl ExprLinter for LittleToNo {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() < 5 {
            return None;
        }

        let correction: Vec<char> =
            [4, 1, 2, 3, 0].iter().try_fold(Vec::new(), |mut acc, &i| {
                acc.extend_from_slice(toks.get(i)?.get_ch(src));
                Some(acc)
            })?;

        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                correction,
                span.get_content(src),
            )],
            message: "The phrase `little to no` is more idiomatic than `no to little`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Converts the clunky or unidiomatic phrase `no to little` into the standard English idiom `little to no`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::LittleToNo;

    #[test]
    fn hyphen_hyphen() {
        assert_suggestion_result(
            "parents who allow their 15 and 16 year olds to drive on the same streets with no-to-little training",
            LittleToNo::default(),
            "parents who allow their 15 and 16 year olds to drive on the same streets with little-to-no training",
        );
    }

    #[test]
    fn space_space() {
        assert_suggestion_result(
            "There was no to little industrialisation of India.",
            LittleToNo::default(),
            "There was little to no industrialisation of India.",
        );
    }

    #[test]
    #[ignore = "this is surely too rare and quirky to support"]
    fn hyphen_space_hyphen() {
        assert_suggestion_result(
            "The reflecter supports reflection utilities for node.js with no- to little impact on runtime performance.",
            LittleToNo::default(),
            "The reflecter supports reflection utilities for node.js with little- to no impact on runtime performance.",
        );
    }

    #[test]
    fn start_of_sentence() {
        assert_suggestion_result(
            "No to little increase in the number of metrics collected.",
            LittleToNo::default(),
            "Little to no increase in the number of metrics collected.",
        );
    }
}
