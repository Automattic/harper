use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenKind};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct IvePerfectProgressive {
    expr: Box<dyn Expr>,
}

impl Default for IvePerfectProgressive {
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

impl ExprLinter for IvePerfectProgressive {
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
    use super::IvePerfectProgressive;
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_lint_count, assert_suggestion_result,
    };

    #[test]
    fn suggests_im_looking() {
        assert_suggestion_result(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn corrects_basic_im() {
        assert_suggestion_result(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions() {
        assert_good_and_bad_suggestions(
            "I've looking into it.",
            IvePerfectProgressive::default(),
            &["I'm looking into it.", "I've been looking into it."],
            &[],
        );
    }

    #[test]
    fn allows_ive_looked() {
        assert_lint_count("I've looked into it.", IvePerfectProgressive::default(), 0);
    }

    #[test]
    fn allows_ive_been_looking() {
        assert_lint_count(
            "I've been looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn allows_ive_seen() {
        assert_lint_count(
            "I've seen the results.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn allows_ive_long_been_looking() {
        assert_lint_count(
            "I've long been looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn no_match_with_punctuation_between() {
        assert_lint_count(
            "I've, looking into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn handles_newline_whitespace() {
        assert_suggestion_result(
            "I've\nlooking into it.",
            IvePerfectProgressive::default(),
            "I'm\nlooking into it.",
        );
    }

    #[test]
    fn capitalization_all_caps_base() {
        assert_suggestion_result(
            "I'VE looking into it.",
            IvePerfectProgressive::default(),
            "I'M looking into it.",
        );
    }

    #[test]
    fn works_for_weve() {
        assert_suggestion_result(
            "We've looking into it.",
            IvePerfectProgressive::default(),
            "We're looking into it.",
        );
    }

    #[test]
    fn suggests_im_looking_non_contracted() {
        assert_suggestion_result(
            "I have looking into it.",
            IvePerfectProgressive::default(),
            "I'm looking into it.",
        );
    }

    #[test]
    fn offers_both_suggestions_non_contracted() {
        assert_good_and_bad_suggestions(
            "They have looking into it.",
            IvePerfectProgressive::default(),
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
            IvePerfectProgressive::default(),
            0,
        );
    }

    #[test]
    fn allows_i_have_looked() {
        assert_lint_count(
            "I have looked into it.",
            IvePerfectProgressive::default(),
            0,
        );
    }
}
