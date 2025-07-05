use crate::{
    expr::{Expr, SequenceExpr}, linting::{ExprLinter, Lint, LintKind, Suggestion}, Token, TokenStringExt
};

pub struct AdjectiveDoubleDegree {
    expr: Box<dyn Expr>,
}

impl Default for AdjectiveDoubleDegree {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::word_set(&["more", "most"]).t_ws().then(
                |tok: &Token, _src: &[char]| {
                    eprintln!("🍅 '{:?}'", tok.span.get_content_string(_src));
                    // tok.kind.is_comparative_adjective() || tok.kind.is_superlative_adjective()
                    let (cmp, sup) = (tok.kind.is_comparative_adjective(), tok.kind.is_superlative_adjective());
                    eprintln!("🍅🍅 '{:?}'", (cmp, sup));
                    cmp || sup
                },
            )),
        }
    }
}

impl ExprLinter for AdjectiveDoubleDegree {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        eprintln!("🍏 '{}'", toks.span()?.get_content_string(src));

        // TODO: implement
        None
    }

    fn description(&self) -> &'static str {
        "Finds adjectives that are used as double degrees (e.g. 'more prettier')."
    }
}

#[cfg(test)]
mod tests {
    use super::AdjectiveDoubleDegree;
    use crate::linting::tests::{assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result};

    #[test]
    fn flag_more_prettier() {
        assert_lint_count("more prettier", AdjectiveDoubleDegree::default(), 1);
    }

    #[test]
    fn fix_more_prettier() {
        assert_suggestion_result("more prettier", AdjectiveDoubleDegree::default(), "prettier");
    }

    #[test]
    fn flag_most_prettiest() {
        assert_lint_count("most prettiest", AdjectiveDoubleDegree::default(), 1);
    }

    #[test]
    fn fix_most_prettiest() {
        assert_suggestion_result("most prettiest", AdjectiveDoubleDegree::default(), "prettiest");
    }

    #[test]
    fn flag_more_better() {
        assert_lint_count("more better", AdjectiveDoubleDegree::default(), 1);
    }

    #[test]
    fn fix_more_better() {
        assert_suggestion_result("more better", AdjectiveDoubleDegree::default(), "better");
    }

    #[test]
    fn flag_most_best() {
        assert_lint_count("most best", AdjectiveDoubleDegree::default(), 1);
    }

    #[test]
    fn fix_most_best() {
        assert_suggestion_result("most best", AdjectiveDoubleDegree::default(), "best");
    }

    #[test]
    fn dont_flag_more_best_practices() {
        assert_lint_count("more best practices", AdjectiveDoubleDegree::default(), 0);
    }
}