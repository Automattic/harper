use std::{ops::Range, sync::Arc};

use crate::{
    Token, TokenKind, TokenStringExt,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::expr_linter::Chunk,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::NominalPhrase,
};

pub struct SoonToBe {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<Range<usize>>>,
}

impl Default for SoonToBe {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let soon_to_be = || {
            SequenceExpr::default()
                .t_aco("soon")
                .t_ws()
                .t_aco("to")
                .t_ws()
                .t_aco("be")
        };

        let nominal_tail = || {
            SequenceExpr::optional(SequenceExpr::default().then_one_or_more_adverbs().t_ws())
                .then(NominalPhrase)
        };

        let hyphenated_number_modifier = || {
            SequenceExpr::default()
                .then_number()
                .then_hyphen()
                .then_nominal()
                .then_optional(SequenceExpr::default().then_hyphen().then_adjective())
                .t_ws()
                .then_nominal()
        };

        let hyphenated_compound = || {
            SequenceExpr::default()
                .then_kind_any(&[TokenKind::is_word_like as fn(&TokenKind) -> bool])
                .then_hyphen()
                .then_nominal()
        };

        let trailing_phrase = || {
            SequenceExpr::any_of(vec![
                Box::new(hyphenated_number_modifier()),
                Box::new(hyphenated_compound()),
                Box::new(nominal_tail()),
            ])
        };

        map.insert(
            SequenceExpr::default()
                .then_determiner()
                .t_ws()
                .then_seq(soon_to_be())
                .t_ws()
                .then_seq(trailing_phrase()),
            2..7,
        );

        map.insert(
            SequenceExpr::default()
                .then_seq(soon_to_be())
                .t_ws()
                .then_seq(trailing_phrase()),
            0..5,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for SoonToBe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let range = self.map.lookup(0, matched_tokens, source)?;
        let span = matched_tokens.get(range.start..range.end)?.span()?;
        let template = span.get_content(source);

        let mut nominal_found = false;
        for tok in matched_tokens.iter().skip(range.end) {
            if tok.kind.is_whitespace() || tok.kind.is_hyphen() {
                continue;
            }

            if tok.kind.is_punctuation() {
                break;
            }

            if tok.kind.is_nominal() {
                if tok.kind.is_preposition() {
                    continue;
                } else {
                    nominal_found = true;
                    break;
                }
            }
        }

        if !nominal_found {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "soon-to-be",
                template,
            )],
            message: "Use hyphens when `soon to be` modifies a noun.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Hyphenates `soon-to-be` when it appears before a noun."
    }
}

#[cfg(test)]
mod tests {
    use super::SoonToBe;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn hyphenates_possessive_phrase() {
        assert_suggestion_result(
            "We met his soon to be boss at lunch.",
            SoonToBe::default(),
            "We met his soon-to-be boss at lunch.",
        );
    }

    #[test]
    fn hyphenates_article_phrase() {
        assert_suggestion_result(
            "They toasted the soon to be couple.",
            SoonToBe::default(),
            "They toasted the soon-to-be couple.",
        );
    }

    #[test]
    fn hyphenates_sentence_start() {
        assert_suggestion_result(
            "Soon to be parents filled the classroom.",
            SoonToBe::default(),
            "Soon-to-be parents filled the classroom.",
        );
    }

    #[test]
    fn allows_existing_hyphens() {
        assert_no_lints("We met his soon-to-be boss yesterday.", SoonToBe::default());
    }

    #[test]
    fn keeps_non_adjectival_use() {
        assert_no_lints("The concert is soon to be over.", SoonToBe::default());
    }

    #[test]
    fn hyphenates_with_adverb() {
        assert_suggestion_result(
            "Our soon to be newly married friends visited.",
            SoonToBe::default(),
            "Our soon-to-be newly married friends visited.",
        );
    }

    #[test]
    fn hyphenates_hyphenated_number_phrase() {
        assert_suggestion_result(
            "Our soon to be 5-year-old son starts school.",
            SoonToBe::default(),
            "Our soon-to-be 5-year-old son starts school.",
        );
    }

    #[test]
    fn hyphenates_in_law_phrase() {
        assert_suggestion_result(
            "She thanked her soon to be in-laws for hosting.",
            SoonToBe::default(),
            "She thanked her soon-to-be in-laws for hosting.",
        );
    }

    #[test]
    fn hyphenates_future_event() {
        assert_suggestion_result(
            "We reserved space for our soon to be celebration.",
            SoonToBe::default(),
            "We reserved space for our soon-to-be celebration.",
        );
    }

    #[test]
    fn ignores_misaligned_verb_chain() {
        assert_lint_count(
            "They will soon to be moving overseas.",
            SoonToBe::default(),
            0,
        );
    }

    #[test]
    fn hyphenates_guest_example() {
        assert_suggestion_result(
            "I cooked for my soon to be guests.",
            SoonToBe::default(),
            "I cooked for my soon-to-be guests.",
        );
    }

    #[test]
    fn ignores_rearranged_phrase() {
        assert_no_lints("We hope to soon be home.", SoonToBe::default());
    }
}
