use crate::expr::{Expr, SequenceExpr};
use crate::patterns::WordSet;
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct SomethingIs {
    expr: Box<dyn Expr>,
}

impl Default for SomethingIs {
    fn default() -> Self {
        let forms = WordSet::new(&["somethings", "anythings", "everythings", "nothings"]);

        let expr = SequenceExpr::with(forms)
            .t_ws()
            .then_optional(SequenceExpr::default().then_one_or_more_adverbs().t_ws())
            .then_kind_any(&[TokenKind::is_verb_progressive_form]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for SomethingIs {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offender = matched_tokens.first()?;
        let original = offender.span.get_content(source);
        let stem_len = original.len().checked_sub(1)?;
        let stem = original[..stem_len].to_vec();

        let mut contraction = stem.clone();
        contraction.extend(['\'', 's']);

        let mut expanded = stem;
        expanded.push(' ');
        expanded.extend(['i', 's']);

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case(contraction, original),
                Suggestion::replace_with_match_case(expanded, original),
            ],
            message: "Prefer the contraction or full `is` rather than pluralizing this pronoun."
                .into(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Flags forms like `somethings` before progressive verbs and suggests using `something's` or `something is`."
    }
}

#[cfg(test)]
mod tests {
    use super::SomethingIs;
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn fixes_somethings_going() {
        assert_suggestion_result(
            "Somethings going well today.",
            SomethingIs::default(),
            "Something's going well today.",
        );
    }

    #[test]
    fn fixes_anythings_happening() {
        assert_suggestion_result(
            "Anythings happening tonight?",
            SomethingIs::default(),
            "Anything's happening tonight?",
        );
    }

    #[test]
    fn fixes_everythings_working() {
        assert_suggestion_result(
            "Everythings working smoothly.",
            SomethingIs::default(),
            "Everything's working smoothly.",
        );
    }

    #[test]
    fn fixes_nothings_changing() {
        assert_suggestion_result(
            "Nothings changing around here.",
            SomethingIs::default(),
            "Nothing's changing around here.",
        );
    }

    #[test]
    fn fixes_with_adverb() {
        assert_suggestion_result(
            "Somethings really happening now.",
            SomethingIs::default(),
            "Something's really happening now.",
        );
    }

    #[test]
    fn fixes_uppercase() {
        assert_suggestion_result(
            "SOMETHINGS HAPPENING NOW!",
            SomethingIs::default(),
            "SOMETHING'S HAPPENING NOW!",
        );
    }

    #[test]
    fn offers_is_expansion() {
        assert_nth_suggestion_result(
            "Somethings going wrong.",
            SomethingIs::default(),
            "Something is going wrong.",
            1,
        );
    }

    #[test]
    fn no_lint_when_contracted() {
        assert_no_lints("Something's going well today.", SomethingIs::default());
    }

    #[test]
    fn no_lint_when_plural_noun() {
        assert_lint_count(
            "Somethings in the attic kept us awake.",
            SomethingIs::default(),
            0,
        );
    }

    #[test]
    fn no_lint_at_sentence_end() {
        assert_no_lints("Somethings.", SomethingIs::default());
    }
}
