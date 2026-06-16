use crate::{
    Lint, Token, TokenStringExt,
    expr::{AnchorStart, Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LongTimeAgo {
    expr: SequenceExpr,
}

impl Default for LongTimeAgo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of(vec![
                Box::new(AnchorStart),
                Box::new(SequenceExpr::default().then_word_except(&["a"]).t_ws()),
            ])
            .t_aco("long")
            .t_ws()
            .t_aco("time")
            .t_ws()
            .t_aco("ago"),
        }
    }
}

impl ExprLinter for LongTimeAgo {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = match matched_tokens.len() {
            5 => matched_tokens.span()?,
            7 => matched_tokens[2..].span()?,
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "a long time ago",
                span.get_content(source),
            )],
            message: "The correct phrase is `a long time ago`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the missing article `a` in the phrase `long time ago`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::LongTimeAgo;

    #[test]
    fn fix_long_time_ago_mid_sentence() {
        assert_suggestion_result(
            "Simple URL redirect handler written long time ago in PHP - yaroslav-ilin/shortener.",
            LongTimeAgo::default(),
            "Simple URL redirect handler written a long time ago in PHP - yaroslav-ilin/shortener.",
        );
    }

    #[test]
    fn fix_long_time_ago_start_of_sentence() {
        assert_suggestion_result(
            "Long time ago, I wrote this code.",
            LongTimeAgo::default(),
            "A long time ago, I wrote this code.",
        );
    }

    #[test]
    fn dont_flag_a_long_time_ago_mid_sentence() {
        assert_no_lints(
            "Created a long time ago and now finally saved in this gist as a backup.",
            LongTimeAgo::default(),
        );
    }

    #[test]
    fn dont_flag_a_long_time_ago_start_of_sentence() {
        assert_no_lints(
            "a long time ago i had random site redirector on my we site that would send visitors to... - sites.txt.",
            LongTimeAgo::default(),
        );
    }
}
