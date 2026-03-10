use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, expr_linter::Sentence},
};

mod four_digits;
mod two_digits;

use four_digits::match_to_lint_four_digits;

pub struct PluralDecades {
    expr: SequenceExpr,
}

impl Default for PluralDecades {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_cardinal_number()
                .then_apostrophe()
                .t_aco("s"),
        }
    }
}

impl ExprLinter for PluralDecades {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags plural decades erroneously using an apostrophe before the `s`"
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // eprintln!("📅 {}", crate::linting::debug::format_lint_match(toks, ctx, src));
        if toks.len() != 3 {
            return None;
        }

        let (decade_chars, _s_chars) =
            (toks[0].span.get_content(src), toks[2].span.get_content(src));

        // TODO does not yet support two-digit decades like 80's
        // if decade_chars.len() != 4 || !decade_chars.ends_with(&['0']) {
        if ![2, 4].contains(&decade_chars.len()) || !decade_chars.ends_with(&['0']) {
            return None;
        }

        // TODO delegate to four_digits.rs or two_digits.rs

        if decade_chars.len() == 4 {
            match_to_lint_four_digits(toks, src, ctx)
        } else {
            // TODO delegate to two_digits.rs
            None
        }
    }
}
