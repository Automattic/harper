use crate::{
    Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
};

pub struct ByOnesOwn {
    expr: SequenceExpr,
}

impl Default for ByOnesOwn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("by")
                .t_ws()
                .then_possessive_determiner()
                .t_ws()
                .t_aco("own"),
        }
    }
}

impl ExprLinter for ByOnesOwn {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // If any word comes next then "own" is probably a qualifier.
        if followed_by_word(ctx, |_| true) {
            return None;
        }

        let span = toks.span()?;

        let (by_idx, ws1_idx, whose_idx) = (0, 1, 2);

        let by_ws = toks[by_idx..=ws1_idx].span()?.get_content(src);
        let ws_1 = toks[ws1_idx].get_ch(src);

        // my, its
        let whose = toks[whose_idx].get_ch(src);
        // my own, its own
        let whose_own = toks[whose_idx..].span()?.get_content(src);

        // my -> myself, its -> itself
        let reflexive = match whose {
            _ if whose.eq_ch(&['m', 'y']) => &["myself"][..],
            _ if whose.eq_ch(&['o', 'u', 'r']) => &["ourselves"][..],
            _ if whose.eq_ch(&['y', 'o', 'u', 'r']) => &["yourself", "yourselves"][..],
            _ if whose.eq_ch(&['h', 'i', 's']) => &["himself"][..],
            _ if whose.eq_ch(&['h', 'e', 'r']) => &["herself"][..],
            _ if whose.eq_ch(&['i', 't', 's']) => &["itself"][..],
            _ if whose.eq_ch(&['t', 'h', 'e', 'i', 'r']) => &["themselves", "themself"][..],
            _ if whose.eq_ch(&['o', 'n', 'e', '\'', 's']) => &["oneself"][..],
            _ => return None, // Actually unreachable
        };

        let sugg_on_ones_own = Suggestion::replace_with_match_case(
            "on".chars()
                .chain(ws_1.iter().copied())
                .chain(whose_own.iter().copied())
                .collect::<Vec<char>>(),
            span.get_content(src),
        );

        let suggestions = std::iter::once(sugg_on_ones_own)
            .chain(reflexive.iter().map(|reflexive_str| {
                let by_whoseself = by_ws
                    .iter()
                    .copied()
                    .chain(reflexive_str.chars())
                    .collect::<Vec<char>>();
                Suggestion::replace_with_match_case(by_whoseself, span.get_content(src))
            }))
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message:
                "`By one's own` is not idiomatic. Consider either `on one's own` or `by oneself`."
                    .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Fixes incorrect phrases like `by my own` by suggesting `on my own` or `by myself`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    use super::ByOnesOwn;

