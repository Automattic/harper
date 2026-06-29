use super::super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Number, Punctuation, Span, Token, TokenStringExt};
use harper_brill::UPOS;

/// Linter that checks if `criterion` or `phenomenon` should be plural after a
/// nearby plural modifier.
#[derive(Default)]
pub struct PluralCriterionPhenomenon;

impl Linter for PluralCriterionPhenomenon {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (noun_index, token) in chunk.iter().enumerate() {
                let Some(noun) = lower_word(token, document) else {
                    continue;
                };

                if matches!(noun.as_str(), "criterion" | "phenomenon")
                    && has_plural_modifier_before(chunk, noun_index, document)
                {
                    lints.push(make_lint(token.span, &noun));
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "The words “criteria” and “phenomena” are the plurals of “criterion” and “phenomenon”, respectively. This rule checks for singular forms used after plural modifiers."
    }
}

fn make_lint(span: Span<char>, noun: &str) -> Lint {
    let replacement = match noun {
        "criterion" => "criteria",
        "phenomenon" => "phenomena",
        _ => unreachable!("unexpected criterion/phenomenon noun: {noun}"),
    };

    Lint {
        span,
        lint_kind: LintKind::Repetition,
        message: "Use the plural form after a plural modifier.".to_owned(),
        priority: 63,
        suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
    }
}

fn has_plural_modifier_before(chunk: &[Token], noun_index: usize, document: &Document) -> bool {
    let mut word_count = 0;
    let mut index = noun_index;

    while let Some(previous_index) = previous_significant_token(chunk, index) {
        let token = &chunk[previous_index];

        match modifier_number(token, document) {
            ModifierNumber::Plural => return true,
            ModifierNumber::Singular => return false,
            ModifierNumber::Unknown => {}
        }

        if token.kind.is_word() {
            word_count += 1;
            if word_count > 4 {
                return false;
            }
        } else if !is_parenthetical_punctuation(token) {
            return false;
        }

        index = previous_index;
    }

    false
}

fn previous_significant_token(chunk: &[Token], before: usize) -> Option<usize> {
    (0..before)
        .rev()
        .find(|index| !chunk[*index].kind.is_whitespace())
}

fn lower_word(token: &Token, document: &Document) -> Option<String> {
    token.kind.is_word().then(|| {
        let mut word: String = token
            .span
            .get_content(document.get_source())
            .iter()
            .copied()
            .collect();
        word.make_ascii_lowercase();
        word
    })
}

enum ModifierNumber {
    Plural,
    Singular,
    Unknown,
}

fn modifier_number(token: &Token, document: &Document) -> ModifierNumber {
    if let Some(Number {
        value,
        suffix: None,
        ..
    }) = token.kind.as_number()
    {
        return if (value.0 - 1.0).abs() > f64::EPSILON {
            ModifierNumber::Plural
        } else {
            ModifierNumber::Singular
        };
    }

    let Some(word) = lower_word(token, document) else {
        return ModifierNumber::Unknown;
    };

    if token.kind.is_upos(UPOS::NUM) {
        return if word == "one" {
            ModifierNumber::Singular
        } else {
            ModifierNumber::Plural
        };
    }

    if token.kind.is_quantifier() {
        return match word.as_str() {
            "each" | "every" => ModifierNumber::Singular,
            "both" | "few" | "fewer" | "many" | "multiple" | "several" => ModifierNumber::Plural,
            _ => ModifierNumber::Unknown,
        };
    }

    if token.kind.is_demonstrative_determiner() {
        return match word.as_str() {
            "these" | "those" => ModifierNumber::Plural,
            "this" | "that" => ModifierNumber::Singular,
            _ => ModifierNumber::Unknown,
        };
    }

    if matches!(word.as_str(), "a" | "an") {
        return ModifierNumber::Singular;
    }

    ModifierNumber::Unknown
}

fn is_parenthetical_punctuation(token: &Token) -> bool {
    matches!(
        token.kind.as_punctuation(),
        Some(
            Punctuation::OpenRound
                | Punctuation::CloseRound
                | Punctuation::OpenSquare
                | Punctuation::CloseSquare
                | Punctuation::OpenCurly
                | Punctuation::CloseCurly
                | Punctuation::Hyphen
        )
    )
}

#[cfg(test)]
mod tests {
    use super::PluralCriterionPhenomenon;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn detects_reported_namesake_example() {
        assert_suggestion_result(
            "To embody this original goal, I wanted to make the project a namesake of an author known for their literary devices and prose. I also had three other (practical) criterion:",
            PluralCriterionPhenomenon,
            "To embody this original goal, I wanted to make the project a namesake of an author known for their literary devices and prose. I also had three other (practical) criteria:",
        );
    }

