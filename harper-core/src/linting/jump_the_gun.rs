use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct JumpTheGun {
    expr: All,
}

impl Default for JumpTheGun {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["jump", "jumped", "jumping", "jumps"])
                .t_ws()
                .t_set(&["a", "an", "the"])
                .t_ws()
                .t_set(&["gun", "guns"])
                .and_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_aco("the")
                        .t_ws()
                        .t_aco("gun"),
                ),
        }
    }
}

impl ExprLinter for JumpTheGun {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let det_ws_gun_span = matched_tokens[2..].span()?;

        Some(Lint {
            span: det_ws_gun_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "the gun",
                det_ws_gun_span.get_content(source),
            )],
            message: "The correct idiom is `jump the gun`".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects incorrect usage of the `jump the gun` idiom."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::JumpTheGun;

    #[test]
    fn dont_flag_jump_the_gun() {
        assert_no_lints("jump the gun", JumpTheGun::default());
    }

    // Real-world tests

    #[test]
    fn fix_jump_a_gun() {
        assert_suggestion_result(
            "But in Race Max Pro, it's possible to get a happy finger case and jump a gun.",
            JumpTheGun::default(),
            "But in Race Max Pro, it's possible to get a happy finger case and jump the gun.",
        );
    }

    #[test]
    fn fix_jumped_a_gun() {
        assert_suggestion_result(
            "Well, I may have jumped a gun here creating this issue.",
            JumpTheGun::default(),
            "Well, I may have jumped the gun here creating this issue.",
        );
    }

    #[test]
    fn fix_jumped_the_guns() {
        assert_suggestion_result(
            "Lexxy being a js library I jumped the guns and felt removing the button completely on initialization was safer.",
            JumpTheGun::default(),
            "Lexxy being a js library I jumped the gun and felt removing the button completely on initialization was safer.",
        );
    }

    #[test]
    fn fix_jumping_the_guns() {
        assert_suggestion_result(
            "To make sure I'm not jumping the guns, @hkuadithya how did you change the JDK version exactly?",
            JumpTheGun::default(),
            "To make sure I'm not jumping the gun, @hkuadithya how did you change the JDK version exactly?",
        );
    }
}