    #[test]
    fn fix_by_her_own() {
        assert_good_and_bad_suggestions(
            "If she is not managing by her own, it means that there is something wrong with either our documentation, or our error messages, or our ...",
            ByOnesOwn::default(),
            &[
                "If she is not managing on her own, it means that there is something wrong with either our documentation, or our error messages, or our ...",
                "If she is not managing by herself, it means that there is something wrong with either our documentation, or our error messages, or our ...",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_her_breathing() {
        assert_no_lints(
            "The silence of this deep section of the Spire was profound, broken only by her own ragged breathing.",
            ByOnesOwn::default(),
        );
    }

    #[test]
    fn fix_by_his_own() {
        assert_good_and_bad_suggestions(
            "And make the hover info function creating the <title> tag by his own.",
            ByOnesOwn::default(),
            &[
                "And make the hover info function creating the <title> tag on his own.",
                "And make the hover info function creating the <title> tag by himself.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_by_his_own_comma() {
        assert_good_and_bad_suggestions(
            "it also makes a report all by his own, without touching anything",
            ByOnesOwn::default(),
            &[
                "it also makes a report all on his own, without touching anything",
                "it also makes a report all by himself, without touching anything",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_his_own_efforts() {
        assert_no_lints(
            "My point is if someone needs jemalloc he gets it by his own efforts clearly realising why/when and so on.",
            ByOnesOwn::default(),
        );
    }

    #[test]
    fn fix_by_its_own_in_quotes() {
        assert_good_and_bad_suggestions(
            "I set AdGuard DNS via Windows 11 settings, and I always found them to be changed \"by its own\".",
            ByOnesOwn::default(),
            &[
                "I set AdGuard DNS via Windows 11 settings, and I always found them to be changed \"on its own\".",
                "I set AdGuard DNS via Windows 11 settings, and I always found them to be changed \"by itself\".",
            ],
            &[],
        );
    }

    #[test]
    fn fix_by_its_own_at_end() {
        assert_good_and_bad_suggestions(
            "The wheel starts speeding and spinning by its own.",
            ByOnesOwn::default(),
            &[
                "The wheel starts speeding and spinning on its own.",
                "The wheel starts speeding and spinning by itself.",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_its_own_changes() {
        assert_no_lints("Remix is spooked by its own changes", ByOnesOwn::default());
    }

    #[test]
    fn fix_by_my_own() {
        assert_good_and_bad_suggestions(
            "how can I build a project by my own",
            ByOnesOwn::default(),
            &[
                "how can I build a project on my own",
                "how can I build a project by myself",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_my_own_hr() {
        assert_no_lints(
            "My colledge' ID is totally messed up by my own HR",
            ByOnesOwn::default(),
        );
    }

    #[test]
    fn fix_by_our_own() {
        assert_good_and_bad_suggestions(
            "So, I think we should have the alternative BN implementation by our own.",
            ByOnesOwn::default(),
            &[
                "So, I think we should have the alternative BN implementation on our own.",
                "So, I think we should have the alternative BN implementation by ourselves.",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_our_own_standards() {
        assert_no_lints("Better by our own standards.", ByOnesOwn::default());
    }

    #[test]
    fn fix_by_their_own() {
        assert_good_and_bad_suggestions(
            "Therefore, it would be great to allow the users to do the serialization by their own, rather than giving the full control to the SDK internally.",
            ByOnesOwn::default(),
            &[
                "Therefore, it would be great to allow the users to do the serialization on their own, rather than giving the full control to the SDK internally.",
                "Therefore, it would be great to allow the users to do the serialization by themselves, rather than giving the full control to the SDK internally.",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_their_own_bots() {
        assert_no_lints(
            "GitHub needs a way to report issues and comments, even by their own bots.",
            ByOnesOwn::default(),
        );
    }

    #[test]
    fn fix_by_your_own() {
        assert_good_and_bad_suggestions(
            "You should be able to see a new wallpaper slideshow created by your own.",
            ByOnesOwn::default(),
            &[
                "You should be able to see a new wallpaper slideshow created on your own.",
                "You should be able to see a new wallpaper slideshow created by yourself.",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_by_your_own_lib() {
        assert_no_lints(
            "Design with JSX, powered by your own component library.",
            ByOnesOwn::default(),
        );
    }

    // For future contributors who want to improve this linter

    #[test]
    #[ignore = "We can't detect the end of sentence when punctuation is missing."]
    fn dont_flag_by_its_own_end_no_punc() {
        assert_no_lints(
            "same infininte running and when i stop this node forcefully it show s the currect output dont know why it dont stop by its own i think its a bug",
            ByOnesOwn::default(),
        );
    }

    #[test]
    #[ignore = "We might be able to handle this one"]
    fn fix_by_your_own_and() {
        assert_good_and_bad_suggestions(
            "After you install SPAdes by your own and add spades bin to the $PATH, ",
            ByOnesOwn::default(),
            &[
                "After you install SPAdes on your own and add spades bin to the $PATH, ",
                "After you install SPAdes by yourself and add spades bin to the $PATH, ",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "A construct that needs to be fixed that we can't handle yet"]
    fn fix_by_your_own_using() {
        assert_good_and_bad_suggestions(
            "I made some steps to this direction and currently you can implement it by your own using middleware",
            ByOnesOwn::default(),
            &[
                "I made some steps in this direction and currently you can implement it on your own using middleware",
                "I made some steps in this direction and currently you can implement it by yourself using middleware",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "We might be able to handle this"]
    fn fix_by_your_own_preposition() {
        assert_good_and_bad_suggestions(
            "Of course feel free to add this by your own by pull request :)",
            ByOnesOwn::default(),
            &[
                "Of course feel free to add this on your own by pull request :)",
                "Of course feel free to add this by yourself by pull request :)",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "Another construction we should fix but can't yet"]
    fn fix_by_your_own_we() {
        assert_good_and_bad_suggestions(
            "If you would like to develop this feature by your own we would be happy to help with this but",
            ByOnesOwn::default(),
            &[
                "If you would like to develop this feature on your own we would be happy to help with this but",
                "If you would like to develop this feature by yourself we would be happy to help with this but",
            ],
            &[],
        );
    }

    #[test]
    #[ignore = "See if it's always a legit error when the next word is a preposition"]
    fn fix_by_her_own_during() {
        assert_good_and_bad_suggestions(
            "most of the time she gets mad by her own during topics which can end her in an angry/annoyed mood",
            ByOnesOwn::default(),
            &[
                "most of the time she gets mad on her own during topics which can end her in an angry/annoyed mood",
                "most of the time she gets mad by herself during topics which can end her in an angry/annoyed mood",
            ],
            &[],
        );
    }
}
