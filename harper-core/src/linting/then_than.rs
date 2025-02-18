use super::{Lint, LintKind, PatternLinter};
use crate::linting::Suggestion;
use crate::patterns::{OwnedPatternExt, Pattern, SequencePattern, WordSet};
use crate::Token;

#[doc = "Corrects the misuse of `then` to `than`."]
pub struct ThenThan {
    pattern: Box<dyn Pattern>,
}
impl ThenThan {
    pub fn new() -> Self {
        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then(Box::new(WordSet::all(&["better", "other"]).or(Box::new(
                        |tok: &Token, _source: &[char]| {
                            tok.kind.is_adjective() && !tok.kind.is_noun()
                        },
                    ))))
                    .then_whitespace()
                    .then_any_capitalization_of("then"),
            ),
        }
    }
}

impl Default for ThenThan {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternLinter for ThenThan {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.last()?.span;
        let offending_text = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                "than".chars().collect(),
                offending_text,
            )],
            message: "Did you mean `than`?".to_string(),
            priority: 31,
        })
    }
    fn description(&self) -> &'static str {
        "Corrects the misuse of `then` to `than`."
    }
}

#[cfg(test)]
mod tests {
    use super::ThenThan;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn stronger_should_change() {
        assert_suggestion_result(
            "a chain is no stronger then its weakest link",
            ThenThan::default(),
            "a chain is no stronger than its weakest link",
        );
    }

    #[test]
    fn half_a_loaf_should_change() {
        assert_suggestion_result(
            "half a loaf is better then no bread",
            ThenThan::default(),
            "half a loaf is better than no bread",
        );
    }

    #[test]
    fn then_everyone_clapped_should_be_allowed() {
        assert_lint_count("and then everyone clapped", ThenThan::default(), 0);
    }

    #[test]
    fn crazier_than_rat_should_change() {
        assert_suggestion_result(
            "crazier then a shithouse rat",
            ThenThan::default(),
            "crazier than a shithouse rat",
        );
    }

    #[test]
    fn poke_in_eye_should_change() {
        assert_suggestion_result(
            "better then a poke in the eye with a sharp stick",
            ThenThan::default(),
            "better than a poke in the eye with a sharp stick",
        );
    }

    #[test]
    fn other_then_should_change() {
        assert_suggestion_result(
            "There was no one other then us at the campsite.",
            ThenThan::default(),
            "There was no one other than us at the campsite.",
        );
    }
}
