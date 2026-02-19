use crate::expr::SequenceExpr;
use crate::expr::{Expr, OwnedExprExt};
use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct WinPrize {
    expr: Box<dyn Expr>,
}

impl Default for WinPrize {
    fn default() -> Self {
        let verbs = Lrc::new(WordSet::new(&["win", "wins", "won", "winning"]));
        let miss = Lrc::new(WordSet::new(&["price", "prices", "prise", "prises"]));

        let pattern = SequenceExpr::with(verbs.clone())
            .then_whitespace()
            .then_determiner()
            .then_whitespace()
            .then(miss.clone())
            .or_longest(SequenceExpr::with(verbs).then_whitespace().then(miss));

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for WinPrize {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let candidate = toks.last()?;
        let raw = candidate.span.get_content_string(src).to_lowercase();
        let repl = match raw.as_str() {
            "price" | "prise" => "prize",
            "prices" | "prises" => "prizes",
            _ => return None,
        };

        Some(Lint {
            span: candidate.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(repl.chars().collect())],
            message: format!("Perhaps you meant `{repl}`, the word for an award."),
            priority: 50,
        })
    }

    fn description(&self) -> &str {
        "Catches the mix-up between `price`/`prise` and `prize` after the verb `win`."
    }
}

#[cfg(test)]
mod tests {
    use super::WinPrize;
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
    };

    #[test]
    fn fix_price_singular() {
        assert_suggestion_result(
            "Lena won a price in the coding marathon.",
            WinPrize::default(),
            "Lena won a prize in the coding marathon.",
        );
    }

    #[test]
    fn fix_price_plural() {
        assert_top3_suggestion_result(
            "Our team won the prices announced yesterday.",
            WinPrize::default(),
            "Our team won the prizes announced yesterday.",
        );
    }

    #[test]
    fn fix_prise_singular() {
        assert_suggestion_result(
            "He finally won the prise he'd dreamed of.",
            WinPrize::default(),
            "He finally won the prize he'd dreamed of.",
        );
    }

    #[test]
    fn fix_prise_plural() {
        assert_suggestion_result(
            "The inventors won several prises at the expo.",
            WinPrize::default(),
            "The inventors won several prizes at the expo.",
        );
    }

    #[test]
    fn ignore_correct_prize() {
        assert_lint_count(
            "Miranda won the grand prize last year.",
            WinPrize::default(),
            0,
        );
    }

    #[test]
    fn fix_no_det() {
        assert_suggestion_result("I won prices!", WinPrize::default(), "I won prizes!");
    }
}
