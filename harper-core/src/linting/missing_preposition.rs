use harper_brill::UPOS;

use crate::expr::AnchorStart;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::AnyPattern;
use crate::patterns::UPOSSet;
use crate::{Token, TokenKind, TokenStringExt};

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
        // The missing-preposition construction is copular (subject + be-verb +
        // adjective + object). Keep the original AUX constraint while excluding
        // modal auxiliaries such as "can" and non-copular verbs such as "has".
        .then_kind_both(|kind| kind.is_upos(UPOS::AUX), TokenKind::is_linking_verb)
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

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        if matched_tokens.last()?.kind.is_upos(UPOS::ADP) {
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
    fn allows_issue_1585_public_examples() {
        // See https://github.com/Automattic/harper/issues/1585
        assert_no_lints(
            "Once the installation is done, you can open your project folder.",
            MissingPreposition::default(),
        );
        assert_no_lints(
            "It has several validation levels:",
            MissingPreposition::default(),
        );
        assert_no_lints(
            "You can close it if you prefer and if any of these show up again I'll reopen and for any new ones I'll open fresh issues.",
            MissingPreposition::default(),
        );
        assert_no_lints(
            "Her foot, that there was hardly room to open her mouth.",
            MissingPreposition::default(),
        );
    }
}
