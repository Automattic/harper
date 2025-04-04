use super::Suggestion;
use super::pattern_linter::PatternLinter;
use crate::linting::LintKind;
use crate::patterns::{Pattern, SequencePattern, WordSet};
use crate::{CharStringExt, Lint, Lrc, Token, TokenStringExt};
use hashbrown::HashSet;

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
pub struct MultipleSequentialPronouns {
    pattern: Box<dyn Pattern>,
    subject_pronouns: HashSet<&'static str>,
    object_pronouns: HashSet<&'static str>,
    possessive_adjectives: HashSet<&'static str>,
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        // Some words occur in multiple positions in the paradigm
        // but this is a set, so it doesn't matter and is much clearer
        let pronouns = Lrc::new(WordSet::new(&[
            "I", "you", "he", "she", "it", // subject case, singular
            "me", "you", "him", "her", "it", // object case, singular
            "we", "you", "they", // subject case, plural
            "us", "you", "them", // object case, plural
            "mine", "yours", "his", "hers", // possessive pronouns, singular
            "ours", "yours", "theirs", // possessive pronouns, plural
            "my", "your", "his", "her", "its", // possessive adjectives, singular
            "our", "your", "their", // possessive adjectives, plural
        ]));

        // TODO: temporary sets of pronouns - remove when WordMetadata has this info
        let subject_pronouns = HashSet::from([
            "I", "you", "he", "she", "it", // subject case, singular
            "we", "you", "they", // subject case, plural
        ]);

        let object_pronouns = HashSet::from([
            "me", "you", "him", "her", "it", // object case, singular
            "us", "you", "them", // object case, plural
        ]);

        let possessive_adjectives = HashSet::from([
            "my", "your", "his", "her", "its", // possessive adjectives, singular
            "our", "your", "their", // possessive adjectives, plural
        ]);

        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then(pronouns.clone())
                    .then_one_or_more(
                        SequencePattern::default()
                            .then_whitespace()
                            .then(pronouns.clone()),
                    ),
            ),
            subject_pronouns,
            object_pronouns,
            possessive_adjectives,
        }
    }

    fn is_subject_pronoun(&self, word: &str) -> bool {
        self.subject_pronouns.contains(word)
    }

    fn is_object_pronoun(&self, word: &str) -> bool {
        self.object_pronouns.contains(word)
    }

    fn is_possessive_adjective(&self, word: &str) -> bool {
        self.possessive_adjectives.contains(word)
    }
}

impl PatternLinter for MultipleSequentialPronouns {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut suggestions = Vec::new();

        if matched_tokens.len() == 3 {
            let first_word = matched_tokens[0].span.get_content(source).to_string();
            let second_word = matched_tokens[2].span.get_content(source).to_string();
            // Bug 578: "I can lend you my car" - if 1st is object and second is possessive adjective, don't lint
            if self.is_object_pronoun(&first_word) && self.is_possessive_adjective(&second_word) {
                return None;
            }
            // Bug 724: "One told me they were able to begin reading" - if 1st is object ans second is subject, don't lint
            if self.is_object_pronoun(&first_word) && self.is_subject_pronoun(&second_word) {
                return None;
            }

            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[0].span.get_content(source).to_vec(),
            ));
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[2].span.get_content(source).to_vec(),
            ));
        }

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Repetition,
            message: "There are too many personal pronouns in sequence here.".to_owned(),
            priority: 63,
            suggestions,
        })
    }

    fn description(&self) -> &'static str {
        "When editing work to change point of view (i.e. first-person or third-person) it is common to add pronouns while neglecting to remove old ones. This rule catches cases where you have multiple disparate pronouns in sequence."
    }
}

impl Default for MultipleSequentialPronouns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MultipleSequentialPronouns;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn can_detect_two_pronouns() {
        assert_lint_count(
            "...little bit about my I want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn can_detect_three_pronouns() {
        assert_lint_count(
            "...little bit about my I you want to do.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn allows_single_pronouns() {
        assert_lint_count(
            "...little bit about I want to do.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn detects_multiple_pronouns_at_end() {
        assert_lint_count(
            "...I need to explain this to you them.",
            MultipleSequentialPronouns::new(),
            1,
        )
    }

    #[test]
    fn comma_separated() {
        assert_lint_count("To prove it, we...", MultipleSequentialPronouns::new(), 0)
    }

    #[test]
    fn dont_flag_578() {
        assert_lint_count(
            "I can lend you my car.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn dont_flag_724() {
        assert_lint_count(
            "One told me they were able to begin reading.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }
}
