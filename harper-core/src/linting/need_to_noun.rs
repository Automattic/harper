use crate::Token;
use crate::char_string::char_string;
use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::expr::UnlessStep;
use crate::patterns::DerivedFrom;
use crate::patterns::WordSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct NeedToNoun {
    expr: Box<dyn Expr>,
}

impl Default for NeedToNoun {
    fn default() -> Self {
        let postfix_exceptions = LongestMatchOf::new(vec![
            Box::new(|tok: &Token, _: &[char]| {
                tok.kind.is_adverb()
                    || tok.kind.is_determiner()
                    || tok.kind.is_unlintable()
                    || tok.kind.is_pronoun()
            }),
            Box::new(WordSet::new(&["about", "into", "it"])),
        ]);

        let exceptions = SequenceExpr::anything()
            .t_any()
            .t_any()
            .t_any()
            .then_word_set(&["be", "match"]);

        let a = SequenceExpr::default()
            .then_kind_where(|kind| kind.is_nominal())
            .t_ws()
            .then_unless(postfix_exceptions);

        let b =
            SequenceExpr::default().then_kind_where(|kind| kind.is_nominal() && !kind.is_verb());

        let expr = SequenceExpr::with(DerivedFrom::new_from_str("need"))
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then(a.or(b));

        Self {
            expr: Box::new(expr.and(UnlessStep::new(exceptions, |_: &Token, _: &[char]| true))),
        }
    }
}

impl ExprLinter for NeedToNoun {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let to_idx = 2;
        let to_token = &matched_tokens[to_idx];

        let noun_idx = 4;
        let noun_token = &matched_tokens[noun_idx];

        let noun_text = noun_token.span.get_content_string(source);
        let span = to_token.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith(char_string!("the").to_vec())],
            message: format!(
                "`need to` should be followed by a verb, not a noun or pronoun like `{noun_text}`."
            ),
            priority: 48,
        })
    }

    fn description(&self) -> &'static str {
        "Flags `need to` when it is immediately followed by a noun, which usually means the infinitive verb is missing."
    }
}

#[cfg(test)]
mod tests {
    use super::NeedToNoun;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn flags_need_to_noun() {
        assert_suggestion_result(
            "I need to information now.",
            NeedToNoun::default(),
            "I need the information now.",
        );
    }

    #[test]
    fn allows_need_to_verb() {
        assert_lint_count("I need to leave now.", NeedToNoun::default(), 0);
    }

