mod four_digits;
mod two_digits;

use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, expr_linter::Sentence},
};

use four_digits::match_to_lint_four_digits;
use two_digits::match_to_lint_two_digits;

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

        let (decade_chars, s_chars) =
            (toks[0].span.get_content(src), toks[2].span.get_content(src));

        let (before_context, after_context): (Option<&[Token]>, Option<&[Token]>) = match ctx {
            Some((pw, nw)) => {
                if pw.is_empty() {
                    if nw.is_empty() {
                        (None, None)
                    } else {
                        (None, Some(nw))
                    }
                } else if nw.is_empty() {
                    (Some(pw), None)
                } else {
                    (Some(pw), Some(nw))
                }
            }
            None => (None, None),
        };

        if decade_chars.len() == 4 {
            match_to_lint_four_digits(
                toks,
                src,
                decade_chars,
                s_chars,
                before_context,
                after_context,
            )
        } else {
            match_to_lint_two_digits(
                toks,
                src,
                decade_chars,
                s_chars,
                before_context,
                after_context,
            )
        }
    }
}
