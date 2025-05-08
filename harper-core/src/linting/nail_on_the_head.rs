use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
};

pub struct NailOnTheHead {
    pattern: Box<dyn Pattern>,
}

impl Default for NailOnTheHead {
    fn default() -> Self {
        let mis = WordSet::new(&["hat", "had", "hit", "hid"]);
        let pattern = SequencePattern::default()
            .t_aco("nail")
            .then_whitespace()
            .t_aco("on")
            .then_whitespace()
            .t_aco("the")
            .then_whitespace()
            .then(mis);
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for NailOnTheHead {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let offender = toks.last()?;
        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith("head".chars().collect())],
            message: "Did you mean `head`?".to_owned(),
            priority: 45,
        })
    }

    fn description(&self) -> &str {
        "Replaces hat/had/hit/hid in the idiom `nail on the head` with `head`."
    }
}

#[cfg(test)]
mod tests {
    use super::NailOnTheHead;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_hat() {
        assert_suggestion_result(
            "She hit the nail on the hat.",
            NailOnTheHead::default(),
            "She hit the nail on the head.",
        );
    }

    #[test]
    fn fix_had() {
        assert_suggestion_result(
            "You really put the nail on the had with that comment.",
            NailOnTheHead::default(),
            "You really put the nail on the head with that comment.",
        );
    }

    #[test]
    fn fix_hit() {
        assert_suggestion_result(
            "They hit the nail on the hit regarding our problem.",
            NailOnTheHead::default(),
            "They hit the nail on the head regarding our problem.",
        );
    }

    #[test]
    fn fix_hid() {
        assert_suggestion_result(
            "The article nails the nail on the hid this time.",
            NailOnTheHead::default(),
            "The article nails the nail on the head this time.",
        );
    }

    #[test]
    fn ignore_correct() {
        assert_lint_count("She hit the nail on the head.", NailOnTheHead::default(), 0);
    }
}
