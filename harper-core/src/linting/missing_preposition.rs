use harper_brill::UPOS;

use crate::expr::AnchorStart;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::AnyPattern;
use crate::patterns::UPOSSet;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

/// Predicate adjectives that idiomatically take a prepositional complement, so a
/// bare noun phrase right after them signals a missing preposition. Restricting
/// the rule to these (rather than any adjective) is what keeps it from
/// over-firing on attributive adjective-noun phrases ("live hedgehogs") and
/// clause-heading adjectives ("sure he'd start"), which are structurally
/// identical to the real errors. Trade-off: the rule only flags listed
/// adjectives — precision over recall, deliberately.
const PP_TAKING_ADJECTIVES: &[&str] = &[
    "afraid", "angry", "annoyed", "anxious", "ashamed", "aware", "capable",
    "certain", "characteristic", "confident", "conscious", "critical", "curious",
    "dependent", "desirous", "devoid", "envious", "fearful", "fond", "famous",
    "free", "full", "glad", "guilty", "ignorant", "incapable", "independent",
    "indicative", "interested", "jealous", "keen", "mindful", "nervous",
    "oblivious", "passionate", "proud", "reminiscent", "representative",
    "resentful", "respectful", "responsible", "scared", "sceptical", "skeptical",
    "suspicious", "terrified", "thankful", "tired", "tolerant", "typical",
    "unaware", "wary", "weary", "worried", "worthy",
];

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

        // The predicate adjective must be one that idiomatically takes a
        // prepositional complement (interested IN, famous FOR, angry AT, afraid
        // OF). A general descriptive adjective merely modifies the following noun
        // attributively ("live hedgehogs", "terrible stuff") or heads a clause
        // ("sure he'd start") — neither is a missing-preposition error. Matching
        // the adjective against this set is what separates the real errors from
        // the structurally-identical false positives the bare pattern produces.
        let adj_idx = matched_tokens
            .iter()
            .position(|t| t.kind.is_upos(UPOS::ADJ))?;
        let adj = matched_tokens[adj_idx].get_str(source).to_lowercase();
        if !PP_TAKING_ADJECTIVES.contains(&adj.as_str()) {
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
}
