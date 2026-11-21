use crate::{
    CharStringExt, Lint, Span, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AsSoonAsPossible {
    expr: SequenceExpr,
}

impl Default for AsSoonAsPossible {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("asap")
                .then_optional(SequenceExpr::whitespace().then_word_seq(&["as", "possible"])),
        }
    }
}

impl ExprLinter for AsSoonAsPossible {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let opt_as_before = if let Some(([.., word, ws], _)) = ctx {
            (ws.kind.is_whitespace() && word.kind.is_word() && word.get_ch(src).eq_ch(&['a', 's']))
                .then_some(word)
        } else {
            None
        };

        let (span, offer_asap) = match toks.len() {
            1 => (toks.span()?, false),
            5 => {
                let span = opt_as_before
                    .and_then(|as_tok| toks.span().map(|s| Span::new(as_tok.span.start, s.end)))
                    .or_else(|| toks.span())?;
                (span, true)
            }
            _ => return None,
        };

        // Always all-caps, so not `replace_with_match_case`.
        let offer_asap = offer_asap.then(|| Suggestion::ReplaceWith(vec!['A', 'S', 'A', 'P']));

        // We can't use `replace_with_match_case` because it's by character index
        // and will cause unexpected characters to be uppercased if ASAP was uppercase.
        // Instead, if "ASAP" is capitalized and followed by all-caps "AS", we suggest an all-caps expansion.
        let offer_to_expand = Suggestion::ReplaceWith(
            if toks[0].get_ch(src) == ['A', 'S', 'A', 'P']
                && toks.get(2).map(|t| t.get_ch(src)) == Some(&['A', 'S'])
            {
                "AS SOON AS POSSIBLE".chars().collect()
            } else {
                "as soon as possible".chars().collect()
            },
        );

        let suggestions: Vec<_> = offer_asap
            .into_iter()
            .chain(std::iter::once(offer_to_expand))
            .collect();

        let (lint_kind, message) = if suggestions.len() == 1 {
            (
                LintKind::Miscellaneous,
                // As in `initialism_linter.rs`
                "Try expanding this initialism.".to_owned(),
            )
        } else {
            (
                LintKind::Redundancy,
                "You can avoid redundancy since `ASAP` already means `as soon as possible`."
                    .to_owned(),
            )
        };

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
        "Looks for redundant use of the initialism `ASAP` and offers to expand when not redundant."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result};

    use super::AsSoonAsPossible;

    #[test]
    fn corrects_asap() {
        assert_suggestion_result(
            "Please respond asap.",
            AsSoonAsPossible::default(),
            "Please respond as soon as possible.",
        );
    }

    #[test]
    fn corrects_asap_caps_as_possible() {
        assert_good_and_bad_suggestions(
            "Regardless, let's get the deprecation in ASAP as possible to signal to users to stop using it.",
            AsSoonAsPossible::default(),
            &[
                "Regardless, let's get the deprecation in as soon as possible to signal to users to stop using it.",
                "Regardless, let's get the deprecation in ASAP to signal to users to stop using it.",
            ],
            &[],
        );
    }

    #[test]
    fn corrects_asap_lower_as_possible() {
        assert_good_and_bad_suggestions(
            "I need $1000 asap as possible",
            AsSoonAsPossible::default(),
            &["I need $1000 as soon as possible", "I need $1000 ASAP"],
            &[],
        );
    }

    #[test]
    fn corrects_asap_as_possible_all_caps() {
        assert_good_and_bad_suggestions(
            "WELL JUST TELL HIM TO CALL ME ASAP AS POSSIBLE.",
            AsSoonAsPossible::default(),
            &[
                "WELL JUST TELL HIM TO CALL ME AS SOON AS POSSIBLE.",
                "WELL JUST TELL HIM TO CALL ME ASAP.",
            ],
            &[],
        );
    }

    #[test]
    fn corrects_as_asap_caps_as_possible() {
        assert_good_and_bad_suggestions(
            "Please feel free just to send one of us a brief, descriptive email with your question, and we'll do our best to get back to you as ASAP as possible.",
            AsSoonAsPossible::default(),
            &[
                "Please feel free just to send one of us a brief, descriptive email with your question, and we'll do our best to get back to you as soon as possible.",
                "Please feel free just to send one of us a brief, descriptive email with your question, and we'll do our best to get back to you ASAP.",
            ],
            &[],
        );
    }

    #[test]
    fn corrects_as_asap_lower_as_possible() {
        assert_good_and_bad_suggestions(
            "security updates should still be created as asap as possible.",
            AsSoonAsPossible::default(),
            &[
                "security updates should still be created as soon as possible.",
                "security updates should still be created ASAP.",
            ],
            &[],
        );
    }
}
