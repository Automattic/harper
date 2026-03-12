use harper_brill::UPOS;

use crate::expr::{All, AnchorStart, Expr, ExprMap, SequenceExpr};
use crate::patterns::{NominalPhrase, UPOSSet};
use crate::{Lrc, Token, TokenKind, TokenStringExt};

use super::Suggestion;
use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct PronounInflectionBe {
    expr: Box<dyn Expr>,
    map: Lrc<ExprMap<&'static str>>,
}

impl PronounInflectionBe {
    pub fn new() -> Self {
        let mod_term = Lrc::new(
            SequenceExpr::default()
                .t_ws()
                .then(UPOSSet::new(&[UPOS::ADJ, UPOS::ADV])),
        );

        let mut map = ExprMap::default();

        let are = SequenceExpr::default()
            .then_third_person_singular_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("are")
            .t_any()
            .then_unless(NominalPhrase);
        map.insert(are, "is");

        let are_at_start = SequenceExpr::with(AnchorStart)
            .then_third_person_singular_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("are")
            .t_any()
            .t_any();
        map.insert(are_at_start, "is");

        let arent = SequenceExpr::default()
            .then_third_person_singular_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("aren't")
            .t_any()
            .t_any();
        map.insert(arent, "isn't");

        let is = SequenceExpr::default()
            .then_kind_where(|kind| {
                kind.as_word()
                    .as_ref()
                    .and_then(|m| m.as_ref().and_then(|m| m.np_member))
                    .unwrap_or_default()
            })
            .then_whitespace()
            .then_third_person_plural_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("is")
            .t_any()
            .t_any();
        map.insert(is, "are");

        let is_at_start = SequenceExpr::with(AnchorStart)
            .then_third_person_plural_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("is")
            .t_any()
            .t_any();
        map.insert(is_at_start, "are");

        let isnt = SequenceExpr::default()
            .then_third_person_plural_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("isn't")
            .t_any()
            .t_any();
        map.insert(isnt, "aren't");

        let was = SequenceExpr::default()
            .then_first_person_plural_pronoun()
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("was")
            .t_any()
            .t_any();
        map.insert(was, "were");

        // Special case for second and third-person
        let was_third = SequenceExpr::with(AnchorStart)
            .then_kind_either(
                TokenKind::is_third_person_plural_pronoun,
                TokenKind::is_second_person_pronoun,
            )
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("was")
            .t_any()
            .t_any();
        map.insert(was_third, "were");

        let were = SequenceExpr::with(AnchorStart)
            .then_kind_either(
                TokenKind::is_first_person_singular_pronoun,
                TokenKind::is_third_person_singular_pronoun,
            )
            .then_optional(mod_term.clone())
            .t_ws()
            .t_aco("were")
            .t_any()
            .t_any();

        map.insert(were, "was");

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
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.get_rel(-3)?.span;

        // Determine the correct inflection of "be".
        let correct = self.map.lookup(0, matched_tokens, source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
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
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

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

    #[test]
    fn corrects_i_were() {
        assert_suggestion_result(
            "I were the best player on the field.",
            PronounInflectionBe::default(),
            "I was the best player on the field.",
        );
    }

    #[test]
    fn corrects_we_was() {
        assert_suggestion_result(
            "We was the best players on the field.",
            PronounInflectionBe::default(),
            "We were the best players on the field.",
        );
    }

    #[test]
    fn corrects_you_was() {
        assert_suggestion_result(
            "You was my best friend.",
            PronounInflectionBe::default(),
            "You were my best friend.",
        );
    }

    #[test]
    fn allows_you_were() {
        assert_lint_count(
            "You were my best friend.",
            PronounInflectionBe::default(),
            0,
        );
    }

    #[test]
    fn corrects_he_were() {
        assert_suggestion_result(
            "He were late.",
            PronounInflectionBe::default(),
            "He was late.",
        );
    }

    #[test]
    fn corrects_they_was() {
        assert_suggestion_result(
            "They was on time.",
            PronounInflectionBe::default(),
            "They were on time.",
        );
    }

    #[test]
    fn allows_he_was() {
        assert_lint_count("He was here.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn allows_we_were() {
        assert_lint_count("We were excited.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn corrects_he_arent() {
        assert_suggestion_result(
            "He aren't ready.",
            PronounInflectionBe::default(),
            "He isn't ready.",
        );
    }

    #[test]
    fn corrects_they_isnt() {
        assert_suggestion_result(
            "They isn't coming.",
            PronounInflectionBe::default(),
            "They aren't coming.",
        );
    }

    #[test]
    fn allows_he_isnt() {
        assert_lint_count("He isn't ready.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn allows_they_arent() {
        assert_lint_count("They aren't coming.", PronounInflectionBe::default(), 0);
    }

    #[test]
    fn corrects_she_really_are() {
        assert_suggestion_result(
            "She really are talented.",
            PronounInflectionBe::default(),
            "She really is talented.",
        );
    }

    #[test]
    fn corrects_they_often_is() {
        assert_suggestion_result(
            "They often is late.",
            PronounInflectionBe::default(),
            "They often are late.",
        );
    }

    #[test]
    fn corrects_because_he_are() {
        assert_suggestion_result(
            "because he are tired.",
            PronounInflectionBe::default(),
            "because he is tired.",
        );
    }

    #[test]
    fn allow_behind_him() {
        assert_no_lints(
            "Behind him are new shadows.",
            PronounInflectionBe::default(),
        );
    }

    #[test]
    fn issue_1682() {
        assert_no_lints(
            "Understanding them is significant",
            PronounInflectionBe::default(),
        );
    }
}
