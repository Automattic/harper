use crate::{
    Lrc, Token, TokenStringExt,
    patterns::{EitherPattern, Pattern, SequencePattern, SpelledNumberPattern, WordSet},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct SinceDuration {
    pattern: Box<dyn Pattern>,
}

impl Default for SinceDuration {
    fn default() -> Self {
        let units = WordSet::new(&[
            "minute", "minutes", "hour", "hours", "day", "days", "week", "weeks", "month",
            "months", "year", "years",
        ]);

        let pattern_without_ago = Lrc::new(
            SequencePattern::default()
                .then_any_capitalization_of("since")
                .then_whitespace()
                .then(EitherPattern::new(vec![
                    Box::new(SpelledNumberPattern),
                    Box::new(SequencePattern::default().then_number()),
                ]))
                .then_whitespace()
                .then(units),
        );

        let pattern_with_ago = SequencePattern::default()
            .then(pattern_without_ago.clone())
            .then_whitespace()
            .then_any_capitalization_of("ago");

        Self {
            pattern: Box::new(EitherPattern::new(vec![
                Box::new(pattern_without_ago),
                Box::new(pattern_with_ago),
            ])),
        }
    }
}

impl PatternLinter for SinceDuration {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if let Some(last) = toks.last() {
            if last.span.get_content_string(src).to_lowercase() == "ago" {
                return None;
            }
            let unit_charslice = last.span.get_content(src);
            let mut unit_plus_ago = unit_charslice.to_vec();
            unit_plus_ago.push(' ');
            unit_plus_ago.extend("ago".chars());

            let suggestion = Suggestion::ReplaceWith(unit_plus_ago);
            let suggestions = vec![suggestion];
            return Some(Lint {
                span: last.span,
                lint_kind: LintKind::Miscellaneous,
                suggestions,
                message: "'Since' requires a point in time, not a duration. Adding 'ago' usually fixes the issue.".to_string(),
                priority: 50,
            });
        }
        None
    }

    fn description(&self) -> &str {
        "Detects the use of 'since' with a duration instead of a point in time."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::SinceDuration;
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
    };

    #[test]
    fn catches_spelled() {
        assert_lint_count(
            "I have been waiting since two hours.",
            SinceDuration::default(),
            1,
        );
    }

    #[test]
    fn permits_spelled_with_ago() {
        assert_lint_count(
            "I have been waiting since two hours ago.",
            SinceDuration::default(),
            0,
        );
    }

    #[test]
    fn catches_numerals() {
        assert_lint_count(
            "I have been waiting since 2 hours.",
            SinceDuration::default(),
            1,
        );
    }

    #[test]
    fn permits_numerals_with_ago() {
        assert_lint_count(
            "I have been waiting since 2 hours ago.",
            SinceDuration::default(),
            0,
        );
    }

    #[test]
    fn correct_without_issues() {
        assert_suggestion_result(
            "I'm running v2.2.1 on bare metal (no docker, vm) since two weeks without issues.",
            SinceDuration::default(),
            "I'm running v2.2.1 on bare metal (no docker, vm) since two weeks ago without issues.",
        );
    }

    #[test]
    fn correct_anything_back() {
        assert_suggestion_result(
            "I have not heard anything back since three months.",
            SinceDuration::default(),
            "I have not heard anything back since three months ago.",
        );
    }

    #[test]
    fn correct_get_done() {
        assert_suggestion_result(
            "I am trying to get this done since two days, someone please help.",
            SinceDuration::default(),
            "I am trying to get this done since two days ago, someone please help.",
        );
    }

    #[test]
    fn correct_deprecated() {
        assert_suggestion_result(
            "This project is now officially deprecated, since I worked with virtualabs on the next version of Mirage since three years now: an ecosystem of tools named WHAD.",
            SinceDuration::default(),
            "This project is now officially deprecated, since I worked with virtualabs on the next version of Mirage since three years ago now: an ecosystem of tools named WHAD.",
        );
    }

    #[test]
    fn correct_same() {
        assert_top3_suggestion_result(
            "Same! Since two days.",
            SinceDuration::default(),
            "Same! Since two days ago.",
        );
    }

    #[test]
    fn correct_what_changed() {
        assert_suggestion_result(
            "What changed since two weeks?",
            SinceDuration::default(),
            "What changed since two weeks ago?",
        );
    }

    #[test]
    fn correct_with_period() {
        assert_suggestion_result(
            "I have been waiting since two hours.",
            SinceDuration::default(),
            "I have been waiting since two hours ago.",
        );
    }

    #[test]
    fn correct_with_exclamation() {
        assert_suggestion_result(
            "I have been waiting since two hours!",
            SinceDuration::default(),
            "I have been waiting since two hours ago!",
        );
    }

    #[test]
    fn correct_with_question_mark() {
        assert_suggestion_result(
            "Have you been waiting since two hours?",
            SinceDuration::default(),
            "Have you been waiting since two hours ago?",
        );
    }

    #[test]
    fn correct_with_comma() {
        assert_suggestion_result(
            "Since two days, I have been trying to get this done.",
            SinceDuration::default(),
            "Since two days ago, I have been trying to get this done.",
        );
    }
}
