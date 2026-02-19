use crate::{
    Token,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::expr_linter::Chunk,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::DerivedFrom,
};

pub struct TakeMedicine {
    expr: Box<dyn Expr>,
}

impl Default for TakeMedicine {
    fn default() -> Self {
        let eat_verb = DerivedFrom::new_from_str("eat")
            .or(DerivedFrom::new_from_str("eats"))
            .or(DerivedFrom::new_from_str("ate"))
            .or(DerivedFrom::new_from_str("eating"))
            .or(DerivedFrom::new_from_str("eaten"));

        let medication = DerivedFrom::new_from_str("antibiotic")
            .or(DerivedFrom::new_from_str("medicine"))
            .or(DerivedFrom::new_from_str("medication"))
            .or(DerivedFrom::new_from_str("pill"))
            .or(DerivedFrom::new_from_str("tablet"))
            .or(DerivedFrom::new_from_str("aspirin"))
            .or(DerivedFrom::new_from_str("paracetamol"));

        let modifiers = SequenceExpr::any_of(vec![
            Box::new(SequenceExpr::default().then_determiner()),
            Box::new(SequenceExpr::default().then_possessive_determiner()),
            Box::new(SequenceExpr::default().then_quantifier()),
        ])
        .t_ws();

        let adjectives = SequenceExpr::default().then_one_or_more_adjectives().t_ws();

        let pattern = SequenceExpr::with(eat_verb)
            .t_ws()
            .then_optional(modifiers)
            .then_optional(adjectives)
            .then(medication);

        Self {
            expr: Box::new(pattern),
        }
    }
}

fn replacement_for(
    verb: &Token,
    source: &[char],
    base: &str,
    third_person: &str,
    past: &str,
    past_participle: &str,
    progressive: &str,
) -> Suggestion {
    let replacement = if verb.kind.is_verb_progressive_form() {
        progressive
    } else if verb.kind.is_verb_third_person_singular_present_form() {
        third_person
    } else if verb.kind.is_verb_past_participle_form() && !verb.kind.is_verb_simple_past_form() {
        past_participle
    } else if verb.kind.is_verb_simple_past_form() {
        past
    } else {
        base
    };

    Suggestion::replace_with_match_case(
        replacement.chars().collect(),
        verb.span.get_content(source),
    )
}

impl ExprLinter for TakeMedicine {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let verb = matched_tokens.first()?;
        let span = verb.span;

        let suggestions = vec![
            replacement_for(verb, source, "take", "takes", "took", "taken", "taking"),
            replacement_for(
                verb,
                source,
                "swallow",
                "swallows",
                "swallowed",
                "swallowed",
                "swallowing",
            ),
        ];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Use a verb like `take` or `swallow` with medicine instead of `eat`."
                .to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Encourages pairing medicine-related nouns with verbs like `take` or `swallow` instead of `eat`."
    }
}

#[cfg(test)]
mod tests {
    use super::TakeMedicine;
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn swaps_ate_antibiotics() {
        assert_suggestion_result(
            "I ate antibiotics for a week.",
            TakeMedicine::default(),
            "I took antibiotics for a week.",
        );
    }

    #[test]
    fn swaps_eat_medicine() {
        assert_suggestion_result(
            "You should eat the medicine now.",
            TakeMedicine::default(),
            "You should take the medicine now.",
        );
    }

    #[test]
    fn swaps_eats_medication() {
        assert_suggestion_result(
            "She eats medication daily.",
            TakeMedicine::default(),
            "She takes medication daily.",
        );
    }

    #[test]
    fn swaps_eating_medicines() {
        assert_suggestion_result(
            "Are you eating medicines for that illness?",
            TakeMedicine::default(),
            "Are you taking medicines for that illness?",
        );
    }

    #[test]
    fn swaps_eaten_medication() {
        assert_suggestion_result(
            "He has eaten medication already.",
            TakeMedicine::default(),
            "He has taken medication already.",
        );
    }

    #[test]
    fn swaps_eat_pills() {
        assert_suggestion_result(
            "He ate the pills without water.",
            TakeMedicine::default(),
            "He took the pills without water.",
        );
    }

    #[test]
    fn swaps_eating_paracetamol() {
        assert_suggestion_result(
            "She is eating paracetamol for her headache.",
            TakeMedicine::default(),
            "She is taking paracetamol for her headache.",
        );
    }

    #[test]
    fn handles_possessive_modifier() {
        assert_suggestion_result(
            "Please eat my antibiotics.",
            TakeMedicine::default(),
            "Please take my antibiotics.",
        );
    }

    #[test]
    fn handles_adjectives() {
        assert_suggestion_result(
            "They ate the prescribed antibiotics.",
            TakeMedicine::default(),
            "They took the prescribed antibiotics.",
        );
    }

    #[test]
    fn supports_uppercase() {
        assert_suggestion_result(
            "Eat antibiotics with water.",
            TakeMedicine::default(),
            "Take antibiotics with water.",
        );
    }

    #[test]
    fn offers_swallow_alternative() {
        assert_nth_suggestion_result(
            "He ate the medication without water.",
            TakeMedicine::default(),
            "He swallowed the medication without water.",
            1,
        );
    }

    #[test]
    fn ignores_correct_usage() {
        assert_lint_count(
            "She took antibiotics last winter.",
            TakeMedicine::default(),
            0,
        );
    }

    #[test]
    fn ignores_unrelated_eat() {
        assert_lint_count(
            "They ate dinner after taking medicine.",
            TakeMedicine::default(),
            0,
        );
    }
}
