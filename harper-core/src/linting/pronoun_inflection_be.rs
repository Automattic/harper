use harper_brill::UPOS;

use crate::Lrc;
use crate::Token;
use crate::expr::All;
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

        let mut all = All::default();
        all.add(map.clone());
        all.add(|tok: &Token, _: &[char]| tok.kind.is_upos(UPOS::PRON));

        Self {
            expr: Box::new(all),
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
        let span = matched_tokens.get(2)?.span;

        // Determine the correct inflection of "be".
        let correct = self.map.lookup(0, matched_tokens, source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                correct,
                span.get_content(source),
            )],
            message: "Make the verb agree with its subject.".to_owned(),
            priority: 30,
        })
    }
    fn description(&self) -> &str {
        "Checks subject–verb agreement for the verb `be`. Third-person singular \
         pronouns (`he`, `she`, `it`) require the singular form `is`, while the \
         plural pronoun `they` takes `are`. The linter flags mismatches such as \
         `He are` or `They is` and offers the correct concord."
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

    #[test]
    fn allows_asdf_is() {
        assert_lint_count("asdf is not a word", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn no_subject() {
        assert_lint_count("is set", PronounInflectionBe::default(), 0);
    }
}
