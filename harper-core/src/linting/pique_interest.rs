use crate::{
    CharString, CharStringExt, Token,
    char_string::char_string,
    patterns::{Pattern, SequencePattern, WordSet},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct PiqueInterest {
    pattern: Box<dyn Pattern>,
}

impl Default for PiqueInterest {
    fn default() -> Self {
        let pattern = SequencePattern::default()
            .then(WordSet::new(&[
                "peak", "peaked", "peek", "peeked", "peeking", "peaking",
            ]))
            .then_whitespace()
            .then_not_plural_nominal()
            .then_whitespace()
            .t_aco("interest");

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PiqueInterest {
    fn to_correct(word: &str) -> Option<CharString> {
        Some(match word.to_lowercase().as_str() {
            "peak" => char_string!("pique"),
            "peek" => char_string!("pique"),
            "peeked" => char_string!("piqued"),
            "peaked" => char_string!("piqued"),
            "peaking" => char_string!("piquing"),
            "peeking" => char_string!("piquing"),
            _ => return None,
        })
    }
}

impl PatternLinter for PiqueInterest {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0].span;
        let word = span.get_content_string(source).to_lowercase();
        let correct = Self::to_correct(&word)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                correct.to_vec(),
                matched_tokens[0].span.get_content(source),
            )],
            message: format!(
                "Did you mean `{}` instead of `{}`?",
                correct.to_string(),
                word,
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect usage of `peak` or `peek` when the intended word is `pique`, as in the phrase `you've peaked my interest`."
    }
}

#[cfg(test)]
mod tests {
    use super::PiqueInterest;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_peak_interest() {
        assert_suggestion_result(
            "The story managed to peak his interest.",
            PiqueInterest::default(),
            "The story managed to pique his interest.",
        );
    }

    #[test]
    fn corrects_peeked_interest_at_start() {
        assert_suggestion_result(
            "Peeked his interest, did she?",
            PiqueInterest::default(),
            "Piqued his interest, did she?",
        );
    }

    #[test]
    fn corrects_peak_interest_in_middle() {
        assert_suggestion_result(
            "She tried to peak his interest during the lecture.",
            PiqueInterest::default(),
            "She tried to pique his interest during the lecture.",
        );
    }

    #[test]
    fn corrects_peaked_interest_at_end() {
        assert_suggestion_result(
            "All along, she hoped she peaked his interest.",
            PiqueInterest::default(),
            "All along, she hoped she piqued his interest.",
        );
    }

    #[test]
    fn does_not_correct_unrelated_peak() {
        assert_suggestion_result(
            "He reached the peak of the mountain.",
            PiqueInterest::default(),
            "He reached the peak of the mountain.",
        );
    }

    #[test]
    fn corrects_peaking_interest() {
        assert_suggestion_result(
            "She was peaking his interest with her stories.",
            PiqueInterest::default(),
            "She was piquing his interest with her stories.",
        );
    }
}
