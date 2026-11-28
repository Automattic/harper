use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word, preceded_by_word},
    },
};

const CARDIOLOGY_BEFORE: &[&str] = &["fixed", "wide", "paradoxical", "reversed", "physiological"];

const CARDIOLOGY_AFTER: &[&str] = &["heart", "sound", "sounds"];

const ORDINAL_NOUNS: &[&str] = &[
    "array", "arrays", "part", "parts", "half", "halves", "tier", "tiers", "level", "levels",
    "attempt", "attempts",
];

const SINGULAR_OUTER_DETS: &[&str] = &[
    "a", "an", "the", "every", "same", "that", "this", "each", "another", "one",
];

const PLURAL_OUTER_DETS: &[&str] = &["these", "those", "such", "many", "few", "several"];

/// Flags blends of `split second` and `fraction of a second` (such as `split of a second`).
pub struct SplitOfASecond {
    expr: SequenceExpr,
}

impl Default for SplitOfASecond {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["split", "splits"])
                .t_ws()
                .t_aco("of")
                .then_optional(SequenceExpr::whitespace().then_word_set(["a", "the"]))
                .t_ws()
                .then_word_set(["second", "seconds"]),
        }
    }
}

impl ExprLinter for SplitOfASecond {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let split_tok = toks.first()?;
        let of_tok = toks.get(2)?;
        let second_tok = toks.last()?;

        // Guardrail 3: require lowercase `of`, inner article, and `second(s)` so title-cased
        // proper nouns like `Split of a Second` are never flagged.
        if of_tok.get_ch(src) != ['o', 'f'] {
            return None;
        }
        let second_ch = second_tok.get_ch(src);
        let second_is_plural = if second_ch == ['s', 'e', 'c', 'o', 'n', 'd'] {
            false
        } else if second_ch == ['s', 'e', 'c', 'o', 'n', 'd', 's'] {
            true
        } else {
            return None;
        };
        if toks.len() == 7 {
            let inner = toks.get(4)?.get_ch(src);
            if inner != ['a'] && inner != ['t', 'h', 'e'] {
                return None;
            }
        }

        // Guardrail 1: cardiology / auscultation terminology (`split of [the] second heart sound`).
        if preceded_by_word(ctx, |t| {
            t.get_ch(src)
                .eq_any_ignore_ascii_case_str(CARDIOLOGY_BEFORE)
        }) || followed_by_word(ctx, |t| {
            t.get_ch(src).eq_any_ignore_ascii_case_str(CARDIOLOGY_AFTER)
        }) {
            return None;
        }

        let split_is_plural = split_tok
            .get_ch(src)
            .eq_any_ignore_ascii_case_str(&["splits"]);

        // Guardrail 2: data structures / ordinal nouns (`splits of second array`).
        if split_is_plural
            && toks.len() == 5
            && !second_is_plural
            && followed_by_word(ctx, |t| {
                t.get_ch(src).eq_any_ignore_ascii_case_str(ORDINAL_NOUNS)
            })
        {
            return None;
        }

        let span = toks.span()?;
        let chars = span.get_content(src);

        let has_singular_det = preceded_by_word(ctx, |t| {
            t.get_ch(src)
                .eq_any_ignore_ascii_case_str(SINGULAR_OUTER_DETS)
        });
        let has_plural_det = preceded_by_word(ctx, |t| {
            t.get_ch(src)
                .eq_any_ignore_ascii_case_str(PLURAL_OUTER_DETS)
        });
        let before_strict_noun = followed_by_word(ctx, |t| t.kind.is_noun() && !t.kind.is_verb());

        let replacements: &[&str] = if has_singular_det {
            &["split second"]
        } else if has_plural_det {
            &["split-second", "split second"]
        } else if split_is_plural && second_is_plural {
            &["split seconds", "a split second"]
        } else if before_strict_noun {
            &["split-second", "a split second", "split second"]
        } else {
            &["a split second", "split-second"]
        };

        let suggestions = replacements
            .iter()
            .map(|r| Suggestion::replace_with_match_case_str(r, chars))
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Are you confusing `split second` and `fraction of a second`?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `split of a second` blends to the idiomatic `split second` or `split-second`."
    }
}

