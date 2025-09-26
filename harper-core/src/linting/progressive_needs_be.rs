use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct ProgressiveNeedsBe {
    expr: Box<dyn Expr>,
}

impl Default for ProgressiveNeedsBe {
    fn default() -> Self {
        // Support both contracted (I've/We've/You've/They've) and non-contracted
        // (I have/We have/You have/They have) forms before a progressive verb.
        let contracted = SequenceExpr::word_set(&["I've", "We've", "You've", "They've"])
            .t_ws()
            .then_kind_both(TokenKind::is_verb, TokenKind::is_verb_progressive_form);

        let non_contracted = SequenceExpr::word_set(&["I", "We", "You", "They"])
            .t_ws()
            .then_any_capitalization_of("have")
            .t_ws()
            .then_kind_both(TokenKind::is_verb, TokenKind::is_verb_progressive_form);

        let expr = SequenceExpr::any_of(vec![Box::new(contracted), Box::new(non_contracted)]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ProgressiveNeedsBe {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        use crate::CharStringExt;

        // Collect the word tokens in the matched slice
        let word_toks: Vec<&Token> = toks.iter().filter(|t| t.kind.is_word()).collect();
        let first_word = *word_toks.first()?; // contraction or pronoun

        // If this is the non-contracted pattern, extend the replacement span to include "have"
        let have_tok_opt = word_toks
            .iter()
            .find(|t| t.span.get_content(src).eq_ignore_ascii_case_str("have"))
            .copied();

        let span = if let Some(have_tok) = have_tok_opt {
            crate::Span::new(first_word.span.start, have_tok.span.end)
        } else {
            first_word.span
        };

        // Choose the correct "be" contraction based on the pronoun
        let pronoun_str: String = first_word.span.get_content(src).iter().copied().collect();
        let lower = pronoun_str.to_lowercase();
        let progressive_replacement = if lower.starts_with("i") {
            "I'm"
        } else if lower.starts_with("we") {
            "We're"
        } else if lower.starts_with("you") {
            "You're"
        } else if lower.starts_with("they") {
            "They're"
        } else {
            "I'm"
        };

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case(
                    progressive_replacement.chars().collect(),
                    span.get_content(src),
                ),
                Suggestion::InsertAfter(" been".chars().collect()),
            ],
            message: "Use present progressive (`…'re/…'m …`) or present perfect progressive (`… have been …`/`…'ve been …`) instead of `… have …ing` or `…'ve …ing`.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects the ungrammatical patterns `<pronoun> have …ing` (e.g., `I have …ing`) and `<pronoun>'ve …ing` (e.g., `I've …ing`) and suggests either the present progressive (e.g., `I'm/We're/You're/They're …`) or the present perfect progressive (e.g., `I/We/You/They have been …` or `I've/We've/You've/They've been …`)."
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressiveNeedsBe;
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_lint_count, assert_suggestion_result,
    };

    #[test]
    fn suggests_im_looking() {
        assert_suggestion_result(
            "I've looking into it.",
            ProgressiveNeedsBe::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn corrects_basic_im() {
        assert_suggestion_result(
            "I've looking into it.",
            ProgressiveNeedsBe::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions() {
        assert_good_and_bad_suggestions(
            "I've looking into it.",
            ProgressiveNeedsBe::default(),
            &["I'm looking into it.", "I've been looking into it."],
            &[],
        );
    }

    #[test]
    fn allows_ive_looked() {
        assert_lint_count("I've looked into it.", ProgressiveNeedsBe::default(), 0);
    }

    #[test]
    fn allows_ive_been_looking() {
        assert_lint_count(
            "I've been looking into it.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }

    #[test]
    fn allows_ive_seen() {
        assert_lint_count("I've seen the results.", ProgressiveNeedsBe::default(), 0);
    }

    #[test]
    fn allows_ive_long_been_looking() {
        assert_lint_count(
            "I've long been looking into it.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }

    #[test]
    fn no_match_with_punctuation_between() {
        assert_lint_count("I've, looking into it.", ProgressiveNeedsBe::default(), 0);
    }

    #[test]
    fn handles_newline_whitespace() {
        assert_suggestion_result(
            "I've\nlooking into it.",
            ProgressiveNeedsBe::default(),
            "I'm\nlooking into it.",
        );
    }

    #[test]
    fn capitalization_all_caps_base() {
        assert_suggestion_result(
            "I'VE looking into it.",
            ProgressiveNeedsBe::default(),
            "I'M looking into it.",
        );
    }

    #[test]
    fn works_for_weve() {
        assert_suggestion_result(
            "We've looking into it.",
            ProgressiveNeedsBe::default(),
            "We're looking into it.",
        );
    }

    #[test]
    fn suggests_im_looking_non_contracted() {
        assert_suggestion_result(
            "I have looking into it.",
            ProgressiveNeedsBe::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions_non_contracted() {
        assert_good_and_bad_suggestions(
            "They have looking into it.",
            ProgressiveNeedsBe::default(),
            &[
                "They're looking into it.",
                "They have been looking into it.",
            ],
            &[],
        );
    }

    #[test]
    fn allows_i_have_been_looking() {
        assert_lint_count(
            "I have been looking into it.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }

    #[test]
    fn allows_i_have_looked() {
        assert_lint_count("I have looked into it.", ProgressiveNeedsBe::default(), 0);
    }

    // Additional generalized cases
    // Contracted: I've/We've/You've/They've + gerund
    #[test]
    fn ive_working() {
        assert_suggestion_result(
            "I've working on it today.",
            ProgressiveNeedsBe::default(),
            "I'm working on it today.",
        );
    }
    #[test]
    fn weve_working() {
        assert_suggestion_result(
            "We've working on it today.",
            ProgressiveNeedsBe::default(),
            "We're working on it today.",
        );
    }
    #[test]
    fn youve_working() {
        assert_suggestion_result(
            "You've working on it today.",
            ProgressiveNeedsBe::default(),
            "You're working on it today.",
        );
    }
    #[test]
    fn theyve_working() {
        assert_suggestion_result(
            "They've working on it today.",
            ProgressiveNeedsBe::default(),
            "They're working on it today.",
        );
    }

    #[test]
    fn ive_eating() {
        assert_suggestion_result(
            "I've eating it today.",
            ProgressiveNeedsBe::default(),
            "I'm eating it today.",
        );
    }
    #[test]
    fn weve_eating() {
        assert_suggestion_result(
            "We've eating it today.",
            ProgressiveNeedsBe::default(),
            "We're eating it today.",
        );
    }
    #[test]
    fn youve_eating() {
        assert_suggestion_result(
            "You've eating it today.",
            ProgressiveNeedsBe::default(),
            "You're eating it today.",
        );
    }
    #[test]
    fn theyve_eating() {
        assert_suggestion_result(
            "They've eating it today.",
            ProgressiveNeedsBe::default(),
            "They're eating it today.",
        );
    }

    #[test]
    fn ive_reading() {
        assert_suggestion_result(
            "I've reading it today.",
            ProgressiveNeedsBe::default(),
            "I'm reading it today.",
        );
    }
    #[test]
    fn weve_reading() {
        assert_suggestion_result(
            "We've reading it today.",
            ProgressiveNeedsBe::default(),
            "We're reading it today.",
        );
    }
    #[test]
    fn youve_reading() {
        assert_suggestion_result(
            "You've reading it today.",
            ProgressiveNeedsBe::default(),
            "You're reading it today.",
        );
    }
    #[test]
    fn theyve_reading() {
        assert_suggestion_result(
            "They've reading it today.",
            ProgressiveNeedsBe::default(),
            "They're reading it today.",
        );
    }

    #[test]
    fn ive_writing() {
        assert_suggestion_result(
            "I've writing it today.",
            ProgressiveNeedsBe::default(),
            "I'm writing it today.",
        );
    }
    #[test]
    fn weve_writing() {
        assert_suggestion_result(
            "We've writing it today.",
            ProgressiveNeedsBe::default(),
            "We're writing it today.",
        );
    }
    #[test]
    fn youve_writing() {
        assert_suggestion_result(
            "You've writing it today.",
            ProgressiveNeedsBe::default(),
            "You're writing it today.",
        );
    }
    #[test]
    fn theyve_writing() {
        assert_suggestion_result(
            "They've writing it today.",
            ProgressiveNeedsBe::default(),
            "They're writing it today.",
        );
    }

    #[test]
    fn ive_speaking() {
        assert_suggestion_result(
            "I've speaking about it today.",
            ProgressiveNeedsBe::default(),
            "I'm speaking about it today.",
        );
    }
    #[test]
    fn weve_speaking() {
        assert_suggestion_result(
            "We've speaking about it today.",
            ProgressiveNeedsBe::default(),
            "We're speaking about it today.",
        );
    }
    #[test]
    fn youve_speaking() {
        assert_suggestion_result(
            "You've speaking about it today.",
            ProgressiveNeedsBe::default(),
            "You're speaking about it today.",
        );
    }
    #[test]
    fn theyve_speaking() {
        assert_suggestion_result(
            "They've speaking about it today.",
            ProgressiveNeedsBe::default(),
            "They're speaking about it today.",
        );
    }

    #[test]
    fn ive_studying() {
        assert_suggestion_result(
            "I've studying it today.",
            ProgressiveNeedsBe::default(),
            "I'm studying it today.",
        );
    }
    #[test]
    fn weve_studying() {
        assert_suggestion_result(
            "We've studying it today.",
            ProgressiveNeedsBe::default(),
            "We're studying it today.",
        );
    }
    #[test]
    fn youve_studying() {
        assert_suggestion_result(
            "You've studying it today.",
            ProgressiveNeedsBe::default(),
            "You're studying it today.",
        );
    }
    #[test]
    fn theyve_studying() {
        assert_suggestion_result(
            "They've studying it today.",
            ProgressiveNeedsBe::default(),
            "They're studying it today.",
        );
    }

    #[test]
    fn ive_testing() {
        assert_suggestion_result(
            "I've testing it today.",
            ProgressiveNeedsBe::default(),
            "I'm testing it today.",
        );
    }
    #[test]
    fn weve_testing() {
        assert_suggestion_result(
            "We've testing it today.",
            ProgressiveNeedsBe::default(),
            "We're testing it today.",
        );
    }
    #[test]
    fn youve_testing() {
        assert_suggestion_result(
            "You've testing it today.",
            ProgressiveNeedsBe::default(),
            "You're testing it today.",
        );
    }
    #[test]
    fn theyve_testing() {
        assert_suggestion_result(
            "They've testing it today.",
            ProgressiveNeedsBe::default(),
            "They're testing it today.",
        );
    }

    #[test]
    fn ive_using() {
        assert_suggestion_result(
            "I've using it today.",
            ProgressiveNeedsBe::default(),
            "I'm using it today.",
        );
    }
    #[test]
    fn weve_using() {
        assert_suggestion_result(
            "We've using it today.",
            ProgressiveNeedsBe::default(),
            "We're using it today.",
        );
    }
    #[test]
    fn youve_using() {
        assert_suggestion_result(
            "You've using it today.",
            ProgressiveNeedsBe::default(),
            "You're using it today.",
        );
    }
    #[test]
    fn theyve_using() {
        assert_suggestion_result(
            "They've using it today.",
            ProgressiveNeedsBe::default(),
            "They're using it today.",
        );
    }

    // Non-contracted: I/We/You/They have + gerund
    #[test]
    fn i_have_working() {
        assert_suggestion_result(
            "I have working on it today.",
            ProgressiveNeedsBe::default(),
            "I'm working on it today.",
        );
    }
    #[test]
    fn we_have_working() {
        assert_suggestion_result(
            "We have working on it today.",
            ProgressiveNeedsBe::default(),
            "We're working on it today.",
        );
    }
    #[test]
    fn you_have_working() {
        assert_suggestion_result(
            "You have working on it today.",
            ProgressiveNeedsBe::default(),
            "You're working on it today.",
        );
    }
    #[test]
    fn they_have_working() {
        assert_suggestion_result(
            "They have working on it today.",
            ProgressiveNeedsBe::default(),
            "They're working on it today.",
        );
    }

    #[test]
    fn i_have_eating() {
        assert_suggestion_result(
            "I have eating it today.",
            ProgressiveNeedsBe::default(),
            "I'm eating it today.",
        );
    }
    #[test]
    fn we_have_eating() {
        assert_suggestion_result(
            "We have eating it today.",
            ProgressiveNeedsBe::default(),
            "We're eating it today.",
        );
    }
    #[test]
    fn you_have_eating() {
        assert_suggestion_result(
            "You have eating it today.",
            ProgressiveNeedsBe::default(),
            "You're eating it today.",
        );
    }
    #[test]
    fn they_have_eating() {
        assert_suggestion_result(
            "They have eating it today.",
            ProgressiveNeedsBe::default(),
            "They're eating it today.",
        );
    }

    #[test]
    fn i_have_reading() {
        assert_suggestion_result(
            "I have reading it today.",
            ProgressiveNeedsBe::default(),
            "I'm reading it today.",
        );
    }
    #[test]
    fn we_have_reading() {
        assert_suggestion_result(
            "We have reading it today.",
            ProgressiveNeedsBe::default(),
            "We're reading it today.",
        );
    }
    #[test]
    fn you_have_reading() {
        assert_suggestion_result(
            "You have reading it today.",
            ProgressiveNeedsBe::default(),
            "You're reading it today.",
        );
    }
    #[test]
    fn they_have_reading() {
        assert_suggestion_result(
            "They have reading it today.",
            ProgressiveNeedsBe::default(),
            "They're reading it today.",
        );
    }

    #[test]
    fn i_have_writing() {
        assert_suggestion_result(
            "I have writing it today.",
            ProgressiveNeedsBe::default(),
            "I'm writing it today.",
        );
    }
    #[test]
    fn we_have_writing() {
        assert_suggestion_result(
            "We have writing it today.",
            ProgressiveNeedsBe::default(),
            "We're writing it today.",
        );
    }
    #[test]
    fn you_have_writing() {
        assert_suggestion_result(
            "You have writing it today.",
            ProgressiveNeedsBe::default(),
            "You're writing it today.",
        );
    }
    #[test]
    fn they_have_writing() {
        assert_suggestion_result(
            "They have writing it today.",
            ProgressiveNeedsBe::default(),
            "They're writing it today.",
        );
    }

    #[test]
    fn i_have_speaking() {
        assert_suggestion_result(
            "I have speaking about it today.",
            ProgressiveNeedsBe::default(),
            "I'm speaking about it today.",
        );
    }
    #[test]
    fn we_have_speaking() {
        assert_suggestion_result(
            "We have speaking about it today.",
            ProgressiveNeedsBe::default(),
            "We're speaking about it today.",
        );
    }
    #[test]
    fn you_have_speaking() {
        assert_suggestion_result(
            "You have speaking about it today.",
            ProgressiveNeedsBe::default(),
            "You're speaking about it today.",
        );
    }
    #[test]
    fn they_have_speaking() {
        assert_suggestion_result(
            "They have speaking about it today.",
            ProgressiveNeedsBe::default(),
            "They're speaking about it today.",
        );
    }

    #[test]
    fn i_have_studying() {
        assert_suggestion_result(
            "I have studying it today.",
            ProgressiveNeedsBe::default(),
            "I'm studying it today.",
        );
    }
    #[test]
    fn we_have_studying() {
        assert_suggestion_result(
            "We have studying it today.",
            ProgressiveNeedsBe::default(),
            "We're studying it today.",
        );
    }
    #[test]
    fn you_have_studying() {
        assert_suggestion_result(
            "You have studying it today.",
            ProgressiveNeedsBe::default(),
            "You're studying it today.",
        );
    }
    #[test]
    fn they_have_studying() {
        assert_suggestion_result(
            "They have studying it today.",
            ProgressiveNeedsBe::default(),
            "They're studying it today.",
        );
    }

    #[test]
    fn i_have_testing() {
        assert_suggestion_result(
            "I have testing it today.",
            ProgressiveNeedsBe::default(),
            "I'm testing it today.",
        );
    }
    #[test]
    fn we_have_testing() {
        assert_suggestion_result(
            "We have testing it today.",
            ProgressiveNeedsBe::default(),
            "We're testing it today.",
        );
    }
    #[test]
    fn you_have_testing() {
        assert_suggestion_result(
            "You have testing it today.",
            ProgressiveNeedsBe::default(),
            "You're testing it today.",
        );
    }
    #[test]
    fn they_have_testing() {
        assert_suggestion_result(
            "They have testing it today.",
            ProgressiveNeedsBe::default(),
            "They're testing it today.",
        );
    }

    #[test]
    fn i_have_using() {
        assert_suggestion_result(
            "I have using it today.",
            ProgressiveNeedsBe::default(),
            "I'm using it today.",
        );
    }
    #[test]
    fn we_have_using() {
        assert_suggestion_result(
            "We have using it today.",
            ProgressiveNeedsBe::default(),
            "We're using it today.",
        );
    }
    #[test]
    fn you_have_using() {
        assert_suggestion_result(
            "You have using it today.",
            ProgressiveNeedsBe::default(),
            "You're using it today.",
        );
    }
    #[test]
    fn they_have_using() {
        assert_suggestion_result(
            "They have using it today.",
            ProgressiveNeedsBe::default(),
            "They're using it today.",
        );
    }

    // Both-suggestion checks
    #[test]
    fn both_suggestions_ive_working() {
        assert_good_and_bad_suggestions(
            "I've working today.",
            ProgressiveNeedsBe::default(),
            &["I'm working today.", "I've been working today."],
            &[],
        );
    }
    #[test]
    fn both_suggestions_we_have_reading() {
        assert_good_and_bad_suggestions(
            "We have reading it today.",
            ProgressiveNeedsBe::default(),
            &["We're reading it today.", "We have been reading it today."],
            &[],
        );
    }
    #[test]
    fn both_suggestions_youve_reading() {
        assert_good_and_bad_suggestions(
            "You've reading today.",
            ProgressiveNeedsBe::default(),
            &["You're reading today.", "You've been reading today."],
            &[],
        );
    }
    #[test]
    fn both_suggestions_they_have_writing() {
        assert_good_and_bad_suggestions(
            "They have writing today.",
            ProgressiveNeedsBe::default(),
            &["They're writing today.", "They have been writing today."],
            &[],
        );
    }

    // Non-match and allowed-form checks
    fn no_match_punctuation_contracted() {
        assert_lint_count("I've, working today.", ProgressiveNeedsBe::default(), 0);
    }
    #[test]
    fn no_match_punctuation_non_contracted() {
        assert_lint_count("I have, working today.", ProgressiveNeedsBe::default(), 0);
    }
    #[test]
    fn no_match_adverb_interruption() {
        assert_lint_count(
            "I have quickly working today.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }
    #[test]
    fn allowed_contracted_have_been() {
        assert_lint_count(
            "You've been studying today.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }
    #[test]
    fn allowed_non_contracted_have_been() {
        assert_lint_count(
            "You have been studying today.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }
    #[test]
    fn allowed_they_have_been() {
        assert_lint_count(
            "They have been testing today.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }
    #[test]
    fn allowed_theyve_been() {
        assert_lint_count(
            "They've been testing today.",
            ProgressiveNeedsBe::default(),
            0,
        );
    }

    #[test]
    fn capitalization_variants_non_contracted() {
        assert_suggestion_result(
            "WE HAVE working today.",
            ProgressiveNeedsBe::default(),
            "WE'RE working today.",
        );
    }
    #[test]
    fn newline_variants_non_contracted() {
        assert_suggestion_result(
            "We have\nworking on it today.",
            ProgressiveNeedsBe::default(),
            "We're\nworking on it today.",
        );
    }
}
