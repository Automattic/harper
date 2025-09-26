use crate::CharStringExt;
use crate::linting::expr_linter::find_the_only_token_matching;
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::{
    Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
};

pub struct MixedBag {
    expr: Box<dyn Expr>,
}

impl Default for MixedBag {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .then_kind_any_or_words(
                        &[TokenKind::is_adjective, TokenKind::is_adverb] as &[_],
                        &["a"],
                    )
                    .t_ws()
                    .t_aco("mixed")
                    .t_ws()
                    .t_aco("bad"),
            ),
        }
    }
}

impl ExprLinter for MixedBag {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let bad_span = find_the_only_token_matching(toks, src, |tok, _src| {
            tok.span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['b', 'a', 'd'])
        })?
        .span;

        Some(Lint {
            span: bad_span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "bag",
                bad_span.get_content(src),
            )],
            message: "Corrects the eggcorn `mixed bad` to `mixed bag`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the eggcorn `mixed bad` to `mixed bag`."
    }
}

#[cfg(test)]
mod tests {
    use super::MixedBag;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn a_mixed_bad() {
        assert_suggestion_result(
            "CommandLine interface is already a mixed bad of wstring and #ifdef to   string or wstring.",
            MixedBag::default(),
            "CommandLine interface is already a mixed bag of wstring and #ifdef to   string or wstring.",
        );
    }

    #[test]
    fn big_mixed_bag() {
        assert_suggestion_result(
            "Speaking of dungeons , the dungeons in this game are a big mixed bad.",
            MixedBag::default(),
            "Speaking of dungeons , the dungeons in this game are a big mixed bag.",
        );
    }

    #[test]
    fn damn_mixed_bag() {
        assert_suggestion_result(
            "This is a damn mixed bad which left me frustrated, and yet longing for more.",
            MixedBag::default(),
            "This is a damn mixed bag which left me frustrated, and yet longing for more.",
        );
    }

    #[test]
    fn huge_mixed_bag() {
        assert_suggestion_result(
            "Also a huge mixed bad of no name monitors of different sizes that all have different color settings on.",
            MixedBag::default(),
            "Also a huge mixed bag of no name monitors of different sizes that all have different color settings on.",
        );
    }

    #[test]
    fn large_mixed_bag() {
        assert_suggestion_result(
            "I’m still struggling to comprehend how it throws such a large mixed bad of symptoms in the mix this time.",
            MixedBag::default(),
            "I’m still struggling to comprehend how it throws such a large mixed bag of symptoms in the mix this time.",
        );
    }

    #[test]
    fn massive_mixed_bad() {
        assert_suggestion_result(
            "Anyway. In topic, Swano was a massive mixed bad in this game.",
            MixedBag::default(),
            "Anyway. In topic, Swano was a massive mixed bag in this game.",
        );
    }

    #[test]
    fn massively_mixed_bag() {
        assert_suggestion_result(
            "While certain things are more common to be either way, it's a massively mixed bad overall.",
            MixedBag::default(),
            "While certain things are more common to be either way, it's a massively mixed bag overall.",
        );
    }

    #[test]
    fn pretty_mixed_bag() {
        assert_suggestion_result(
            "It's a pretty mixed bad for me: Evolution Xavier for comic Xavier. Evolution Magneto for comic Magneto.",
            MixedBag::default(),
            "It's a pretty mixed bag for me: Evolution Xavier for comic Xavier. Evolution Magneto for comic Magneto.",
        );
    }

    #[test]
    fn rather_mixed_bag() {
        assert_suggestion_result(
            "Well chaps, as expected the TS contains a rather mixed bad of promise and disappointment.",
            MixedBag::default(),
            "Well chaps, as expected the TS contains a rather mixed bag of promise and disappointment.",
        );
    }

    #[test]
    fn really_mixed_bag() {
        assert_suggestion_result(
            "This is a really mixed bad On one hand you have some of Eminem's highest highs and his lowest lows but ever.",
            MixedBag::default(),
            "This is a really mixed bag On one hand you have some of Eminem's highest highs and his lowest lows but ever.",
        );
    }

    #[test]
    fn slightly_mixed_bag() {
        assert_suggestion_result(
            "I absolutely love Yes Minister and Yes Prime Minister but it did end up a slightly mixed bad in terms of impact.",
            MixedBag::default(),
            "I absolutely love Yes Minister and Yes Prime Minister but it did end up a slightly mixed bag in terms of impact.",
        );
    }

    #[test]
    fn somewhat_mixed_bag() {
        assert_suggestion_result(
            "A somewhat mixed bad. The space is pleasant with a rustic vibe.",
            MixedBag::default(),
            "A somewhat mixed bag. The space is pleasant with a rustic vibe.",
        );
    }

    #[test]
    fn very_mixed_bag() {
        assert_suggestion_result(
            "AVAILABLE MEN is a very mixed bad of short films about gay subjects that very from excellent to weak.",
            MixedBag::default(),
            "AVAILABLE MEN is a very mixed bag of short films about gay subjects that very from excellent to weak.",
        );
    }
}
