use std::sync::Arc;

use crate::{
    Token,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct BeAllowed {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for BeAllowed {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::default()
                .t_aco("will")
                .t_ws()
                .then_word_set(&["not"])
                .t_ws()
                .t_aco("allowed")
                .t_ws()
                .t_aco("to")
                .t_ws()
                .then_verb(),
            4,
        );

        map.insert(
            SequenceExpr::default()
                .t_aco("won't")
                .t_ws()
                .t_aco("allowed")
                .t_ws()
                .t_aco("to")
                .t_ws()
                .then_verb(),
            2,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for BeAllowed {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let allowed_index = *self.map.lookup(0, matched_tokens, source)?;
        let allowed_token = matched_tokens.get(allowed_index)?;
        let span = allowed_token.span;
        let template = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                "be allowed".chars().collect(),
                template,
            )],
            message: "Add `be` so this reads `be allowed`.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures the passive form uses `be allowed` after future negatives."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::BeAllowed;

    #[test]
    fn corrects_basic_sentence() {
        assert_suggestion_result(
            "You will not allowed to enter the lab.",
            BeAllowed::default(),
            "You will not be allowed to enter the lab.",
        );
    }

    #[test]
    fn corrects_first_person_subject() {
        assert_suggestion_result(
            "I will not allowed to go tonight.",
            BeAllowed::default(),
            "I will not be allowed to go tonight.",
        );
    }

    #[test]
    fn corrects_plural_subject() {
        assert_suggestion_result(
            "Students will not allowed to submit late work.",
            BeAllowed::default(),
            "Students will not be allowed to submit late work.",
        );
    }

    #[test]
    fn corrects_with_intro_clause() {
        assert_suggestion_result(
            "Because of policy, workers will not allowed to take photos.",
            BeAllowed::default(),
            "Because of policy, workers will not be allowed to take photos.",
        );
    }

    #[test]
    fn corrects_contracted_form() {
        assert_suggestion_result(
            "They won't allowed to park here during events.",
            BeAllowed::default(),
            "They won't be allowed to park here during events.",
        );
    }

    #[test]
    fn corrects_all_caps() {
        assert_suggestion_result(
            "THEY WILL NOT ALLOWED TO ENTER.",
            BeAllowed::default(),
            "THEY WILL NOT BE ALLOWED TO ENTER.",
        );
    }

    #[test]
    fn corrects_with_trailing_clause() {
        assert_suggestion_result(
            "Without a permit, guests will not allowed to stay overnight at the cabin.",
            BeAllowed::default(),
            "Without a permit, guests will not be allowed to stay overnight at the cabin.",
        );
    }

    #[test]
    fn corrects_with_modal_context() {
        assert_suggestion_result(
            "Even with approval, contractors will not allowed to access production.",
            BeAllowed::default(),
            "Even with approval, contractors will not be allowed to access production.",
        );
    }

    #[test]
    fn leaves_correct_phrase_untouched() {
        assert_suggestion_result(
            "They will not be allowed to park here during events.",
            BeAllowed::default(),
            "They will not be allowed to park here during events.",
        );
    }

    #[test]
    fn leaves_other_verbs_alone() {
        assert_lint_count(
            "We will not allow visitors after nine.",
            BeAllowed::default(),
            0,
        );
    }

    #[test]
    fn leaves_similar_sequence_without_to() {
        assert_lint_count(
            "They won't be allowed to park here during events.",
            BeAllowed::default(),
            0,
        );
    }
}
