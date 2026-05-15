use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Span, Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct YourBeforeAuxiliary {
    expr: SequenceExpr,
}

impl Default for YourBeforeAuxiliary {
    fn default() -> Self {
        // Auxiliary verbs that commonly follow "your" in error
        let auxiliaries = WordSet::new(&[
            "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did",
            "will", "would", "shall", "should", "can", "could", "may", "might", "must",
        ]);

        // Pattern: "your" (case-insensitive) followed by whitespace then an auxiliary
        let pattern = SequenceExpr::aco("your")
            .then_whitespace()
            .then(auxiliaries);

        Self { expr: pattern }
    }
}

impl ExprLinter for YourBeforeAuxiliary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() < 3 {
            return None;
        }

        // toks[0] is "your" (the aco pattern)
        // toks[1] is whitespace (ignored)
        // toks[2] is the auxiliary verb
        let your_span = toks[0].span;
        let aux_span = toks[2].span;

        Some(Lint {
            span: your_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "you".chars().collect(),
                your_span.get_content(src),
            )],
            message: "Did you mean `you`? `your` is a possessive determiner, not a subject pronoun.".to_owned(),
            priority: 60,
        })
    }

    fn description(&self) -> &str {
        "Detects `your` mistakenly used as `you` before auxiliary verbs (e.g., `your is` → `you are`)."
    }
}

pub struct TheirBeforeAuxiliary {
    expr: SequenceExpr,
}

impl Default for TheirBeforeAuxiliary {
    fn default() -> Self {
        let auxiliaries = WordSet::new(&[
            "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did",
            "will", "would", "shall", "should", "can", "could", "may", "might", "must",
        ]);

        // Pattern: "their" (case-insensitive) followed by whitespace then an auxiliary
        let pattern = SequenceExpr::aco("their")
            .then_whitespace()
            .then(auxiliaries);

        Self { expr: pattern }
    }
}

impl ExprLinter for TheirBeforeAuxiliary {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() < 3 {
            return None;
        }

        let their_span = toks[0].span;

        Some(Lint {
            span: their_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case("there".chars().collect(), their_span.get_content(src)),
                Suggestion::replace_with_match_case("they're".chars().collect(), their_span.get_content(src)),
            ],
            message: "Did you mean `there` or `they're`? `their` is a possessive determiner, not a subject pronoun or adverb.".to_owned(),
            priority: 60,
        })
    }

    fn description(&self) -> &str {
        "Detects `their` mistakenly used as `there` or `they're` before auxiliary verbs."
    }
}

#[cfg(test)]
mod tests {
    use super::{YourBeforeAuxiliary, TheirBeforeAuxiliary};
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_your_is() {
        assert_suggestion_result(
            "Your profile name for your is GitHub account.",
            YourBeforeAuxiliary::default(),
            "you profile name for your is GitHub account.",
        );
    }

    #[test]
    fn flags_your_are() {
        assert_suggestion_result(
            "Your are going to love this.",
            YourBeforeAuxiliary::default(),
            "you are going to love this.",
        );
    }

    #[test]
    fn flags_your_was() {
        assert_suggestion_result(
            "Your was the best option.",
            YourBeforeAuxiliary::default(),
            "you was the best option.",
        );
    }

    #[test]
    fn flags_your_were() {
        assert_suggestion_result(
            "Your were there, weren't you?",
            YourBeforeAuxiliary::default(),
            "you were there, weren't you?",
        );
    }

    #[test]
    fn flags_your_have() {
        assert_suggestion_result(
            "Your have to see this.",
            YourBeforeAuxiliary::default(),
            "you have to see this.",
        );
    }

    #[test]
    fn flags_your_has() {
        assert_suggestion_result(
            "Your has been amazing.",
            YourBeforeAuxiliary::default(),
            "you has been amazing.",
        );
    }

    #[test]
    fn flags_your_can() {
        assert_suggestion_result(
            "Your can do it!",
            YourBeforeAuxiliary::default(),
            "you can do it!",
        );
    }

    #[test]
    fn flags_your_will() {
        assert_suggestion_result(
            "Your will succeed.",
            YourBeforeAuxiliary::default(),
            "you will succeed.",
        );
    }

    #[test]
    fn flags_your_do() {
        assert_suggestion_result(
            "Your do understand, right?",
            YourBeforeAuxiliary::default(),
            "you do understand, right?",
        );
    }

    #[test]
    fn flags_your_does() {
        assert_suggestion_result(
            "Your does look nice.",
            YourBeforeAuxiliary::default(),
            "you does look nice.",
        );
    }

    #[test]
    fn flags_your_did() {
        assert_suggestion_result(
            "Your did a great job.",
            YourBeforeAuxiliary::default(),
            "you did a great job.",
        );
    }

