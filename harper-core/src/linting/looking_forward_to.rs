use hashbrown::HashSet;

use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct LookingForwardTo {
    expr: Box<dyn Expr>,
}

impl Default for LookingForwardTo {
    fn default() -> Self {
        let looking_forward_to = FixedPhrase::from_phrase("looking forward to");

        let pattern = SequenceExpr::with(looking_forward_to)
            .t_ws()
            // TODO: update the use the verb with progressive tense function later
            .then_verb();

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for LookingForwardTo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], src: &[char]) -> Option<Lint> {
        let span = matched_tokens.last()?.span;
        let verb = matched_tokens.last()?.span.get_content_string(src);
        if verb.ends_with("ing") {
            return None;
        }

        // TODO: create a util function to handle the appending of -ing
        // to verbs, taking into account exceptions and irregular forms.
        let exception_word: HashSet<&str> = [
            // Verbs ending in -ee
            "see",
            "flee",
            "agree",
            "knee",
            "guarantee",
            // Verbs ending in -oe
            "hoe",
            "toe",
            // Verbs ending in -ye or to avoid confusion
            "dye",
            "eye",
            // Irregular/spelling clarification
            "singe",
            "tinge",
        ]
        .iter()
        .cloned()
        .collect();

        let gerund_form: String =
            if verb.to_lowercase().ends_with('e') && !exception_word.contains(verb.as_str()) {
                verb.trim_end_matches('e').to_string() + "ing"
            } else {
                format!("{verb}ing")
            };

        println!("gerund_form: -{gerund_form}- -- verb: -{verb}-");
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            message: format!(
                "The verb `{verb}` must be in the gerund form (verb + -ing) after 'looking forward to'.",
            ),
            suggestions: vec![Suggestion::replace_with_match_case(
                gerund_form.chars().collect(),
                span.get_content(src),
            )],
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "This rule identifies instances where the phrase `looking forward to` is followed by a base form verb instead of the required gerund (verb + `-ing` form)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::LookingForwardTo;

    #[test]
    fn not_lint_with_correct_verb() {
        assert_suggestion_result(
            "She was looking forward to see the grandchildren again.",
            LookingForwardTo::default(),
            "She was looking forward to seeing the grandchildren again.",
        );
        // assert_lint_count(
        //     "She was looking forward to seeing the grandchildren again.",
        //     LookingForwardTo::default(),
        //     0,
        // );
    }

    #[test]
    fn lint_with_incorrect_verb() {
        assert_suggestion_result(
            "She was looking forward to see the grandchildren again.",
            LookingForwardTo::default(),
            "She was looking forward to seeing the grandchildren again.",
        );
    }

    #[test]
    fn lint_with_incorrect_verb_ending_in_e() {
        assert_suggestion_result(
            "She was looking forward to make the grandchildren happy.",
            LookingForwardTo::default(),
            "She was looking forward to making the grandchildren happy.",
        );
    }

    #[test]
    fn not_lint_with_non_verb() {
        assert_lint_count(
            "She was looking forward to the grandchildren's visit.",
            LookingForwardTo::default(),
            0,
        );
    }
}
