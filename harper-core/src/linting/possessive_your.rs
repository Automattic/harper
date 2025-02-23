use crate::{
    Token,
    patterns::{Pattern, SequencePattern},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct PossessiveYour {
    pattern: Box<dyn Pattern>,
}

impl Default for PossessiveYour {
    fn default() -> Self {
        let pattern =
            SequencePattern::aco("you")
                .then_whitespace()
                .then(|tok: &Token, _source: &[char]| {
                    tok.kind.is_nominal() && !tok.kind.is_likely_homograph()
                });

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for PossessiveYour {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let orig_chars = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("your".chars().collect(), orig_chars),
                Suggestion::replace_with_match_case("you're an".chars().collect(), orig_chars),
            ],
            message: "The possessive version of this word is more common in this context."
                .to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "The possessive version of `you` is more common before nouns."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::PossessiveYour;

    #[test]
    fn your_comments() {
        assert_suggestion_result(
            "You comments may end up in the documentation.",
            PossessiveYour::default(),
            "Your comments may end up in the documentation.",
        );
    }

    #[test]
    fn allow_intro_page() {
        assert_lint_count(
            "You can try out an editor that uses Harper under-the-hood here.",
            PossessiveYour::default(),
            0,
        );
    }
}
