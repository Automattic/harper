use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct NakedEye {
    expr: SequenceExpr,
}

impl Default for NakedEye {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::optional(
                SequenceExpr::word_set(&[
                    "hide", "hides", "hid", "hidden", "hiding", "keep", "keeps", "kept", "keeping",
                ])
                .t_ws()
                .then_optional(
                    SequenceExpr::optional(
                        SequenceExpr::default()
                            .then_kind_any(&[
                                TokenKind::is_determiner as fn(&TokenKind) -> bool,
                                TokenKind::is_adjective,
                                TokenKind::is_oov,
                            ])
                            .t_ws(),
                    )
                    .then_noun()
                    .t_ws(),
                ),
            )
            .then_preposition()
            .t_ws()
            .t_aco("the")
            .t_ws()
            .t_aco("naked")
            .t_ws()
            .t_set(&["eye", "eyes"]),
        }
    }
}

impl ExprLinter for NakedEye {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (verb, _adj, _noun, prep, _idiom) = match toks.len() {
            7 => (None, None, None, &toks[0], &toks[2..]),
            9 => (Some(&toks[0]), None, None, &toks[2], &toks[4..]),
            11 => (Some(&toks[0]), None, Some(&toks[2]), &toks[4], &toks[6..]),
            13 => (
                Some(&toks[0]),
                Some(&toks[2]),
                Some(&toks[4]),
                &toks[6],
                &toks[8..],
            ),
            _ => return None,
        };

        let prep = prep.span;
        let prepch = prep.get_content(src);

        match (
            verb.is_some(),
            prepch.eq_ch(&['f', 'r', 'o', 'm']),
            prepch.eq_any_ignore_ascii_case_str(&["to", "with", "by"]),
        ) {
            (_, _, true) => return None,    // known good preposition
            (true, true, _) => return None, // hide from
            (_, false, _) => return None,   // any other preposition
            (_, true, _) => (),             // from
        };

        let suggestions = ["to", "with", "by"]
            .into_iter()
            .map(|p| Suggestion::replace_with_match_case_str(p, prep.get_content(src)))
            .collect();

        Some(Lint {
            span: prep,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message: "This idiom usually requires a different preposition.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the wrong preposition used instead of `to`, `with`, or `by` the naked eye."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::NakedEye;

    #[test]
    #[ignore = "Underscores are not parts of words"]
    fn dont_flag_hiding_adj_noun_from() {
        assert_no_lints(
            "Once the opt-in is complete, the obtained lock() and unlock() utilities enable hiding __experimental APIs from the naked eye:",
            NakedEye::default(),
        );
    }

    #[test]
    fn dont_flag_easily_hide_noun_from() {
        assert_no_lints(
            "But there is something we forgot about, something that allows one to easily hide things from the naked eye ... macros!",
            NakedEye::default(),
        );
    }

    #[test]
    fn dont_flag_well_hidden_from() {
        assert_no_lints(
            "We successfully tampered with the release, well hidden from the naked eye.",
            NakedEye::default(),
        );
    }

    #[test]
    fn but_from_the_naked_eye() {
        assert_suggestion_result(
            "As you can see from the pic the pedestrian is being detected but from the naked eye it can't be seen due to the slope.",
            NakedEye::default(),
            "As you can see from the pic the pedestrian is being detected but to the naked eye it can't be seen due to the slope.",
        );
    }

    #[test]
    fn even_from_the_naked_eye() {
        assert_suggestion_result(
            "the Photoshop one when even from the naked eye you can tell that if you were to zoom into",
            NakedEye::default(),
            "the Photoshop one when even with the naked eye you can tell that if you were to zoom into",
        );
    }

    #[test]
    fn looks_legit_to_the_naked_eye() {
        assert_suggestion_result(
            "In fact, it looks like legit 1080p from the naked eye I guess... 😭",
            NakedEye::default(),
            "In fact, it looks like legit 1080p to the naked eye I guess... 😭",
        );
    }

    #[test]
    fn observed_from_the_naked_eye() {
        assert_suggestion_result(
            "... when observed from the naked eye for the bone micro-architecture of the osteoporotic and healthy cases ...",
            NakedEye::default(),
            "... when observed with the naked eye for the bone micro-architecture of the osteoporotic and healthy cases ...",
        );
    }

    #[test]
    fn being_private_hidden_from() {
        assert_no_lints(
            "... which makes the entire concept of it being private hidden from the naked eye.",
            NakedEye::default(),
        );
    }

    #[test]
    fn dont_flag_keep_det_noun_from() {
        assert_no_lints(
            "... but how can I keep this key from the naked eyes?",
            NakedEye::default(),
        );
    }

    #[test]
    fn just_from_the_naked_eye() {
        assert_suggestion_result(
            "What I want is, just from the naked eye, I think there are only 5 colors on this picture",
            NakedEye::default(),
            "What I want is, just with the naked eye, I think there are only 5 colors on this picture",
        );
    }

    #[test]
    fn invisibility_from_the_naked_eye() {
        assert_suggestion_result(
            "Considering the real security implications of true invisibility from the naked eye",
            NakedEye::default(),
            "Considering the real security implications of true invisibility to the naked eye",
        );
    }
}
