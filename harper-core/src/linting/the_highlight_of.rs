use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, find_the_only_token_index_matching, followed_by_word},
    },
    patterns::InflectionOfBe,
};

pub struct TheHighlightOf {
    expr: FirstMatchOf,
}

impl Default for TheHighlightOf {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::any_of([
                        Box::new(InflectionOfBe::default()) as Box<dyn Expr>,
                        // Box::new(|t: &Token, _: &[char]| t.kind.is_frequency_adverb()),
                        Box::new(|t: &Token, _: &[char]| t.kind.is_adverb()),
                    ])
                    .t_ws()
                    .then_word_seq(&["the", "highlight", "to"]),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["the", "highlight", "to"])
                        .t_ws()
                        .then_possessive_determiner(),
                ),
            ]),
        }
    }
}

impl ExprLinter for TheHighlightOf {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        let i = find_the_only_token_index_matching(toks, src, |t, _s| {
            t.get_ch(src).eq_str("highlight")
        })?;

        // be the highlight to -> highlight = 4 -> might need the next word
        // the highlight to my -> 2 -> might need the previous word

        if i == 4
            && followed_by_word(ctx, |t: &Token| {
                t.kind.is_verb_lemma() || t.get_ch(src).eq_str("date")
            })
        {
            return None;
        }

        let span = toks.get(i + 2)?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                span.get_content(src),
            )],
            message: "Use `highlight of` rather than `highlight to`".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `the highlight to` to `the highlight of`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::TheHighlightOf;

    #[test]
    fn always_the_hl_to_my() {
        assert_suggestion_result(
            "Your posts are always the highlight to my day!",
            TheHighlightOf::default(),
            "Your posts are always the highlight of my day!",
        );
    }

    #[test]
    fn definitely_the_hl_to_my() {
        assert_suggestion_result(
            "Staying here is definitely the highlight to my trip to Bali.",
            TheHighlightOf::default(),
            "Staying here is definitely the highlight of my trip to Bali.",
        )
    }

    #[test]
    fn are_the_hl_to_my() {
        assert_suggestion_result(
            "My children are the highlight to my life, and this little guy just takes the cake.",
            TheHighlightOf::default(),
            "My children are the highlight of my life, and this little guy just takes the cake.",
        );
    }

    #[test]
    fn be_the_hl_to_my() {
        assert_suggestion_result(
            "Be the highlight to my season and let me steal you for Prom ??",
            TheHighlightOf::default(),
            "Be the highlight of my season and let me steal you for Prom ??",
        );
    }

    #[test]
    fn been_the_hl_to_my() {
        assert_suggestion_result(
            "BN Collection has been the highlight to my year in gaming//Questions about the Starforce games.",
            TheHighlightOf::default(),
            "BN Collection has been the highlight of my year in gaming//Questions about the Starforce games.",
        );
    }

    #[test]
    fn is_the_hl_to_my() {
        assert_suggestion_result(
            "and Now The Gym Is The Highlight to My Day",
            TheHighlightOf::default(),
            "and Now The Gym Is The Highlight of My Day",
        );
    }

    #[test]
    fn was_the_hl_to_my() {
        assert_suggestion_result(
            "Yesterday was the highlight to my entire wk",
            TheHighlightOf::default(),
            "Yesterday was the highlight of my entire wk",
        );
    }

    #[test]
    fn the_hl_to_my() {
        assert_suggestion_result(
            "Missed my alarm, was about to miss the highlight to my weekend",
            TheHighlightOf::default(),
            "Missed my alarm, was about to miss the highlight of my weekend",
        );
    }

    #[test]
    fn the_hl_to_our() {
        assert_suggestion_result(
            "The highlight to our Saturday!",
            TheHighlightOf::default(),
            "The highlight of our Saturday!",
        )
    }

    #[test]
    fn dont_flag_to_end() {
        assert_no_lints("...highlight to end his day", TheHighlightOf::default())
    }

    #[test]
    fn dont_flag_to_enable() {
        assert_no_lints(
            "...highlight to enable the worry-free entities.",
            TheHighlightOf::default(),
        )
    }
}
