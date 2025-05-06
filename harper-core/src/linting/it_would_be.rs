use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{OwnedPatternExt, Pattern, SequencePattern, WordSet},
};

pub struct ItWouldBe {
    pattern: Box<dyn Pattern>,
}

impl Default for ItWouldBe {
    fn default() -> Self {
        /* ─────────────── helpers ─────────────── */
        let head_verbs = WordSet::new(&["believe", "doubt", "think", "assume", "guess"]);
        let modals = WordSet::new(&["might", "would", "will"]);
        let adjectives = WordSet::new(&["good", "bad", "wonderful", "real"]);
        let tail_nouns = WordSet::new(&[
            "bummer",
            "pity",
            "shame",
            "pleasure",
            "idea",
            "experience",
            "problem",
            "catastrophe",
            "disaster",
            "trap",
            "challenge",
        ]);

        let branch = |has_not: bool, has_adj: bool| {
            let mut p = SequencePattern::default()
                .then(head_verbs.clone())
                .then_whitespace()
                .t_aco("i") // the mistaken pronoun
                .then_whitespace()
                .then(modals.clone());

            if has_not {
                p = p.then_whitespace().t_aco("not");
            }

            p = p.then_whitespace().t_aco("be").then_whitespace().t_aco("a");

            if has_adj {
                p = p.then_whitespace().then(adjectives.clone());
            }

            p.then_whitespace().then(tail_nouns.clone())
        };

        let combined = branch(false, false)
            .or(Box::new(branch(false, true)))
            .or(Box::new(branch(true, false)))
            .or(Box::new(branch(true, true)));

        Self {
            pattern: Box::new(combined),
        }
    }
}

impl PatternLinter for ItWouldBe {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let pronoun = &toks[2];
        let span = pronoun.span;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith("it".chars().collect())],
            message: "In this construction the pronoun should be “it”, not “I”. \
                      e.g. *“I think **it** would be a shame …”*"
                .to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Replaces the incorrect sequence “I might/would/will (not) be a …” with “it …”, \
         as in “I think **it** would be a shame.”"
    }
}

#[cfg(test)]
mod tests {
    use super::ItWouldBe;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_simple_shame() {
        assert_suggestion_result(
            "I think I would be a shame if this happened.",
            ItWouldBe::default(),
            "I think it would be a shame if this happened.",
        );
    }

    #[test]
    fn flags_believe_bummer() {
        assert_suggestion_result(
            "We believe I might not be a bummer after all.",
            ItWouldBe::default(),
            "We believe it might not be a bummer after all.",
        );
    }

    #[test]
    fn flags_doubt_good_idea() {
        assert_suggestion_result(
            "They doubt I will be a good idea for the team.",
            ItWouldBe::default(),
            "They doubt it will be a good idea for the team.",
        );
    }

    #[test]
    fn ignores_correct_it() {
        assert_lint_count(
            "I think it would be a shame if this happened.",
            ItWouldBe::default(),
            0,
        );
    }

    #[test]
    fn ignores_first_person_statement() {
        assert_lint_count(
            "I would be a good fit for the role.",
            ItWouldBe::default(),
            0,
        );
    }
}