    #[test]
    fn flags_your_would() {
        assert_suggestion_result(
            "Your would be happy.",
            YourBeforeAuxiliary::default(),
            "you would be happy.",
        );
    }

    #[test]
    fn flags_your_should() {
        assert_suggestion_result(
            "Your should try this.",
            YourBeforeAuxiliary::default(),
            "you should try this.",
        );
    }

    #[test]
    fn flags_your_could() {
        assert_suggestion_result(
            "Your could win.",
            YourBeforeAuxiliary::default(),
            "you could win.",
        );
    }

    #[test]
    fn flags_your_may() {
        assert_suggestion_result(
            "Your may leave now.",
            YourBeforeAuxiliary::default(),
            "you may leave now.",
        );
    }

    #[test]
    fn flags_your_might() {
        assert_suggestion_result(
            "Your might be right.",
            YourBeforeAuxiliary::default(),
            "you might be right.",
        );
    }

    #[test]
    fn flags_your_must() {
        assert_suggestion_result(
            "Your must complete this.",
            YourBeforeAuxiliary::default(),
            "you must complete this.",
        );
    }

    #[test]
    fn flags_your_shall() {
        assert_suggestion_result(
            "Your shall be rewarded.",
            YourBeforeAuxiliary::default(),
            "you shall be rewarded.",
        );
    }

    #[test]
    fn flags_your_be() {
        assert_suggestion_result(
            "Your be quiet!",
            YourBeforeAuxiliary::default(),
            "you be quiet!",
        );
    }

    #[test]
    fn flags_your_being() {
        assert_suggestion_result(
            "Your being here helps.",
            YourBeforeAuxiliary::default(),
            "you being here helps.",
        );
    }

    #[test]
    fn flags_your_been() {
        assert_suggestion_result(
            "Your been working hard.",
            YourBeforeAuxiliary::default(),
            "you been working hard.",
        );
    }

    #[test]
    fn flags_your_all_caps() {
        assert_suggestion_result(
            "YOUR ARE THE BEST!",
            YourBeforeAuxiliary::default(),
            "you ARE THE BEST!",
        );
    }

    #[test]
    fn flags_your_lowercase() {
        assert_suggestion_result(
            "your is the key.",
            YourBeforeAuxiliary::default(),
            "you is the key.",
        );
    }

    #[test]
    fn ignores_your_noun() {
        assert_lint_count(
            "Your profile looks great.",
            YourBeforeAuxiliary::default(),
            0,
        );
    }

    #[test]
    fn ignores_your_verb() {
        assert_lint_count(
            "Your account has been verified.",
            YourBeforeAuxiliary::default(),
            0,
        );
    }

    #[test]
    fn flags_their_is() {
        assert_suggestion_result(
            "Their is no way to know.",
            TheirBeforeAuxiliary::default(),
            "there is no way to know.",
        );
    }

    #[test]
    fn flags_their_are() {
        assert_suggestion_result(
            "Their are many options.",
            TheirBeforeAuxiliary::default(),
            "there are many options.",
        );
    }

    #[test]
    fn flags_their_was() {
        assert_suggestion_result(
            "Their was a problem.",
            TheirBeforeAuxiliary::default(),
            "there was a problem.",
        );
    }

    #[test]
    fn flags_their_were() {
        assert_suggestion_result(
            "Their were some issues.",
            TheirBeforeAuxiliary::default(),
            "there were some issues.",
        );
    }

    #[test]
    fn flags_their_have() {
        assert_suggestion_result(
            "Their have been changes.",
            TheirBeforeAuxiliary::default(),
            "there have been changes.",
        );
    }

    #[test]
    fn flags_their_can() {
        assert_suggestion_result(
            "Their can be done.",
            TheirBeforeAuxiliary::default(),
            "there can be done.",
        );
    }

    #[test]
    fn flags_their_will() {
        assert_suggestion_result(
            "Their will be consequences.",
            TheirBeforeAuxiliary::default(),
            "there will be consequences.",
        );
    }

    #[test]
    fn flags_their_would() {
        assert_suggestion_result(
            "Their would be better.",
            TheirBeforeAuxiliary::default(),
            "there would be better.",
        );
    }

    #[test]
    fn flags_their_all_caps() {
        assert_suggestion_result(
            "THEIR ARE MANY WAYS.",
            TheirBeforeAuxiliary::default(),
            "there ARE MANY WAYS.",
        );
    }

    #[test]
    fn ignores_their_possessive() {
        assert_lint_count(
            "Their house is beautiful.",
            TheirBeforeAuxiliary::default(),
            0,
        );
    }

    #[test]
    fn ignores_their_verb() {
        assert_lint_count(
            "Their friends have arrived.",
            TheirBeforeAuxiliary::default(),
            0,
        );
    }
}