    #[test]
    fn allows_need_to_finish() {
        assert_lint_count(
            "I need to finish this report by tomorrow.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_call() {
        assert_lint_count(
            "You need to call your mother tonight.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_talk() {
        assert_lint_count(
            "We need to talk about the budget.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_leave() {
        assert_lint_count(
            "They need to leave early to catch the train.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_practice() {
        assert_lint_count(
            "She needs to practice her German more often.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_fix() {
        assert_lint_count(
            "He needs to fix his bike before the weekend.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_decide() {
        assert_lint_count(
            "We need to decide where to go for dinner.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_update() {
        assert_lint_count(
            "You need to update your password.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_take() {
        assert_lint_count(
            "I need to take a break and get some fresh air.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_need_to_clean() {
        assert_lint_count(
            "They need to clean the house before guests arrive.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn avoids_false_positive_for_need_to_verify() {
        assert_lint_count(
            "I need to verify the expenses before submission.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn flags_need_to_compiler() {
        assert_suggestion_result(
            "We simply don't need to compiler to do as much work anymore.",
            NeedToNoun::default(),
            "We simply don't need the compiler to do as much work anymore.",
        );
    }

    #[test]
    fn catches_false_negative_for_need_to_credentials() {
        assert_suggestion_result(
            "I need to credentials before logging in.",
            NeedToNoun::default(),
            "I need the credentials before logging in.",
        );
    }

    #[test]
    fn flags_need_to_report() {
        assert_suggestion_result(
            "We need to report before the meeting starts.",
            NeedToNoun::default(),
            "We need the report before the meeting starts.",
        );
    }

    #[test]
    fn flags_need_to_password() {
        assert_suggestion_result(
            "You need to password to access the server.",
            NeedToNoun::default(),
            "You need the password to access the server.",
        );
    }

    #[test]
    fn flags_need_to_data() {
        assert_suggestion_result(
            "They need to data analyzed by tomorrow.",
            NeedToNoun::default(),
            "They need the data analyzed by tomorrow.",
        );
    }

    #[test]
    fn flags_need_to_approval() {
        assert_suggestion_result(
            "She will need to approval of her manager first.",
            NeedToNoun::default(),
            "She will need the approval of her manager first.",
        );
    }

    #[test]
    fn flags_need_to_backup() {
        assert_suggestion_result(
            "We might need to backup if the main system fails.",
            NeedToNoun::default(),
            "We might need the backup if the main system fails.",
        );
    }

    #[test]
    fn flags_need_to_permit() {
        assert_suggestion_result(
            "He didn’t realize he would need to permit to film there.",
            NeedToNoun::default(),
            "He didn’t realize he would need the permit to film there.",
        );
    }

    #[test]
    fn flags_need_to_tools() {
        assert_suggestion_result(
            "You’ll need to right tools to fix that.",
            NeedToNoun::default(),
            "You’ll need the right tools to fix that.",
        );
    }

    #[test]
    fn flags_need_to_context() {
        assert_suggestion_result(
            "We need to context to make sense of his decision.",
            NeedToNoun::default(),
            "We need the context to make sense of his decision.",
        );
    }

    #[test]
    fn flags_need_to_funds() {
        assert_suggestion_result(
            "They need to funds released before construction begins.",
            NeedToNoun::default(),
            "They need the funds released before construction begins.",
        );
    }

    #[test]
    fn flags_need_to_silence() {
        assert_suggestion_result(
            "I need to silence to think clearly.",
            NeedToNoun::default(),
            "I need the silence to think clearly.",
        );
    }

    #[test]
    fn flags_needs_to_approval() {
        assert_suggestion_result(
            "She needs to approval from her advisor.",
            NeedToNoun::default(),
            "She needs the approval from her advisor.",
        );
    }

    #[test]
    fn avoids_false_positive_for_needs_to_coordinate() {
        assert_lint_count(
            "She needs to collaborate with everyone on the plan.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn catches_false_negative_for_needs_to_credentials() {
        assert_suggestion_result(
            "He needs to credentials ready before the audit.",
            NeedToNoun::default(),
            "He needs the credentials ready before the audit.",
        );
    }

    #[test]
    fn allows_needs_to_finalize() {
        assert_lint_count(
            "She needs to finalize the schedule.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn flags_needed_to_permit() {
        assert_suggestion_result(
            "They needed to permit before entering the site.",
            NeedToNoun::default(),
            "They needed the permit before entering the site.",
        );
    }

    #[test]
    fn avoids_false_positive_for_needed_to_explain() {
        assert_lint_count(
            "They needed to explain the new policy carefully.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn catches_false_negative_for_needed_to_authorization() {
        assert_suggestion_result(
            "They needed to authorization before proceeding.",
            NeedToNoun::default(),
            "They needed the authorization before proceeding.",
        );
    }

    #[test]
    fn allows_needed_to_file() {
        assert_lint_count(
            "They needed to file the paperwork before noon.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn flags_needing_to_documentation() {
        assert_suggestion_result(
            "Needing to documentation slowed the entire process.",
            NeedToNoun::default(),
            "Needing the documentation slowed the entire process.",
        );
    }

    #[test]
    fn avoids_false_positive_for_needing_to_calibrate() {
        assert_lint_count(
            "Needing to calibrate the equipment delayed us slightly.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn catches_false_negative_for_needing_to_confirmation() {
        assert_suggestion_result(
            "Needing to confirmation from legal stalled the launch.",
            NeedToNoun::default(),
            "Needing the confirmation from legal stalled the launch.",
        );
    }

    #[test]
    fn allows_needing_to_call() {
        assert_lint_count(
            "Needing to call your mother is stressful.",
            NeedToNoun::default(),
            0,
        );
    }

    #[test]
    fn allows_issue_2252() {
        assert_no_lints("Things I need to do today:", NeedToNoun::default());
    }

    #[test]
    fn allows_install() {
        assert_no_lints(
            "You need to install it separately, as it's a standalone application.",
            NeedToNoun::default(),
        );
    }

    #[test]
    fn allows_lay() {
        assert_no_lints(
            "Okay, this is a long one, but I feel like I need to lay everything out.",
            NeedToNoun::default(),
        );
    }

    #[test]
    fn allows_overcome() {
        assert_no_lints(
            "We believe every family deserves the opportunity to flourish, and we are committed to providing the resources they need to overcome adversity.",
            NeedToNoun::default(),
        );
    }

    #[test]
    fn allows_need_to_run_into_2433() {
        assert_no_lints(
            "So that they don't need to run into this problem in the future.",
            NeedToNoun::default(),
        );
    }

    #[test]
    fn allows_need_to_match_2446() {
        assert_no_lints(
            "You don't need to match string errors explicitly.",
            NeedToNoun::default(),
        );
    }

    #[test]
    fn allows_need_to_match_exactly_2446() {
        assert_no_lints("They need to match exactly.", NeedToNoun::default());
    }
}
