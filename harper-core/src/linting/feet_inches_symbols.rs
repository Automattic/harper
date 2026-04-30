use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FeetInchesSymbols {
    expr: SequenceExpr,
}

impl Default for FeetInchesSymbols {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_cardinal_number()
                .then_optional(crate::patterns::WhitespacePattern)
                .then_apostrophe()
                .then_optional(crate::patterns::WhitespacePattern)
                .then_cardinal_number()
                .then_optional(crate::patterns::WhitespacePattern)
                .then_quote(),
        }
    }
}

impl ExprLinter for FeetInchesSymbols {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Pattern: number [ws]? ' [ws]? number [ws]? "
        // Expected tokens: 1-2 for feet part, 3-6 for inches part
        if toks.len() < 4 {
            return None;
        }

        let inch_token_idx = toks.len() - 1;

        // Check that last token is a quote (inches symbol)
        if !toks[inch_token_idx].kind.is_quote() {
            return None;
        }

        // Find the apostrophe (feet symbol) at token 1 or 2
        let foot_symbol_idx = toks
            .iter()
            .skip(1) // Skip token 0 (always a number)
            .take(2) // Check tokens 1, 2 for foot symbol
            .position(|tok| tok.kind.is_apostrophe())
            .map(|pos| pos + 1)?; // Add 1 to account for skipped token 0

        // We replace the span from foot symbol to inch symbol, including any whitespace between them.
        let toks = &toks[foot_symbol_idx..=inch_token_idx];

        // Retaining any whitespace between the foot and inch symbols as-is.
        let inner = &toks[1..toks.len() - 1];

        let (span, between) = (toks.span()?, inner.span()?);

        let replacement = std::iter::once('′')
            .chain(between.get_content(src).iter().copied())
            .chain(std::iter::once('″'))
            .collect::<Vec<char>>();

        Some(Lint {
            span,
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(replacement)],
            message: "For feet and inches, use the prime and double prime symbols (′ and ″) instead of apostrophes/quotes.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the use of ASCII apostrophes and quotes for feet and inches to Unicode prime and double prime symbols."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::FeetInchesSymbols;

    #[test]
    fn without_spaces_after() {
        assert_suggestion_result(
            "i'm 5'7\", 140 pounds and the small was a perfect fit for me.",
            FeetInchesSymbols::default(),
            "i'm 5′7″, 140 pounds and the small was a perfect fit for me.",
        );
    }

    #[test]
    fn with_spaces_after() {
        assert_suggestion_result(
            "i'm a tall gal, 5' 11\" with broad shoulders and i weigh about 165",
            FeetInchesSymbols::default(),
            "i'm a tall gal, 5′ 11″ with broad shoulders and i weigh about 165",
        );
    }

    #[test]
    fn with_spaces_before() {
        assert_suggestion_result(
            "I'm about 6 ' 1 \" tall.",
            FeetInchesSymbols::default(),
            "I'm about 6 ′ 1 ″ tall.",
        );
    }
}
