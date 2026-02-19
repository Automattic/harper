use super::Suggestion;
use super::expr_linter::ExprLinter;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::LintKind;
use crate::linting::expr_linter::Chunk;
use crate::patterns::WordSet;
use crate::{CharStringExt, Lint, Lrc, Token, TokenStringExt};

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
pub struct MultipleSequentialPronouns {
    expr: Box<dyn Expr>,
    subject_pronouns: Lrc<WordSet>,
    object_pronouns: Lrc<WordSet>,
    possessive_adjectives: Lrc<WordSet>,
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        // Some words occur in multiple positions in the paradigm
        // but this is a set, so it doesn't matter and is much clearer
        let pronouns = Lrc::new(WordSet::new(&[
            "i", "you", "he", "she", "it", // subject case, singular
            "me", "you", "him", "her", "it", // object case, singular
            "we", "you", "they", // subject case, plural
            "us", "you", "them", // object case, plural
            "mine", "yours", "his", "hers", // possessive pronouns, singular
            "ours", "yours", "theirs", // possessive pronouns, plural
            "my", "your", "his", "her", "its", // possessive adjectives, singular
            "our", "your", "their", // possessive adjectives, plural
        ]));

        // TODO: temporary sets of pronouns - remove when DictWordMetadata has this info
        let subject_pronouns = Lrc::new(WordSet::new(&[
            "i", "you", "he", "she", "it", // subject case, singular
            "we", "you", "they", // subject case, plural
        ]));

        let object_pronouns = Lrc::new(WordSet::new(&[
            "me", "you", "him", "her", "it", // object case, singular
            "us", "you", "them", // object case, plural
        ]));

        let possessive_adjectives = Lrc::new(WordSet::new(&[
            "my", "your", "his", "her", "its", // possessive adjectives, singular
            "our", "your", "their", // possessive adjectives, plural
        ]));

        Self {
            expr: Box::new(
                SequenceExpr::with(pronouns.clone())
                    .then_one_or_more(SequenceExpr::whitespace().then(pronouns.clone())),
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

impl ExprLinter for MultipleSequentialPronouns {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut suggestions = Vec::new();

        if matched_tokens.len() == 3 {
            let first_word_raw = matched_tokens[0].span.get_content(source).to_string();
            let first_word = first_word_raw.to_ascii_lowercase();
            let second_word = matched_tokens[2].span.get_content(source).to_string();
            // Bug 578: "I can lend you my car" - if 1st is object and second is possessive adjective, don't lint
            if self.is_object_pronoun(&first_word) && self.is_possessive_adjective(&second_word) {
                return None;
            }
            // Bug 724: "One told me they were able to begin reading" - if 1st is object ans second is subject, don't lint
            if self.is_object_pronoun(&first_word) && self.is_subject_pronoun(&second_word) {
                return None;
            }

            // US is a qualifier meaning American, so uppercase after a possessive is OK.
            if self.is_possessive_adjective(&first_word) && second_word == "US" {
                return None;
            }

            // The same applies to uppercase before a subject pronoun
            if first_word_raw == "US" && self.is_subject_pronoun(&second_word) {
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

    #[test]
    fn dont_flag_us() {
        assert_lint_count(
            "Take the plunge and pull plug from their US tech.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn dont_flag_my_us_your_us() {
        assert_lint_count(
            "My US passport looks different from your US passport.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn dont_flag_subject_after_usa() {
        assert_lint_count(
            "And if it’s manufactured in the US it may have more automation.",
            MultipleSequentialPronouns::new(),
            0,
        )
    }

    #[test]
    fn dont_flag_case_insensitive_cost_him_his_life() {
        assert_lint_count(
            "to the point where it very well likely cost Him his life",
            MultipleSequentialPronouns::new(),
            0,
        )
    }
}
