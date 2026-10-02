use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct ComfortableWith {
    expr: All,
}

impl Default for ComfortableWith {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["comfortable", "of"]).but_not(
                SequenceExpr::anything()
                    .t_any()
                    .t_any()
                    .t_ws()
                    .t_set(["all", "course", "those"]),
            ),
        }
    }
}

impl ExprLinter for ComfortableWith {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if preceded_by_word(context, |t| t.get_ch(source).eq_str("most")) {
            return None;
        }
        let span = matched_tokens.get(2)?.span;
        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "with",
                span.get_content(source),
            )],
            message: "The standard preposition after `comfortable` is `with`".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ComfortableWith;

    #[test]
    fn fix_am_comfortable_of_doing() {
        assert_suggestion_result(
            "It is not something that I am comfortable of doing at the moment until more work is done on the MediaWiki parser migration to Parsoid.",
            ComfortableWith::default(),
            "It is not something that I am comfortable with doing at the moment until more work is done on the MediaWiki parser migration to Parsoid.",
        );
    }

    #[test]
    fn fix_im_comfortable_of_declaring() {
        assert_suggestion_result(
            "identify any bugs and pain points before I'm comfortable of declaring this ready for production use",
            ComfortableWith::default(),
            "identify any bugs and pain points before I'm comfortable with declaring this ready for production use",
        );
    }

    #[test]
    fn fix_really_comfortable_of_converting() {
        assert_suggestion_result(
            "So if you are very comfortable in being in that messy middle and really comfortable of converting chaos into clarity for you and those around you",
            ComfortableWith::default(),
            "So if you are very comfortable in being in that messy middle and really comfortable with converting chaos into clarity for you and those around you",
        );
    }

    #[test]
    fn fix_not_comfortable_of_compiling() {
        assert_suggestion_result(
            "Most people are not comfortable of compiling their own binary, or are technically capable of doing it.",
            ComfortableWith::default(),
            "Most people are not comfortable with compiling their own binary, or are technically capable of doing it.",
        );
    }

    #[test]
    fn dont_flag_comfortable_of_course() {
        assert_no_lints(
            "Folks are more then welcome to discuss Carbon wherever they feel most comfortable of course, but I think we can't really expand the official chat spaces",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn dont_flag_comfortable_comma_of_course() {
        assert_no_lints(
            "Folks are more then welcome to discuss Carbon wherever they feel most comfortable of course, but I think we can't really expand the official chat spaces",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn fix_are_comfortable_of_the() {
        assert_suggestion_result(
            "While we are comfortable of the core design, there is still plenty of activity being done and refinements made",
            ComfortableWith::default(),
            "While we are comfortable with the core design, there is still plenty of activity being done and refinements made",
        );
    }

    #[test]
    fn dont_flag_most_comfortable_of_the_three() {
        assert_no_lints(
            "It is the most comfortable of the three to sit in front of for a long stretch.",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn dont_flag_are_comfortable_of_the_3() {
        assert_no_lints(
            "at 0.8 convergence there is the least amount of red hues and usually that means this is the most comfortable of the 3",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn dont_flag_the_most_comfortable_of_all() {
        assert_no_lints(
            "The seven league boots were, Mr Giant admits, the most comfortable of all the magic boots he's worn.",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn dont_flag_the_least_comfortable_of_those() {
        assert_no_lints(
            "I have big hands, so 1-3-5 feels the least comfortable of those, although it may be the 'automatic' choice.",
            ComfortableWith::default(),
        );
    }

    #[test]
    fn dont_flag_the_most_comfortable_of_those() {
        assert_no_lints(
            "On the other hand, TiDB is the most comfortable of those",
            ComfortableWith::default(),
        );
    }
}
