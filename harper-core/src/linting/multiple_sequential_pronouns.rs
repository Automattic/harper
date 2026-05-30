use crate::{
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    {Lint, Lrc, Token, TokenStringExt},
};

/// Linter that checks if multiple pronouns are being used right after each
/// other. This is a common mistake to make during the revision process.
pub struct MultipleSequentialPronouns {
    expr: SequenceExpr,
}

impl MultipleSequentialPronouns {
    fn new() -> Self {
        let pronouns = Lrc::new(|t: &Token, _s: &[char]| {
            t.kind.is_subject_pronoun() // e.g. I
                || t.kind.is_object_pronoun() // e.g. me
                || t.kind.is_possessive_pronoun() // e.g. mine
                || t.kind.is_possessive_determiner() // e.g. my
        });

        Self {
            expr: SequenceExpr::with(pronouns.clone())
                .then_one_or_more(SequenceExpr::whitespace().then(pronouns.clone())),
        }
    }
}

impl ExprLinter for MultipleSequentialPronouns {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut suggestions = Vec::new();

        if matched_tokens.len() == 3 {
            let first_word_tok = &matched_tokens[0];
            let second_word_tok = &matched_tokens[2];

            let first_word_raw = first_word_tok.get_ch(source);
            let second_word_raw = second_word_tok.get_ch(source);
            // Bug 578: "I can lend you my car" - if 1st is object and second is possessive adjective, don't lint
            if first_word_tok.kind.is_object_pronoun()
                && second_word_tok.kind.is_possessive_determiner()
            {
                return None;
            }
            // Bug 724: "One told me they were able to begin reading" - if 1st is object ans second is subject, don't lint
            if first_word_tok.kind.is_object_pronoun() && second_word_tok.kind.is_subject_pronoun()
            {
                return None;
            }

            // US is a qualifier meaning American, so uppercase after a possessive is OK.
            // Likewise, IT means Information Technology, as in "our IT director"
            if first_word_tok.kind.is_possessive_determiner()
                && (second_word_raw == ['U', 'S'] || second_word_raw == ['I', 'T'])
            {
                return None;
            }

            // The same applies to uppercase before a subject pronoun
            if first_word_raw == ['U', 'S'] && second_word_tok.kind.is_subject_pronoun() {
                return None;
            }

            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[0].get_ch(source).to_vec(),
            ));
            suggestions.push(Suggestion::ReplaceWith(
                matched_tokens[2].get_ch(source).to_vec(),
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

    #[test]
    fn dont_flag_2870() {
        assert_lint_count(
            "their sales derp was just having none of it when our IT director told him, point blank, that we're not moving anything into the cloud",
            MultipleSequentialPronouns::new(),
            0,
        )
    }
}
