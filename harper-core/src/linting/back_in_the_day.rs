use crate::{
    Lrc, Token, TokenStringExt,
    patterns::{ExactPhrase, OwnedPatternExt, Pattern, SequencePattern, WordSet},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct BackInTheDay {
    pattern: Box<dyn Pattern>,
    // The trailing words that should tell us to ignore the rule.
    exceptions: Lrc<WordSet>,
}

impl Default for BackInTheDay {
    fn default() -> Self {
        let exceptions = Lrc::new(WordSet::new(&["before", "of", "when"]));
        let phrase = Lrc::new(ExactPhrase::from_phrase("back in the days"));

        let pattern = SequencePattern::default()
            .then(phrase.clone())
            .then_whitespace()
            .then(exceptions.clone())
            .or(Box::new(phrase));

        Self {
            pattern: Box::new(pattern),
            exceptions,
        }
    }
}

impl PatternLinter for BackInTheDay {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if let Some(tail) = matched_tokens.get(8..) {
            if self.exceptions.matches(tail, source).is_some() {
                return None;
            }
        }

        let span = matched_tokens.span()?;
        let chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "back in the day".chars().collect(),
                chars,
            )],
            message: "Use the more idiomatic version of this phrase.".to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "This linter flags instances of the nonstandard phrase `back in the days`. The correct, more accepted form is `back in the day`"
    }
}

#[cfg(test)]
mod tests {
    use super::BackInTheDay;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn detects_gem_update_case() {
        assert_suggestion_result(
            "... has been resolved through a gem update back in the days",
            BackInTheDay::default(),
            "... has been resolved through a gem update back in the day",
        );
    }

    #[test]
    fn detects_install_case() {
        assert_suggestion_result(
            "Back in the days we're used to install it directly from ...",
            BackInTheDay::default(),
            "Back in the day we're used to install it directly from ...",
        );
    }

    #[test]
    fn detects_composer_json_case() {
        assert_suggestion_result(
            "Back in the days there was only composer.json and ...",
            BackInTheDay::default(),
            "Back in the day there was only composer.json and ...",
        );
    }

    #[test]
    fn detects_version_release_case() {
        assert_suggestion_result(
            "... should have been released back in the days in a version 11",
            BackInTheDay::default(),
            "... should have been released back in the day in a version 11",
        );
    }

    #[test]
    fn avoids_false_positive_springfox() {
        assert_lint_count(
            "Back in the days of SpringFox, there were several requests to ...",
            BackInTheDay::default(),
            0,
        );
    }

    #[test]
    fn avoids_false_positive_ie() {
        assert_lint_count(
            "Back in the days of IE, Powershell used to ...",
            BackInTheDay::default(),
            0,
        );
    }

    #[test]
    fn avoids_false_positive_code_usage() {
        assert_lint_count(
            "Back in the days when I had 100% of my code in ...",
            BackInTheDay::default(),
            0,
        );
    }
    #[test]
    fn catches_uppercase() {
        assert_lint_count(
            "Back in the days, we went for a walk.",
            BackInTheDay::default(),
            1,
        );
    }

    #[test]
    fn catches_lowercase() {
        assert_lint_count(
            "We used to go for walks back in the days.",
            BackInTheDay::default(),
            1,
        );
    }

    #[test]
    fn doesnt_catch_false_positive_of() {
        assert_lint_count(
            "Back in the days of CRTs, computers were expensive.",
            BackInTheDay::default(),
            0,
        );
    }

    #[test]
    fn doesnt_catch_false_positive_when() {
        assert_lint_count(
            "Back in the days when videogame arcades were popular.",
            BackInTheDay::default(),
            0,
        );
    }

    #[test]
    fn catches_comma_when() {
        assert_lint_count(
            "Back in the days, when we were children, we played outside.",
            BackInTheDay::default(),
            1,
        );
    }

    #[test]
    fn doesnt_catch_false_positive_before() {
        assert_lint_count(
            "Back in the days before laptops we had \"luggables\".",
            BackInTheDay::default(),
            0,
        );
    }

    #[test]
    fn catches_comma_before() {
        assert_lint_count(
            "Back in the days, before laptops.",
            BackInTheDay::default(),
            1,
        );
    }

    #[test]
    fn doesnt_catch_qualified_days() {
        assert_lint_count(
            "Back in the old days we did this by hand.",
            BackInTheDay::default(),
            0,
        );
    }
}
