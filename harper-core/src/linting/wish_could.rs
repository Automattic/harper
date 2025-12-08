use super::{Lint, LintKind, Suggestion};
use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, expr_linter::Chunk};

pub struct WishCould {
    expr: Box<dyn Expr>,
}

impl Default for WishCould {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["wish", "wished", "wishes", "wishing"])
                    .t_ws()
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_subject_pronoun()),
                        Box::new(SequenceExpr::word_set(&[
                            // Elective existential indefinite pronouns
                            "anybody",
                            "anyone",
                            // Universal indefinite pronouns
                            "everybody",
                            "everyone",
                            // Negative indefinite pronouns (correct)
                            "nobody",
                            // Negative indefinite pronouns (incorrect)
                            "noone",
                            // Assertive existential indefinite pronouns
                            "somebody",
                            "someone",
                            // Demonstrative pronouns
                            "these",
                            "this",
                            "those",
                        ])),
                    ])
                    .t_ws()
                    .t_aco("can"),
            ),
        }
    }
}

impl ExprLinter for WishCould {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &*self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let can_tok = toks.last()?;
        let can_span = can_tok.span;

        Some(Lint {
            span: can_span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "could",
                can_span.get_content(src),
            )],
            message: "Use 'could' instead of 'can' after 'wish'.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for `can` being used after `wish` when it should be `could`."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn flag_wish_we_can() {
        assert_suggestion_result(
            "i wish we can spend more time together",
            WishCould::default(),
            "i wish we could spend more time together",
        );
    }

    #[test]
    fn flag_wish_i_can() {
        assert_suggestion_result(
            "I wish I can finally forgive myself for all the things I am not.",
            WishCould::default(),
            "I wish I could finally forgive myself for all the things I am not.",
        );
    }

    #[test]
    fn flag_wish_you_can() {
        assert_suggestion_result(
            "I wish you can find your true love.",
            WishCould::default(),
            "I wish you could find your true love.",
        );
    }

    #[test]
    fn flag_wishes_they_can() {
        assert_suggestion_result(
            "What your Therapist wishes they can tell you.",
            WishCould::default(),
            "What your Therapist wishes they could tell you.",
        );
    }

    #[test]
    fn flag_wishing_someone_can() {
        assert_suggestion_result(
            "Forever wishing someone can point me in the right direction",
            WishCould::default(),
            "Forever wishing someone could point me in the right direction",
        );
    }

    #[test]
    fn flag_wish_they_can() {
        assert_suggestion_result(
            "I wish they can plant more trees on this road.",
            WishCould::default(),
            "I wish they could plant more trees on this road.",
        );
    }

    #[test]
    fn flag_wished_he_can() {
        assert_suggestion_result(
            "I just wished he can talk and tell me how he feels",
            WishCould::default(),
            "I just wished he could talk and tell me how he feels",
        );
    }

    #[test]
    fn wish_this_can() {
        assert_suggestion_result(
            "but I wish this can be fixed by Electron team",
            WishCould::default(),
            "but I wish this could be fixed by Electron team",
        )
    }

    #[test]
    fn wish_it_can() {
        assert_suggestion_result(
            "Wish it can be supported.",
            WishCould::default(),
            "Wish it could be supported.",
        )
    }

    #[test]
    fn wish_somebody_can() {
        assert_suggestion_result(
            "I wish somebody can fix this issue.",
            WishCould::default(),
            "I wish somebody could fix this issue.",
        )
    }
}
