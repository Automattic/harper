use crate::expr::{All, Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

/// Linter for detecting common wrong preposition patterns.
///
/// Uses pattern-based detection to identify unambiguous wrong prepositions
/// and suggest correct alternatives. Each pattern is carefully chosen to
/// minimize false positives.
pub struct WrongPreposition {
    expr: All,
}

/// Build the combined expression that matches all wrong preposition patterns.
fn build_pattern_expr() -> All {
    let heads = WordSet::new(&[
        "different",
        "similar",
        "married",
        "compared",
        "superior",
        "inferior",
        "identical",
        "parallel",
    ]);

    let wrong_preps = WordSet::new(&[
        "than",   // "different than" → "different from"
        "then",   // "different then" → "different from"
        "with",   // "similar with" → "similar to", etc.
        "onto",   // "married onto" → "married to"
        "at",     // "married at" → "married to"
    ]);

    let noun_adj_pattern = SequenceExpr::with(heads)
        .then_whitespace()
        .then(wrong_preps);

    let heads2 = WordSet::new(&[
        "prefers",
        "prefer",
        "preferring",
        "preferred",
    ]);

    let verb_wrong_preps = WordSet::new(&["on", "at", "with"]);

    let verb_pattern = SequenceExpr::with(heads2)
        .then_whitespace()
        .then(verb_wrong_preps);

    noun_adj_pattern.or(verb_pattern)
}

impl Default for WrongPreposition {
    fn default() -> Self {
        Self {
            expr: build_pattern_expr(),
        }
    }
}

impl ExprLinter for WrongPreposition {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, tokens: &[Token], src: &[char]) -> Option<Lint> {
        if tokens.len() < 3 {
            return None;
        }

        let first_word = tokens[0].get_str(src).to_lowercase();
        let last_word = tokens.last().unwrap().get_str(src).to_lowercase();

        // Determine correct preposition based on the pattern matched
        let (correct_prep, message) = match (first_word.as_str(), last_word.as_str()) {
            // "different than" / "different then" → "different from"
            ("different", "than") | ("different", "then") => {
                ("from", "Use `different from` when comparing two distinct things.")
            }
            // "similar with" → "similar to"
            ("similar", "with") => {
                ("to", "Use `similar to` when comparing resemblance.")
            }
            // "married with/at/onto" → "married to"
            ("married", "with") | ("married", "at") | ("married", "onto") => {
                ("to", "Use `married to` when referring to a spouse.")
            }
            // "compared with" → "compared to"
            ("compared", "with") => {
                ("to", "Use `compared to` when making an analogy.")
            }
            // "superior with" → "superior to"
            ("superior", "with") => {
                ("to", "Use `superior to` when comparing quality or rank.")
            }
            // "inferior with" → "inferior to"
            ("inferior", "with") => {
                ("to", "Use `inferior to` when comparing quality or rank.")
            }
            // "identical with" → "identical to"
            ("identical", "with") => {
                ("to", "Use `identical to` when showing exact match.")
            }
            // "parallel with" → "parallel to"
            ("parallel", "with") => {
                ("to", "Use `parallel to` when referring to lines or comparisons.")
            }
            // "prefers on/at/with" → "prefers to"
            ("prefers", prep) | ("prefer", prep) | ("preferring", prep) | ("preferred", prep) => {
                if prep == "to" {
                    return None; // already correct
                }
                ("to", "Use `prefers to` when indicating preference.")
            }
            _ => return None,
        };

        let span = tokens.last().unwrap().span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                correct_prep.chars().collect(),
                last_word.chars().collect(),
            )],
            message: message.to_string(),
            priority: 55,
        })
    }

    fn description(&self) -> &str {
        "Detects wrong prepositions like `different than`, `similar with`, `married with` and suggests the correct preposition."
    }
}

#[cfg(test)]
mod tests {
    use super::WrongPreposition;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // Different than/then → different from
    #[test]
    fn flags_different_than() {
        assert_suggestion_result(
            "This is different than the original.",
            WrongPreposition::default(),
            "This is different from the original.",
        );
    }

    #[test]
    fn flags_different_then() {
        assert_suggestion_result(
            "This is different then the original.",
            WrongPreposition::default(),
            "This is different from the original.",
        );
    }

    #[test]
    fn flags_different_than_all_caps() {
        assert_suggestion_result(
            "THIS IS DIFFERENT THAN THE ORIGINAL.",
            WrongPreposition::default(),
            "THIS IS DIFFERENT FROM THE ORIGINAL.",
        );
    }

