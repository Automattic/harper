use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::find_the_only_token_matching;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::{CharStringExt, Token, TokenKind};

pub struct ThoughThought {
    expr: Box<dyn Expr>,
}

impl Default for ThoughThought {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .then_kind_is_but_is_not(
                        TokenKind::is_subject_pronoun,
                        TokenKind::is_object_pronoun,
                    )
                    .t_ws()
                    .t_aco("though")
                    .t_ws()
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_subject_pronoun()),
                        Box::new(SequenceExpr::aco("that")),
                    ]),
            ),
        }
    }
}

impl ExprLinter for ThoughThought {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = find_the_only_token_matching(toks, src, |tok, src| {
            tok.span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['t', 'h', 'o', 'u', 'g', 'h'])
        })?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "thought",
                tok.span.get_content(src),
            )],
            message: "Is this a typo for `thought`?".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `though` when it's a typo for `thought`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        ThoughThought,
        tests::{assert_no_lints, assert_suggestion_result},
    };

    #[test]
    fn fix_i_though_i() {
        assert_suggestion_result(
            "Looking at those I though I had to draw imgui into separate renderpass",
            ThoughThought::default(),
            "Looking at those I thought I had to draw imgui into separate renderpass",
        );
    }

    #[test]
    fn fix_i_though_it() {
        assert_suggestion_result(
            "and I though it was a shame because the data it provides can be ...",
            ThoughThought::default(),
            "and I thought it was a shame because the data it provides can be ...",
        );
    }

    #[test]
    fn fix_i_though_that() {
        assert_suggestion_result(
            "I though that there may be other solutions as in other here",
            ThoughThought::default(),
            "I thought that there may be other solutions as in other here",
        );
    }

    #[test]
    fn fix_i_though_they() {
        assert_suggestion_result(
            "path parsin error ( i though they were extincted )",
            ThoughThought::default(),
            "path parsin error ( i thought they were extincted )",
        );
    }

    #[test]
    fn fix_i_though_we() {
        assert_suggestion_result(
            "and that way I though we need something universial",
            ThoughThought::default(),
            "and that way I thought we need something universial",
        );
    }

    #[test]
    fn fix_i_though_you() {
        assert_suggestion_result(
            "I though you resolved the issue, so I updated my version",
            ThoughThought::default(),
            "I thought you resolved the issue, so I updated my version",
        );
    }

    #[test]
    fn dont_flag_it_though_i() {
        assert_no_lints(
            "am including it though i believe it's nto the case because before this",
            ThoughThought::default(),
        );
    }

    #[test]
    fn dont_flag_it_though_it() {
        assert_no_lints(
            "Prisma works with it though it is not officially supported by Prisma yet.",
            ThoughThought::default(),
        );
    }

    #[test]
    #[ignore = "TODO: Can't check because `it` is both subject and object"]
    fn fix_you_though_it() {
        assert_suggestion_result(
            "it may reveal that the bug is not where you though it was",
            ThoughThought::default(),
            "it may reveal that the bug is not where you thought it was",
        );
    }

    #[test]
    fn dont_flag_you_though_that_1() {
        // Ambiguous: "I can tell you, though, that a project..." vs "I can tell (that) you thought that a project..."
        assert_no_lints(
            "I can tell you though that a project not using headers at all will likely be compiling much faster.",
            ThoughThought::default(),
        );
    }

    #[test]
    fn dont_flag_you_though_that_2() {
        assert_no_lints(
            "I agree with you though that 2D lat/lon grids are unnecessarily confusing",
            ThoughThought::default(),
        );
    }
}
