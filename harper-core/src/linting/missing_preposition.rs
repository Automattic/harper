use harper_brill::UPOS;

use crate::expr::AnchorStart;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::AnyPattern;
use crate::patterns::UPOSSet;
use crate::{CharStringExt, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct MissingPreposition {
    expr: SequenceExpr,
}

impl Default for MissingPreposition {
    fn default() -> Self {
        let expr = SequenceExpr::with(
            AnchorStart.or_longest(
                SequenceExpr::default()
                    .then_non_quantifier_determiner()
                    .t_ws(),
            ),
        )
        .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PRON, UPOS::PROPN]))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::AUX]))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::ADJ]))
        .t_ws()
        .then(UPOSSet::new(&[UPOS::NOUN, UPOS::PRON, UPOS::PROPN]))
        .then_optional(AnyPattern)
        .then_optional(AnyPattern);

        Self { expr }
    }
}

impl ExprLinter for MissingPreposition {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.last()?.kind.is_upos(UPOS::ADP) {
            return None;
        }

        // Skip when the AUX slot is a form of "have" acting as a main verb
        // taking a direct object (e.g. "children has dual citizenship",
        // "she has big dreams"). The AUX tagger can't disambiguate main
        // "have" from auxiliary "have", so we do it here lexically.
        // Locate the AUX token in the matched sequence (the expression's
        // trailing AnyPattern optionals can push the AUX away from a
        // fixed offset, so search by UPOS rather than rely on an index).
        let aux_tok = matched_tokens.iter().find(|t| t.kind.is_upos(UPOS::AUX))?;
        if aux_tok.get_ch(source).eq_any_ignore_ascii_case_str(&[
            "has",
            "have",
            "had",
            "having",
            "hasn't",
            "haven't",
            "hadn't",
            // Typographical / curly apostrophe variants.
            "hasn\u{2019}t",
            "haven\u{2019}t",
            "hadn\u{2019}t",
        ]) {
            return None;
        }

        Some({
            Lint {
                span: matched_tokens[2..4].span()?,
                lint_kind: LintKind::Miscellaneous,
                suggestions: vec![],
                message: "You may be missing a preposition here.".to_owned(),
                priority: 31,
            }
        })
    }

    fn description(&self) -> &'static str {
        "Locates potentially missing prepositions."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    use super::MissingPreposition;

    #[test]
    fn fixes_issue_1513() {
        assert_lint_count(
            "The city is famous its beaches.",
            MissingPreposition::default(),
            1,
        );
        assert_lint_count(
            "The students are interested learning.",
            MissingPreposition::default(),
            1,
        );
    }

    #[test]
    fn allows_corrected_issue_1513() {
        assert_no_lints(
            "The city is famous for its beaches.",
            MissingPreposition::default(),
        );
        assert_no_lints(
            "The students are interested in learning.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn no_lint_without_adj_noun_sequence() {
        assert_lint_count("She is happy.", MissingPreposition::default(), 0);
    }

    #[test]
    fn no_lint_with_preposition_present() {
        assert_lint_count("They are fond of music.", MissingPreposition::default(), 0);
        assert_lint_count(
            "Students are interested in history.",
            MissingPreposition::default(),
            0,
        );
    }

    #[test]
    fn flag_adj_pron_pair() {
        assert_lint_count("He was angry him.", MissingPreposition::default(), 1);
    }

    #[test]
    fn no_lint_empty() {
        assert_lint_count("", MissingPreposition::default(), 0);
    }

    #[test]
    fn allows_tired_herself() {
        assert_no_lints(
            "She had tired herself out with trying.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn allows_terrible_stuff() {
        assert_no_lints(
            "Either it was terrible stuff or the whiskey distorted things.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn allows_issue_1585() {
        assert_no_lints(
            "Each agent has specific tools and tasks orchestrated through a crew workflow.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn allows_one_of_my_children_has_dual_citizenship() {
        // Regression for the issue: main-verb "has" + adj + noun should
        // not be flagged. The AUX slot here is "has" acting as a main
        // verb taking the noun phrase "dual citizenship" as its object.
        assert_no_lints(
            "I am Australian, and one of my children has dual citizenship.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn allows_main_verb_haven_t_with_straight_apostrophe() {
        // Negated main verb "haven't" still takes a direct object.
        assert_no_lints(
            "Most of the new tickets haven't full coverage.",
            MissingPreposition::default(),
        );
    }

    #[test]
    fn allows_main_verb_haven_t_with_curly_apostrophe() {
        // Same case but with the U+2019 right single quotation mark,
        // which most editors and macOS smart-quote substitution produce.
        assert_no_lints(
            "Most of the new tickets haven\u{2019}t full coverage.",
            MissingPreposition::default(),
        );
    }
}
