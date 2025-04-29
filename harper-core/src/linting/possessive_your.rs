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
                .then(|tok: &Token, source: &[char]| {
                    tok.kind.is_nominal()
                        && !tok.kind.is_likely_homograph()
                        && tok.span.get_content(source) != ['g', 'u', 'y', 's']
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
                Suggestion::replace_with_match_case("you're a".chars().collect(), orig_chars),
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
    use crate::linting::tests::{
        assert_lint_count, assert_suggestion_result, assert_top3_suggestion_result,
    };

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

    #[test]
    fn allow_you_guys() {
        assert_lint_count(
            "I mean I'm pretty sure you guys can't do anything with this stuff.",
            PossessiveYour::default(),
            0,
        );
    }

    #[test]
    fn test_top3_suggestion_your() {
        assert_top3_suggestion_result(
            "You combination of artist and teacher.",
            PossessiveYour::default(),
            "Your combination of artist and teacher.",
        );
    }

    #[test]
    fn test_top3_suggestion_youre_a() {
        assert_top3_suggestion_result(
            "You combination of artist and teacher.",
            PossessiveYour::default(),
            "You're a combination of artist and teacher.",
        );
    }

    // #[test]
    // fn test_top3_suggestion_multiple() {
    //     assert_top3_suggestion_result(
    //         "You knowledge. You imagination. You icosahedron",
    //         PossessiveYour::default(),
    //         "Your knowledge. Your imagination. You're an icosahedron",
    //     );
    // }
}