    #[test]
    fn detects_plural_number_with_singular_criterion() {
        assert_suggestion_result(
            "I also had three other (practical) criterion.",
            PluralCriterionPhenomenon,
            "I also had three other (practical) criteria.",
        );
    }

    #[test]
    fn detects_digit_with_singular_criterion() {
        assert_suggestion_result(
            "We used 3 criterion.",
            PluralCriterionPhenomenon,
            "We used 3 criteria.",
        );
    }

    #[test]
    fn detects_these_criterion() {
        assert_suggestion_result(
            "These criterion matter.",
            PluralCriterionPhenomenon,
            "These criteria matter.",
        );
    }

    #[test]
    fn detects_those_criterion() {
        assert_suggestion_result(
            "Those criterion failed.",
            PluralCriterionPhenomenon,
            "Those criteria failed.",
        );
    }

    #[test]
    fn detects_several_phenomenon() {
        assert_suggestion_result(
            "Several strange phenomenon appeared.",
            PluralCriterionPhenomenon,
            "Several strange phenomena appeared.",
        );
    }

    #[test]
    fn detects_multiple_adjectives_before_singular_noun() {
        assert_suggestion_result(
            "Two other practical review criterion failed.",
            PluralCriterionPhenomenon,
            "Two other practical review criteria failed.",
        );
    }

    #[test]
    fn detects_parenthetical_phenomenon() {
        assert_suggestion_result(
            "Many other (rare) phenomenon appeared.",
            PluralCriterionPhenomenon,
            "Many other (rare) phenomena appeared.",
        );
    }

    #[test]
    fn detects_hyphenated_intervening_words() {
        assert_suggestion_result(
            "Both long-term criterion changed.",
            PluralCriterionPhenomenon,
            "Both long-term criteria changed.",
        );
    }

    #[test]
    fn allows_correct_plural_criteria() {
        assert_lint_count(
            "Three other practical criteria were considered.",
            PluralCriterionPhenomenon,
            0,
        )
    }

    #[test]
    fn allows_correct_plural_phenomena() {
        assert_lint_count(
            "Many rare phenomena were on display.",
            PluralCriterionPhenomenon,
            0,
        )
    }

    #[test]
    fn allows_correct_singular_criterion() {
        assert_lint_count("That criterion is important.", PluralCriterionPhenomenon, 0)
    }

    #[test]
    fn allows_correct_singular_phenomenon() {
        assert_lint_count(
            "One strange phenomenon appeared.",
            PluralCriterionPhenomenon,
            0,
        )
    }

    #[test]
    fn allows_the_other_criterion() {
        assert_lint_count(
            "The other criterion was practical.",
            PluralCriterionPhenomenon,
            0,
        )
    }

    #[test]
    fn allows_singular_number_with_singular_criterion() {
        assert_lint_count("1 criterion was enough.", PluralCriterionPhenomenon, 0)
    }

    #[test]
    fn allows_comparative_modifier_with_singular_phenomenon() {
        assert_lint_count(
            "To the wingless a more interesting phenomenon is their dissimilarity.",
            PluralCriterionPhenomenon,
            0,
        )
    }

    #[test]
    fn allows_ambiguous_quantifier_with_singular_criterion() {
        assert_lint_count("No criterion was enough.", PluralCriterionPhenomenon, 0)
    }

    #[test]
    fn allows_plural_modifier_too_far_away() {
        assert_lint_count(
            "Three teams reviewed the policy before the criterion changed.",
            PluralCriterionPhenomenon,
            0,
        )
    }
}
