use crate::Lrc;
use crate::Token;
use crate::expr::ExprMap;
use crate::expr::{Expr, SequenceExpr};

use super::Suggestion;
use super::{ExprLinter, Lint, LintKind};

pub struct PronounInflectionBe {
    expr: Box<dyn Expr>,
    map: Lrc<ExprMap<&'static str>>,
}

impl PronounInflectionBe {
    pub fn new() -> Self {
        let mut map = ExprMap::default();

        let are = SequenceExpr::default()
            .then_third_person_singular_pronoun()
            .t_ws()
            .t_aco("are");
        map.insert(are, "is");

        let is = SequenceExpr::default()
            .then_third_person_plural_pronoun()
            .t_ws()
            .t_aco("is");
        map.insert(is, "are");

        let map = Lrc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl Default for PronounInflectionBe {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for PronounInflectionBe {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.last()?.span;

        // The correct form
        let correct = self.map.lookup(0, matched_tokens, source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                correct,
                span.get_content(source),
            )],
            message: "With the singular pronouns “he”, “she”, and “it”, \
                      the verb “be” must take the singular form “is”."
                .to_string(),
            priority: 30,
        })
    }

    fn description(&self) -> &str {
        "Ensures basic subject-verb agreement by flagging instances where a \
         third-person singular pronoun (“he”, “she”, “it”) is incorrectly \
         paired with the plural verb form “are”. English grammar requires \
         the singular inflection “is” in these cases."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::PronounInflectionBe;

    #[test]
    fn corrects_he_are() {
        assert_suggestion_result(
            "He are my best friend.",
            PronounInflectionBe::default(),
            "He is my best friend.",
        );
    }

    #[test]
    fn corrects_she_are() {
        assert_suggestion_result(
            "She are my best friend.",
            PronounInflectionBe::default(),
            "She is my best friend.",
        );
    }

    #[test]
    fn corrects_they_is() {
        assert_suggestion_result(
            "They is my best friend.",
            PronounInflectionBe::default(),
            "They are my best friend.",
        );
    }

    #[test]
    fn allows_they_are() {
        assert_lint_count(
            "They are my best friend.",
            PronounInflectionBe::default(),
            0,
        );
    }

    #[test]
    fn corrects_it_are() {
        assert_suggestion_result(
            "It are on the table.",
            PronounInflectionBe::default(),
            "It is on the table.",
        );
    }

    #[test]
    fn corrects_he_are_negation() {
        assert_suggestion_result(
            "He are not amused.",
            PronounInflectionBe::default(),
            "He is not amused.",
        );
    }

    #[test]
    fn corrects_she_are_progressive() {
        assert_suggestion_result(
            "She are going to win.",
            PronounInflectionBe::default(),
            "She is going to win.",
        );
    }

    #[test]
    fn corrects_they_is_negation() {
        assert_suggestion_result(
            "They is not ready.",
            PronounInflectionBe::default(),
            "They are not ready.",
        );
    }

    #[test]
    fn corrects_they_is_progressive() {
        assert_suggestion_result(
            "They is planning a trip.",
            PronounInflectionBe::default(),
            "They are planning a trip.",
        );
    }

    #[test]
    fn allows_he_is() {
        assert_lint_count("He is my best friend.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn allows_she_is_lowercase() {
        assert_lint_count("she is excited to go.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn allows_it_is() {
        assert_lint_count("It is what it is.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn allows_they_are_negation() {
        assert_lint_count(
            "They are not interested.",
            PronounInflectionBe::default(),
            0,
        );
    }

    #[test]
    fn allows_they_were() {
        assert_lint_count("They were already here.", PronounInflectionBe::default(), 0);
    }
}