    #[test]
    fn ignores_different_from() {
        assert_lint_count(
            "This is different from the original.",
            WrongPreposition::default(),
            0,
        );
    }

    // Similar with → similar to
    #[test]
    fn flags_similar_with() {
        assert_suggestion_result(
            "My approach is similar with yours.",
            WrongPreposition::default(),
            "My approach is similar to yours.",
        );
    }

    #[test]
    fn ignores_similar_to() {
        assert_lint_count(
            "My approach is similar to yours.",
            WrongPreposition::default(),
            0,
        );
    }

    // Married with/at/onto → married to
    #[test]
    fn flags_married_with() {
        assert_suggestion_result(
            "She is married with a doctor.",
            WrongPreposition::default(),
            "She is married to a doctor.",
        );
    }

    #[test]
    fn flags_married_at() {
        assert_suggestion_result(
            "They got married at a priest.",
            WrongPreposition::default(),
            "They got married to a priest.",
        );
    }

    #[test]
    fn flags_married_onto() {
        assert_suggestion_result(
            "She got married onto a wealthy man.",
            WrongPreposition::default(),
            "She got married to a wealthy man.",
        );
    }

    #[test]
    fn ignores_married_to() {
        assert_lint_count(
            "She is married to a doctor.",
            WrongPreposition::default(),
            0,
        );
    }

    // Compared with → compared to
    #[test]
    fn flags_compared_with() {
        assert_suggestion_result(
            "Life is often compared with a journey.",
            WrongPreposition::default(),
            "Life is often compared to a journey.",
        );
    }

    #[test]
    fn ignores_compared_to() {
        assert_lint_count(
            "Life is often compared to a journey.",
            WrongPreposition::default(),
            0,
        );
    }

    // Superior/inferior with → superior/inferior to
    #[test]
    fn flags_superior_with() {
        assert_suggestion_result(
            "This model is superior with the previous version.",
            WrongPreposition::default(),
            "This model is superior to the previous version.",
        );
    }

    #[test]
    fn flags_inferior_with() {
        assert_suggestion_result(
            "The new version is inferior with the old one.",
            WrongPreposition::default(),
            "The new version is inferior to the old one.",
        );
    }

    #[test]
    fn ignores_superior_to() {
        assert_lint_count(
            "This model is superior to the previous version.",
            WrongPreposition::default(),
            0,
        );
    }

    // Identical with → identical to
    #[test]
    fn flags_identical_with() {
        assert_suggestion_result(
            "This signature is identical with the original.",
            WrongPreposition::default(),
            "This signature is identical to the original.",
        );
    }

    #[test]
    fn ignores_identical_to() {
        assert_lint_count(
            "This signature is identical to the original.",
            WrongPreposition::default(),
            0,
        );
    }

    // Parallel with → parallel to
    #[test]
    fn flags_parallel_with() {
        assert_suggestion_result(
            "This road runs parallel with the highway.",
            WrongPreposition::default(),
            "This road runs parallel to the highway.",
        );
    }

    #[test]
    fn ignores_parallel_to() {
        assert_lint_count(
            "This road runs parallel to the highway.",
            WrongPreposition::default(),
            0,
        );
    }

    // Prefers on/at/with → prefers to
    #[test]
    fn flags_prefers_on() {
        assert_suggestion_result(
            "She prefers on tea over coffee.",
            WrongPreposition::default(),
            "She prefers to tea over coffee.",
        );
    }

    #[test]
    fn flags_prefers_with() {
        assert_suggestion_result(
            "He prefers with the dark mode.",
            WrongPreposition::default(),
            "He prefers to the dark mode.",
        );
    }

    #[test]
    fn flags_prefers_at() {
        assert_suggestion_result(
            "I prefers at this option.",
            WrongPreposition::default(),
            "I prefers to this option.",
        );
    }

    #[test]
    fn flags_preferring_with() {
        assert_suggestion_result(
            "She is preferring with the blue one.",
            WrongPreposition::default(),
            "She is preferring to the blue one.",
        );
    }

    #[test]
    fn flags_preferred_with() {
        assert_suggestion_result(
            "The preferred with method is now outdated.",
            WrongPreposition::default(),
            "The preferred to method is now outdated.",
        );
    }

    #[test]
    fn ignores_prefers_to() {
        assert_lint_count(
            "She prefers to tea over coffee.",
            WrongPreposition::default(),
            0,
        );
    }
}