use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
};

pub struct NoHarmNoFoul {
    expr: All,
}

impl Default for NoHarmNoFoul {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["no", "nor"])
                .t_ws()
                .t_aco("harm")
                .then_optional_comma()
                .t_ws()
                .t_set(&["no", "nor"])
                .t_ws()
                .t_set(&["fowl", "foul"])
                .but_not(
                    SequenceExpr::word_seq(&["no", "harm"])
                        .then_optional_comma()
                        .t_ws()
                        .then_word_seq(&["no", "foul"]),
                ),
        }
    }
}

impl ExprLinter for NoHarmNoFoul {
    type Unit = Sentence;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let mid = &toks[3..toks.len() - 3];

        let correction = "no harm"
            .chars()
            .chain(mid.get_str(src)?.chars())
            .chain("no foul".chars())
            .collect::<Vec<char>>();

        let uses_nor = [0, toks.len() - 3].iter().any(|&i| {
            toks[i]
                .get_ch(src)
                .ends_with_ignore_ascii_case_chars(&['r'])
        });
        let uses_fowl = toks[toks.len() - 1]
            .get_ch(src)
            .eq_ch(&['f', 'o', 'w', 'l']);

        let message = match (uses_nor, uses_fowl) {
            (true, false) => "This idiom uses the word `no` rather than `not`.".to_owned(),
            (true, true) => "Use the words `no` and `foul` in this idiom.".to_owned(),
            (false, true) => {
                "`Fowl` means a bird. This idiom uses `foul`, as in baseball.".to_owned()
            }
            _ => return None,
        };

        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Nonstandard,
            suggestions: vec![Suggestion::replace_with_match_case(
                correction,
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
        "Corrects nonstandard variants of the idiom `no harm, no foul`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::NoHarmNoFoul;

    // We don't enforce the comma, so ensure both legit variants are not flagged.

    #[test]
    fn dont_flag_standard_without_comma() {
        assert_no_lints("no harm no foul", NoHarmNoFoul::default());
    }

    #[test]
    fn dont_flag_standard_with_comma() {
        assert_no_lints("no harm, no foul", NoHarmNoFoul::default());
    }

    // Ensure all error variants are flagged.

    // no harm no fowl
    #[test]
    fn flag_no_harm_no_fowl() {
        assert_suggestion_result(
            "no harm no fowl",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_no_harm_comma_no_fowl() {
        assert_suggestion_result(
            "no harm, no fowl",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // no harm nor foul
    #[test]
    fn flag_no_harm_nor_foul() {
        assert_suggestion_result(
            "no harm nor foul",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_no_harm_comma_nor_foul() {
        assert_suggestion_result(
            "no harm, nor foul",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // no harm nor fowl
    #[test]
    fn flag_no_harm_nor_fowl() {
        assert_suggestion_result(
            "no harm nor fowl",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_no_harm_comma_nor_fowl() {
        assert_suggestion_result(
            "no harm, nor fowl",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // nor harm no foul
    #[test]
    fn flag_nor_harm_no_foul() {
        assert_suggestion_result(
            "nor harm no foul",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_nor_harm_comma_no_foul() {
        assert_suggestion_result(
            "nor harm, no foul",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // nor harm no fowl
    #[test]
    fn flag_nor_harm_no_fowl() {
        assert_suggestion_result(
            "nor harm no fowl",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_nor_harm_comma_no_fowl() {
        assert_suggestion_result(
            "nor harm, no fowl",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // nor harm nor foul
    #[test]
    fn flag_nor_harm_nor_foul() {
        assert_suggestion_result(
            "nor harm nor foul",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_nor_harm_comma_nor_foul() {
        assert_suggestion_result(
            "nor harm, nor foul",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // nor harm nor fowl
    #[test]
    fn flag_nor_harm_nor_fowl() {
        assert_suggestion_result(
            "nor harm nor fowl",
            NoHarmNoFoul::default(),
            "no harm no foul",
        );
    }
    #[test]
    fn flag_nor_harm_comma_nor_fowl() {
        assert_suggestion_result(
            "nor harm, nor fowl",
            NoHarmNoFoul::default(),
            "no harm, no foul",
        );
    }

    // Real-world tests

    #[test]
    fn fix_no_harm_no_fowl_no_comma() {
        assert_suggestion_result(
            "But no harm no fowl either way I now have proof that the experience of new users is not worth including a single word in a dependency list.",
            NoHarmNoFoul::default(),
            "But no harm no foul either way I now have proof that the experience of new users is not worth including a single word in a dependency list.",
        );
    }

    #[test]
    fn fix_no_harm_no_fowl_with_comma() {
        assert_suggestion_result(
            "I expect there will be no implementations that do otherwise, so no harm, no fowl, ugly corner cases go away....",
            NoHarmNoFoul::default(),
            "I expect there will be no implementations that do otherwise, so no harm, no foul, ugly corner cases go away....",
        );
    }

    #[test]
    fn fix_no_harm_nor_foul_no_comma() {
        assert_suggestion_result(
            "and, yes, open source is a labor of love primarily so no harm nor foul when the goals take forever",
            NoHarmNoFoul::default(),
            "and, yes, open source is a labor of love primarily so no harm no foul when the goals take forever",
        );
    }

    #[test]
    fn fix_no_harm_nor_fowl_no_comma() {
        assert_suggestion_result(
            "Thankfully my car bounced off and landed wheels down so no harm nor fowl",
            NoHarmNoFoul::default(),
            "Thankfully my car bounced off and landed wheels down so no harm no foul",
        );
    }

    #[test]
    fn fix_nor_harm_no_foul_with_comma() {
        assert_suggestion_result(
            "This happened today sorry not sorry to those that were scared \"scaleless\" it was too funny nor harm no foul",
            NoHarmNoFoul::default(),
            "This happened today sorry not sorry to those that were scared \"scaleless\" it was too funny no harm no foul",
        );
    }

    #[test]
    fn fix_nor_harm_no_fowl_with_comma() {
        assert_suggestion_result(
            "She was mad. But I say nor harm, no fowl.",
            NoHarmNoFoul::default(),
            "She was mad. But I say no harm, no foul.",
        );
    }

    #[test]
    fn fix_nor_harm_nor_foul() {
        assert_suggestion_result(
            "As long as the item was unharmed nor harm nor foul.",
            NoHarmNoFoul::default(),
            "As long as the item was unharmed no harm no foul.",
        );
    }

    // nor harm nor fowl: no examples found on the Internet
}
