use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ShowCase {
    expr: All,
}

impl Default for ShowCase {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("show").t_ws_h().t_aco("case").but_not(
                SequenceExpr::anything().t_any().t_any().then_any_of([
                    Box::new(SequenceExpr::with(|t: &Token, _: &[char]| {
                        t.kind.is_hyphen()
                    })),
                    Box::new(SequenceExpr::whitespace().t_set([
                        "folded",
                        "folding",
                        "history",
                        "histories",
                        "number",
                        "numbers",
                        "sensitive",
                        "insensitive",
                        "study",
                        "studies",
                    ])),
                ]),
            ),
        }
    }
}

impl ExprLinter for ShowCase {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "showcase",
            span.get_content(source),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous, // as used in `closed_compounds.rs`
            suggestions,
            message: "Did you mean `showcase` (highlight the best qualities of something)?"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `show case` to `showcase`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ShowCase;

    #[test]
    #[ignore = "Fails due to `replace_with_match_case_str` copying case based on character index"]
    fn fix_show_case_noun_title_case() {
        assert_suggestion_result(
            "Progressive Web Apps (PWA) Show Case.",
            ShowCase::default(),
            "Progressive Web Apps (PWA) Showcase.",
        );
    }

    #[test]
    fn fix_show_case_noun_lowercase() {
        assert_suggestion_result(
            "Hello,How to run the show case in release mode.",
            ShowCase::default(),
            "Hello,How to run the showcase in release mode.",
        );
    }

    #[test]
    fn dont_flag_case_insensitive() {
        assert_no_lints(
            "Show case insensitive results when searching for a branch.",
            ShowCase::default(),
        )
    }

    #[test]
    fn dont_flag_case_sensitive() {
        assert_no_lints(
            "starship shows case-sensitive behavior on case-insensitive file system (macOS)",
            ShowCase::default(),
        )
    }

    #[test]
    fn fix_verb() {
        assert_suggestion_result(
            "Some stories don't show case how the component should be used.",
            ShowCase::default(),
            "Some stories don't showcase how the component should be used.",
        );
    }

    #[test]
    fn fix_verb_hyphenated() {
        assert_suggestion_result(
            "It's pretty simple to show-case a feature in your App by using BubbleShowCase framework.",
            ShowCase::default(),
            "It's pretty simple to showcase a feature in your App by using BubbleShowCase framework.",
        )
    }
}
