use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, expr_linter::Sentence},
};

pub struct MultipleFrequencyAdverbs {
    expr: Box<dyn Expr>,
}

impl Default for MultipleFrequencyAdverbs {
    fn default() -> Self {
        let adverb_of_frequency = |t: &Token, s: &[char]| {
            t.kind.is_frequency_adverb()
                && !t
                    .span
                    .get_content(s)
                    .eq_ignore_ascii_case_chars(&['o', 'n', 'l', 'y'])
        };

        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .then(adverb_of_frequency)
                    .then_optional_comma()
                    .t_ws()
                    .then(adverb_of_frequency),
            ),
        }
    }
}

impl ExprLinter for MultipleFrequencyAdverbs {
    // We have to use `Sentence` if our `Expr` includes commas!
    type Unit = Sentence;

    fn description(&self) -> &str {
        "Looks for adjacent adverbs of frequencey, which will be either redundant or contradictory."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (adv1tok, adv2tok) = (toks.first()?, toks.last()?);
        let (adv1span, adv2span) = (adv1tok.span, adv2tok.span);
        let (adv1ch, adv2ch) = (adv1span.get_content(src), adv2span.get_content(src));

        if !adv1ch.eq_ignore_ascii_case_chars(adv2ch) {
            Some(Lint {
                span: toks.span()?,
                lint_kind: LintKind::Usage,
                suggestions: vec![],
                message: format!(
                    "The adverbs of frequency ‘{}’ and ‘{}’ are either redundant or contradictory",
                    adv1ch.to_string(),
                    adv2ch.to_string()
                ),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MultipleFrequencyAdverbs;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn without_comma() {
        assert_lint_count("People have often never even heard of nutrinos, but yeah, about 100 billion solar nutrinos are passing through your thumbnail every second.
", MultipleFrequencyAdverbs::default(), 1);
    }

    #[test]
    fn with_comma() {
        assert_lint_count("often, never", MultipleFrequencyAdverbs::default(), 1);
    }
}
