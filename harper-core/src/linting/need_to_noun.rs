use crate::Token;
use crate::char_string::char_string;
use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::WordSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct NeedToNoun {
    expr: Box<dyn Expr>,
}

impl Default for NeedToNoun {
    fn default() -> Self {
        let postfix_exceptions = LongestMatchOf::new(vec![
            Box::new(|tok: &Token, _: &[char]| tok.kind.is_adverb() || tok.kind.is_determiner()),
            Box::new(WordSet::new(&["about"])),
        ]);

        let a = SequenceExpr::aco("need")
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then(|tok: &Token, _: &[char]| tok.kind.is_nominal())
            .t_ws()
            .then_unless(postfix_exceptions);

        let b = SequenceExpr::aco("need")
            .t_ws()
            .t_aco("to")
            .t_ws()
            .then(|tok: &Token, _: &[char]| tok.kind.is_nominal() && !tok.kind.is_verb());

        Self {
            expr: Box::new(a.or(b)),
        }
    }
}

impl ExprLinter for NeedToNoun {
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
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

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
    fn flags_need_to_compiler() {
        assert_suggestion_result(
            "We simply don't need to compiler to do as much work anymore.",
            NeedToNoun::default(),
            "We simply don't need the compiler to do as much work anymore.",
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
}