#[cfg(test)]
mod tests {
    use super::SplitOfASecond;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn flags_double_article_blends() {
        assert_suggestion_result(
            "The menu opens but closes in a split of a second.",
            SplitOfASecond::default(),
            "The menu opens but closes in a split second.",
        );
        assert_suggestion_result(
            "For the split of a second, I thought it was over.",
            SplitOfASecond::default(),
            "For the split second, I thought it was over.",
        );
        assert_suggestion_result(
            "In the split of the second, Java code is now running.",
            SplitOfASecond::default(),
            "In the split second, Java code is now running.",
        );
        assert_suggestion_result(
            "Every split of a second counts.",
            SplitOfASecond::default(),
            "Every split second counts.",
        );
        assert_suggestion_result(
            "The two phone calls arrived at the same split of a second.",
            SplitOfASecond::default(),
            "The two phone calls arrived at the same split second.",
        );
    }

    #[test]
    fn flags_dropped_outer_and_inner_articles() {
        assert_suggestion_result(
            "The dialog disappears after split of a second.",
            SplitOfASecond::default(),
            "The dialog disappears after a split second.",
        );
        assert_suggestion_result(
            "It flipped horizontally for split of a second.",
            SplitOfASecond::default(),
            "It flipped horizontally for a split second.",
        );
        assert_suggestion_result(
            "The control is rendered for a split of second during page load.",
            SplitOfASecond::default(),
            "The control is rendered for a split second during page load.",
        );
        assert_suggestion_result(
            "The layout flashes for split of second.",
            SplitOfASecond::default(),
            "The layout flashes for a split second.",
        );
        assert_suggestion_result(
            "That split of second was the only time one had.",
            SplitOfASecond::default(),
            "That split second was the only time one had.",
        );
    }

    #[test]
    fn flags_pluralization_slips() {
        assert_suggestion_result(
            "How we would react in a splits of a second!",
            SplitOfASecond::default(),
            "How we would react in a split second!",
        );
        assert_suggestion_result(
            "The client lost the connection only for a split of seconds.",
            SplitOfASecond::default(),
            "The client lost the connection only for a split second.",
        );
        assert_suggestion_result(
            "The printer pauses for splits of seconds while still extruding filament.",
            SplitOfASecond::default(),
            "The printer pauses for split seconds while still extruding filament.",
        );
        assert_suggestion_result(
            "The printer pauses for splits of seconds while still extruding filament.",
            SplitOfASecond::default(),
            "The printer pauses for a split second while still extruding filament.",
        );
    }

    #[test]
    fn flags_prenominal_compound_modifier() {
        assert_suggestion_result(
            "I get these split of second freezes.",
            SplitOfASecond::default(),
            "I get these split-second freezes.",
        );
        assert_suggestion_result(
            "I get these split of second freezes.",
            SplitOfASecond::default(),
            "I get these split second freezes.",
        );
    }

    #[test]
    fn ignores_cardiology_auscultation_terms() {
        assert_no_lints(
            "Quantitative Measurement of Split of the Second Heart Sound (S2)",
            SplitOfASecond::default(),
        );
        assert_no_lints(
            "Auscultation revealed a fixed split of the second sound.",
            SplitOfASecond::default(),
        );
        assert_no_lints(
            "Physiological split of the second disappears on expiration.",
            SplitOfASecond::default(),
        );
    }

    #[test]
    fn ignores_ordinal_noun_data_structures() {
        assert_no_lints(
            "How to find the number of splits of second array to create first array?",
            SplitOfASecond::default(),
        );
        assert_no_lints(
            "We compare splits of second half across runs.",
            SplitOfASecond::default(),
        );
    }

    #[test]
    fn ignores_title_cased_proper_nouns() {
        assert_no_lints(
            "We watched Split of a Second last night.",
            SplitOfASecond::default(),
        );
        assert_no_lints(
            "The racehorse Split Of A Second won by a length.",
            SplitOfASecond::default(),
        );
    }
}
