use super::{ExprLinter, Lint, LintKind};
use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;

pub struct Bought {
    expr: Box<dyn Expr>,
}

impl Default for Bought {
    fn default() -> Self {
        let subject = SequenceExpr::with(Self::is_subject_pronoun_like)
            .t_ws()
            .then_optional(SequenceExpr::default().then_adverb().t_ws())
            .then_optional(SequenceExpr::default().then_auxiliary_verb().t_ws())
            .then_optional(SequenceExpr::default().then_adverb().t_ws())
            .then_any_capitalization_of("bough");

        Self {
            expr: Box::new(subject),
        }
    }
}

impl ExprLinter for Bought {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let typo = matched_tokens.last()?;

        Some(Lint {
            span: typo.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "bought".chars().collect(),
                typo.span.get_content(source),
            )],
            message: "Prefer the past-tense form `bought` here.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Replaces the incorrect past-tense spelling `bough` with `bought` after subject pronouns."
    }
}

impl Bought {
    fn is_subject_pronoun_like(token: &Token, source: &[char]) -> bool {
        if token.kind.is_subject_pronoun() {
            return true;
        }

        if !token.kind.is_word() || !token.kind.is_apostrophized() {
            return false;
        }

        let text = token.span.get_content_string(source);
        let lower = text.to_ascii_lowercase();

        let Some((stem, suffix)) = lower.split_once('\'') else {
            return false;
        };

        let is_subject_stem = matches!(stem, "i" | "you" | "we" | "they" | "he" | "she" | "it");
        let is_supported_suffix = matches!(suffix, "d" | "ve");

        is_subject_stem && is_supported_suffix
    }
}

#[cfg(test)]
mod tests {
    use super::Bought;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_he_bough() {
        assert_suggestion_result(
            "He bough a laptop yesterday.",
            Bought::default(),
            "He bought a laptop yesterday.",
        );
    }

    #[test]
    fn corrects_she_never_bough() {
        assert_suggestion_result(
            "She never bough fresh herbs there.",
            Bought::default(),
            "She never bought fresh herbs there.",
        );
    }

    #[test]
    fn corrects_they_already_bough() {
        assert_suggestion_result(
            "They already bough the train tickets.",
            Bought::default(),
            "They already bought the train tickets.",
        );
    }

    #[test]
    fn corrects_we_have_bough() {
        assert_suggestion_result(
            "We have bough extra paint.",
            Bought::default(),
            "We have bought extra paint.",
        );
    }

    #[test]
    fn corrects_they_have_never_bough() {
        assert_suggestion_result(
            "They have never bough theatre seats online.",
            Bought::default(),
            "They have never bought theatre seats online.",
        );
    }

    #[test]
    fn corrects_ive_bough() {
        assert_suggestion_result(
            "I've bough the ingredients already.",
            Bought::default(),
            "I've bought the ingredients already.",
        );
    }

    #[test]
    fn corrects_wed_bough() {
        assert_suggestion_result(
            "We'd bough snacks before the film.",
            Bought::default(),
            "We'd bought snacks before the film.",
        );
    }

    #[test]
    fn no_lint_for_tree_bough() {
        assert_no_lints("The heavy bough cracked under the snow.", Bought::default());
    }

    #[test]
    fn no_lint_for_he_bought() {
        assert_no_lints("He bought a laptop yesterday.", Bought::default());
    }

    #[test]
    fn no_lint_for_plural_boughs() {
        assert_no_lints("Boughs swayed in the evening breeze.", Bought::default());
    }
}
