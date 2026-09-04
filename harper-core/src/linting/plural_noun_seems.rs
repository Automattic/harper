use crate::{
    CharStringExt, IrregularNouns, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    regular_nouns::get_singulars,
    spell::FstDictionary,
};

pub struct PluralNounSeems {
    expr: SequenceExpr,
}

impl Default for PluralNounSeems {
    fn default() -> Self {
        let expr = SequenceExpr::default()
            .then_kind_where(|kind| {
                kind.is_plural_noun()
                    && !kind.is_singular_noun()
                    && !kind.is_mass_noun()
                    && !kind.is_proper_noun()
            })
            .t_ws()
            .t_aco("seems");

        Self { expr }
    }
}

impl ExprLinter for PluralNounSeems {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let noun = matched_tokens.first()?;
        let noun_chars = noun.get_ch(source);
        if noun_chars.eq_any_ignore_ascii_case_str(&["barracks", "crossroads", "means", "works"]) {
            return None;
        }

        let has_known_singular = !get_singulars(&FstDictionary::curated(), noun_chars).is_empty()
            || IrregularNouns::curated()
                .get_singular_for_plural_chars(noun_chars)
                .is_some();

        if !has_known_singular {
            return None;
        }

        if let Some((before, _)) = context {
            if before.iter().any(|token| token.kind.is_conjunction()) {
                return None;
            }

            let phrase_start = before
                .iter()
                .rposition(|token| {
                    token.kind.is_sentence_terminator() || token.kind.is_paragraph_break()
                })
                .map_or(0, |index| index + 1);

            let phrase = &before[phrase_start..];
            if let Some(preposition_index) =
                phrase.iter().rposition(|token| token.kind.is_preposition())
            {
                let after_preposition = &phrase[preposition_index + 1..];
                let preposition_starts_clause = phrase[..preposition_index]
                    .iter()
                    .all(|token| token.kind.is_whitespace());
                let first_nominal = after_preposition
                    .iter()
                    .position(|token| token.kind.is_nominal());
                let prepositional_object_is_complete = preposition_starts_clause
                    && first_nominal.is_some_and(|index| {
                        after_preposition[index + 1..]
                            .iter()
                            .any(|token| token.kind.is_determiner() || token.kind.is_punctuation())
                    });

                if !prepositional_object_is_complete {
                    return None;
                }
            }
        }

        let offender = matched_tokens.last()?;
        let original = offender.get_ch(source);

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case_str("seem", original)],
            message: "A plural subject takes the verb form `seem`.".to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `seems` to `seem` after an unambiguously plural noun."
    }
}

#[cfg(test)]
mod tests {
    use super::PluralNounSeems;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_issue_example() {
        assert_suggestion_result(
            "Web pages seems to work pretty well so far.",
            PluralNounSeems::default(),
            "Web pages seem to work pretty well so far.",
        );
    }

    #[test]
    fn corrects_simple_plural_subject() {
        assert_suggestion_result(
            "The results seems promising.",
            PluralNounSeems::default(),
            "The results seem promising.",
        );
    }

    #[test]
    fn corrects_plural_after_demonstrative() {
        assert_suggestion_result(
            "These settings seems reasonable.",
            PluralNounSeems::default(),
            "These settings seem reasonable.",
        );
    }

    #[test]
    fn corrects_plural_after_quantifier() {
        assert_suggestion_result(
            "Several options seems viable.",
            PluralNounSeems::default(),
            "Several options seem viable.",
        );
    }

    #[test]
    fn corrects_irregular_plural() {
        assert_suggestion_result(
            "The children seems excited.",
            PluralNounSeems::default(),
            "The children seem excited.",
        );
    }

    #[test]
    fn corrects_uppercase() {
        assert_suggestion_result(
            "THE RESULTS SEEMS PROMISING.",
            PluralNounSeems::default(),
            "THE RESULTS SEEM PROMISING.",
        );
    }

    #[test]
    fn corrects_across_newline() {
        assert_suggestion_result(
            "The results\nseems promising.",
            PluralNounSeems::default(),
            "The results\nseem promising.",
        );
    }

    #[test]
    fn corrects_before_question_mark() {
        assert_suggestion_result(
            "These changes seems safe?",
            PluralNounSeems::default(),
            "These changes seem safe?",
        );
    }

    #[test]
    fn corrects_compound_subject_head() {
        assert_suggestion_result(
            "The profile pages seems unfinished.",
            PluralNounSeems::default(),
            "The profile pages seem unfinished.",
        );
    }

    #[test]
    fn corrects_plural_with_adjective() {
        assert_suggestion_result(
            "The newest reports seems accurate.",
            PluralNounSeems::default(),
            "The newest reports seem accurate.",
        );
    }

    #[test]
    fn allows_singular_subject() {
        assert_no_lints("The web page seems to work.", PluralNounSeems::default());
    }

    #[test]
    fn allows_agreeing_plural_subject() {
        assert_no_lints("The web pages seem to work.", PluralNounSeems::default());
    }

    #[test]
    fn allows_pronoun_subject() {
        assert_no_lints("It seems to work.", PluralNounSeems::default());
    }

    #[test]
    fn allows_plural_inside_prepositional_phrase() {
        for sentence in [
            "The list of pages seems complete.",
            "The list of old pages seems complete.",
            "One of the pages seems broken.",
            "The number of search results seems wrong.",
            "Each of the remaining options seems viable.",
            "A box with red handles seems sturdy.",
            "The difference between apples and the oranges seems obvious.",
            "The choice between the old plans and these alternatives seems difficult.",
            "The choice between apples, bananas, and oranges seems difficult.",
        ] {
            assert_no_lints(sentence, PluralNounSeems::default());
        }
    }

    #[test]
    fn corrects_after_completed_introductory_phrase() {
        for (sentence, corrected) in [
            (
                "At present the pages seems broken.",
                "At present the pages seem broken.",
            ),
            (
                "In this case the results seems promising.",
                "In this case the results seem promising.",
            ),
            (
                "At present, pages seems broken.",
                "At present, pages seem broken.",
            ),
        ] {
            assert_suggestion_result(sentence, PluralNounSeems::default(), corrected);
        }
    }

    #[test]
    fn allows_singular_mass_noun() {
        for sentence in [
            "The news seems encouraging.",
            "The data seems clear.",
            "The means seems justified.",
            "The crossroads seems dangerous.",
            "The barracks seems empty.",
            "The works seems abandoned.",
        ] {
            assert_no_lints(sentence, PluralNounSeems::default());
        }
    }

    #[test]
    fn allows_number_ambiguous_noun() {
        assert_no_lints("The series seems popular.", PluralNounSeems::default());
    }

    #[test]
    fn allows_singular_proper_name() {
        assert_no_lints(
            "The United States seems divided.",
            PluralNounSeems::default(),
        );
    }

    #[test]
    fn allows_possessive_subject_head() {
        assert_no_lints(
            "The users' experience seems smooth.",
            PluralNounSeems::default(),
        );
    }
}
