use crate::{
    CharStringExt, Lint, Span, Token, TokenStringExt,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct LifeSaver {
    expr: SequenceExpr,
}

impl Default for LifeSaver {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(WordSet::new([
                    "lifesafer",
                    "lifesafers",
                    "livesafer",
                    "livesafers",
                    "livesaver",
                    "livesavers",
                ])) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(["life", "live"])
                        .t_ws_h()
                        .t_set(["safer", "safers", "saver", "savers"])
                        .but_not(
                            SequenceExpr::aco("life")
                                .t_ws_h()
                                .t_set(["saver", "savers"]),
                        ),
                ),
            ]),
        }
    }
}

impl ExprLinter for LifeSaver {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        let (l_span, s_span) = match toks.len() {
            1 => (
                Span::new(toks[0].span.start, toks[0].span.start + 4),
                Span::new(toks[0].span.start + 4, toks[0].span.end),
            ),
            3 => (toks[0].span, toks[2].span),
            _ => return None,
        };

        enum CompoundKind {
            Closed,
            Open,
            Hyphenated,
        }

        let ck: CompoundKind = match toks.len() {
            1 => CompoundKind::Closed,
            3 => {
                if toks[1].kind.is_hyphen() {
                    CompoundKind::Hyphenated
                } else {
                    CompoundKind::Open
                }
            }
            _ => return None,
        };

        let pl = s_span
            .get_content(src)
            .ends_with_ignore_ascii_case_chars(&['s']);

        eprintln!(
            "🚨 🛟 “{}” 🛟 {} 🛟 “{}” 🛟",
            l_span.get_content_string(src),
            match ck {
                CompoundKind::Closed => "🔒",
                CompoundKind::Hyphenated => "🔗",
                CompoundKind::Open => "🔓",
            },
            s_span.get_content_string(src)
        );

        let fixed: Vec<char> = match (ck, pl) {
            (CompoundKind::Closed, false) => "lifesaver",
            (CompoundKind::Closed, true) => "lifesavers",
            (CompoundKind::Hyphenated, false) => "life-saver",
            (CompoundKind::Hyphenated, true) => "life-savers",
            (CompoundKind::Open, false) => "life saver",
            (CompoundKind::Open, true) => "life savers",
        }
        .chars()
        .collect();

        let message = format!("Did you mean `{}`?", fixed.to_string());

        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::replace_with_match_case(
                fixed,
                span.get_content(src),
            )],
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of `lifesaver`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::LifeSaver;

    #[test]
    fn closed_sg_vf() {
        assert_suggestion_result(
            "Your such a livesafer!",
            LifeSaver::default(),
            "Your such a lifesaver!",
        );
    }

    #[test]
    fn open_pl_vv() {
        assert_suggestion_result(
            "Their such live savers too!",
            LifeSaver::default(),
            "Their such life savers too!",
        );
    }

    #[test]
    fn hyphen_pl_ff() {
        assert_suggestion_result(
            "Were all such life-safers!",
            LifeSaver::default(),
            "Were all such life-savers!",
        );
    }

    #[test]
    fn dont_flag_closed_sg() {
        assert_no_lints("lifesaver", LifeSaver::default())
    }

    #[test]
    fn dont_flag_open_sg() {
        assert_no_lints("life saver", LifeSaver::default())
    }

    #[test]
    fn dont_flag_hyphen_pl() {
        assert_no_lints("life-savers", LifeSaver::default())
    }
}
