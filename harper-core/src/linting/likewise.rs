use crate::{
    patterns::{All, Invert, Pattern, SequencePattern},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct Likewise {
    pattern: Box<dyn Pattern>,
}
impl Default for Likewise {
    fn default() -> Self {
        let mut pattern = All::default();

        pattern.add(Box::new(
            SequencePattern::aco("like").then_whitespace().t_aco("wise"),
        ));

        pattern.add(Box::new(Invert::new(Box::new(
            SequencePattern::default()
                .then_anything()
                .then_whitespace()
                .then_anything()
                .then_whitespace()
                .then_noun(),
        ))));

        Self {
            pattern: Box::new(pattern),
        }
    }
}
impl PatternLinter for Likewise {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let orig_chars = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "likewise".chars().collect(),
                orig_chars,
            )],
            message: format!("Did you mean the closed compound `{}`?", "likewise"),
            ..Default::default()
        })
    }
    fn description(&self) -> &'static str {
        "Looks for incorrect spacing inside the closed compound `likewise`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Likewise;

    #[test]
    fn wise_men() {
        assert_suggestion_result(
            "Like wise men, we waited.",
            Likewise::default(),
            "Like wise men, we waited.",
        );
    }

    #[test]
    fn like_wise() {
        assert_suggestion_result(
            "He acted, like wise, without hesitation.",
            Likewise::default(),
            "He acted, likewise, without hesitation.",
        );
    }
}
