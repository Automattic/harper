use crate::{
    patterns::{EitherPattern, Pattern, SequencePattern},
    Token,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct PluralConjugate {
    pattern: Box<dyn Pattern>,
}

impl Default for PluralConjugate {
    fn default() -> Self {
        let plural_case = SequencePattern::default()
            .then_plural_noun()
            .then_whitespace()
            .then_exact_word("is");

        let non_plural_case = SequencePattern::default()
            .then(Box::new(|tok: &Token, _source: &[char]| {
                tok.kind.is_not_plural_noun() && tok.kind.is_noun()
            }))
            .then_whitespace()
            .then_exact_word("are");

        let pat = EitherPattern::new(vec![Box::new(plural_case), Box::new(non_plural_case)]);

        Self {
            pattern: Box::new(pat),
        }
    }
}

impl PatternLinter for PluralConjugate {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Lint {
        let should_be_plural = matched_tokens.first().unwrap().kind.is_plural_noun();

        let sug = if should_be_plural {
            vec!['a', 'r', 'e']
        } else {
            vec!['i', 's']
        };

        Lint {
            span: matched_tokens.last().unwrap().span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(sug)],
            message: "Use the alternative conjugation of this verb to be consistent with the noun's plural nature.".to_owned(),
            priority: 63,
        }
    }

    fn description(&self) -> &'static str {
        "Make sure you use the correct conjugation of the verb \"to be\" in plural contexts."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::PluralConjugate;

    #[test]
    fn issue_256() {
        assert_suggestion_result(
            "The bananas is tasty",
            PluralConjugate::default(),
            "The bananas are tasty",
        );
    }

    #[test]
    fn plural_students() {
        assert_suggestion_result(
            "The students is doing their homework.",
            PluralConjugate::default(),
            "The students are doing their homework.",
        );
    }

    #[test]
    fn singular_house() {
        assert_suggestion_result(
            "The house are just sitting there.",
            PluralConjugate::default(),
            "The house is just sitting there.",
        );
    }

    #[test]
    fn review_doc_page() {
        assert_lint_count(
            "If you are testing it, try harder.",
            PluralConjugate::default(),
            0,
        );
    }

    #[test]
    fn pronoun_singular() {
        assert_suggestion_result(
            "If he are testing it.",
            PluralConjugate::default(),
            "If he is testing it.",
        );
    }
}
