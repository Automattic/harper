use crate::{
    Lrc, Token, TokenStringExt,
    patterns::{OwnedPatternExt, Pattern, SequencePattern, WordSet},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct ModalOf {
    pattern: Box<dyn Pattern>,
}

impl Default for ModalOf {
    fn default() -> Self {
        // Note 1. "shan't of" is plausible but very unlikely
        // Note 2. "had of" has trickier false positives and is less common anyway
        // "The only other report we've had of this kind of problem ..."
        // "The code I had of this used to work fine ..."
        let modals = ["could", "might", "must", "should", "would"];
        let mut words = WordSet::new(&modals);
        modals.iter().for_each(|word| {
            words.add(&format!("{}n't", word));
        });

        let modal_of = Lrc::new(
            SequencePattern::default()
                .then(words)
                .then_whitespace()
                .then_exact_word("of"),
        );

        let ws_course = Lrc::new(
            SequencePattern::default()
                .then_whitespace()
                .then_exact_word("course"),
        );

        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then(modal_of.clone())
                    .then(ws_course.clone())
                    .or(Box::new(modal_of.clone())),
            ),
        }
    }
}

impl PatternLinter for ModalOf {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_toks: &[Token], source_chars: &[char]) -> Option<Lint> {
        if matched_toks.len() != 3 {
            return None;
        }

        let span_modal_of = matched_toks[0..3].span().unwrap();
        let span_modal = matched_toks[0].span;

        let modal_have = format!("{} have", span_modal.get_content_string(source_chars))
            .chars()
            .collect();
        let modal_ws_of = span_modal_of.get_content(source_chars);

        Some(Lint {
            span: span_modal_of,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(modal_have, modal_ws_of)],
            message: "Use `have` rather than `of` here.".to_string(),
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Detects `of` mistakenly used with `would`, `could`, `should`, etc."
    }
}

#[cfg(test)]
mod tests {
    use super::ModalOf;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // atomic unit tests

    #[test]
    fn test_lowercase() {
        assert_suggestion_result("could of", ModalOf::default(), "could have");
    }

    #[test]
    fn test_negative() {
        assert_suggestion_result("mightn't of", ModalOf::default(), "mightn't have");
    }

    #[test]
    fn test_uppercase_negative() {
        assert_suggestion_result("Mustn't of", ModalOf::default(), "Mustn't have");
    }

    #[test]
    fn test_false_positive() {
        assert_lint_count("should of course", ModalOf::default(), 0);
    }

    #[test]
    fn test_false_positive_capital_negative() {
        assert_lint_count("Wouldn't of course", ModalOf::default(), 0);
    }

    // real-world tests

    #[test]
    fn test_buggy_implementation() {
        assert_lint_count(
            "... could of just been a buggy implementation",
            ModalOf::default(),
            1,
        );
    }

    #[test]
    fn test_missed_one() {
        assert_lint_count(
            "We already have a function ... that nedb can understand so we might of missed one.",
            ModalOf::default(),
            1,
        );
    }

    #[test]
    fn test_user_option() {
        assert_lint_count(
            "im more likely to believe you might of left in the 'user' option",
            ModalOf::default(),
            1,
        );
    }

    #[test]
    fn catches_must_of() {
        assert_suggestion_result(
            "Ah I must of missed that part.",
            ModalOf::default(),
            "Ah I must have missed that part.",
        );
    }

    #[test]
    fn catches_should_of() {
        assert_lint_count(
            "Yeah I should of just mentioned it should of been a for of.",
            ModalOf::default(),
            2,
        );
    }

    #[test]
    fn catches_would_of() {
        assert_suggestion_result(
            "now this issue would of caused hundreds of thousands of extra lines",
            ModalOf::default(),
            "now this issue would have caused hundreds of thousands of extra lines",
        );
    }

    #[test]
    fn doesnt_catch_you_could_of_course() {
        assert_lint_count(
            "You could of course explicit the else with each possibility",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_compiler_could_of_course() {
        assert_lint_count(
            "The compiler could of course detect this too",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_might_of_course_be() {
        assert_lint_count(
            "There might of course be other places where not implementing the IMemberSource might break ...",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_not_a_must_of_course() {
        assert_lint_count(
            "Not a must of course if the convention should be .ts",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_must_of_course_also() {
        assert_lint_count(
            "the schedular must of course also have run through",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_should_of_course_not() {
        assert_lint_count(
            "not being local should of course not be supported",
            ModalOf::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_would_of_course_just() {
        assert_lint_count(
            "I would of course just test this by compiling with MATX_MULTI_GPU=ON",
            ModalOf::default(),
            0,
        );
    }
}
