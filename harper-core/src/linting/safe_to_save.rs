use harper_brill::UPOS;

use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::patterns::{ModalVerb, UPOSSet, WordSet};
use crate::{
    Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct SafeToSave {
    expr: Box<dyn Expr>,
}

impl Default for SafeToSave {
    fn default() -> Self {
        let with_adv = SequenceExpr::with(ModalVerb::default())
            .then_whitespace()
            .then(UPOSSet::new(&[UPOS::ADV]))
            .then_whitespace()
            .t_aco("safe")
            .then_whitespace()
            .then_unless(WordSet::new(&["to"]));

        let without_adv = SequenceExpr::with(ModalVerb::default())
            .then_whitespace()
            .t_aco("safe")
            .then_whitespace()
            .then_unless(WordSet::new(&["to"]));

        let pattern = with_adv.or_longest(without_adv);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for SafeToSave {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let safe_idx = toks
            .iter()
            .position(|t| t.span.get_content_string(src).to_lowercase() == "safe")?;

        let safe_tok = &toks[safe_idx];

        Some(Lint {
            span: safe_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith("save".chars().collect())],
            message: "The word `safe` is an adjective. Did you mean the verb `save`?".to_string(),
            priority: 57,
        })
    }

    fn description(&self) -> &str {
        "Detects `safe` (adjective) when `save` (verb) is intended after modal verbs like `could` or `should`."
    }
}

#[cfg(test)]
mod tests {
    use super::SafeToSave;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_could_safe() {
        assert_suggestion_result(
            "He could safe my life.",
            SafeToSave::default(),
            "He could save my life.",
        );
    }

    #[test]
    fn corrects_should_safe() {
        assert_suggestion_result(
            "You should safe your work frequently.",
            SafeToSave::default(),
            "You should save your work frequently.",
        );
    }

    #[test]
    fn corrects_will_safe() {
        assert_suggestion_result(
            "This will safe you time.",
            SafeToSave::default(),
            "This will save you time.",
        );
    }

    #[test]
    fn corrects_would_safe() {
        assert_suggestion_result(
            "It would safe us money.",
            SafeToSave::default(),
            "It would save us money.",
        );
    }

    #[test]
    fn corrects_can_safe() {
        assert_suggestion_result(
            "You can safe the document now.",
            SafeToSave::default(),
            "You can save the document now.",
        );
    }

    #[test]
    fn corrects_might_safe() {
        assert_suggestion_result(
            "This might safe the company.",
            SafeToSave::default(),
            "This might save the company.",
        );
    }

    #[test]
    fn corrects_must_safe() {
        assert_suggestion_result(
            "We must safe our resources.",
            SafeToSave::default(),
            "We must save our resources.",
        );
    }

    #[test]
    fn corrects_may_safe() {
        assert_suggestion_result(
            "You may safe your progress here.",
            SafeToSave::default(),
            "You may save your progress here.",
        );
    }

    #[test]
    fn corrects_with_adverb() {
        assert_suggestion_result(
            "You should definitely safe your changes.",
            SafeToSave::default(),
            "You should definitely save your changes.",
        );
    }

    #[test]
    fn corrects_shall_safe() {
        assert_suggestion_result(
            "We shall safe the nation.",
            SafeToSave::default(),
            "We shall save the nation.",
        );
    }

    #[test]
    fn corrects_couldnt_safe() {
        assert_suggestion_result(
            "I couldn't safe the file.",
            SafeToSave::default(),
            "I couldn't save the file.",
        );
    }

    #[test]
    fn allows_safe_to_verb() {
        assert_no_lints("It is safe to assume.", SafeToSave::default());
    }

    #[test]
    fn allows_safe_noun() {
        assert_no_lints("Put the money in the safe today.", SafeToSave::default());
    }

    #[test]
    fn allows_correct_save() {
        assert_no_lints("You should save your work.", SafeToSave::default());
    }
}
