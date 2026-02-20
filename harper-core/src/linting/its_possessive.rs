use std::sync::Arc;

use harper_brill::UPOS;

use crate::Token;
use crate::expr::AnchorStart;
use crate::expr::Expr;
use crate::expr::ExprMap;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::UPOSSet;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct ItsPossessive {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for ItsPossessive {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let adj_term_mid_sentence = SequenceExpr::default()
            .t_ws()
            .then(UPOSSet::new(&[UPOS::ADJ]));

        let mid_sentence = SequenceExpr::with(UPOSSet::new(&[UPOS::VERB, UPOS::ADP]))
            .t_ws()
            .t_aco("it's")
            .then_optional(adj_term_mid_sentence)
            .t_ws()
            .then(
                UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]).or(|tok: &Token, _: &[char]| {
                    tok.kind.as_number().is_some_and(|n| n.suffix.is_some())
                }),
            );

        map.insert(mid_sentence, 2);

        let start_of_sentence_noun = SequenceExpr::with(AnchorStart)
            .t_aco("it's")
            .t_ws()
            .then(
                UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]).or(|tok: &Token, _: &[char]| {
                    tok.kind.as_number().is_some_and(|n| n.suffix.is_some())
                }),
            )
            .then_unless(SequenceExpr::default().t_ws().then(UPOSSet::new(&[
                UPOS::VERB,
                UPOS::PART,
                UPOS::ADP,
                UPOS::NOUN,
                UPOS::PRON,
                UPOS::SCONJ,
                UPOS::CCONJ,
                UPOS::ADV,
            ])));

        map.insert(start_of_sentence_noun, 0);

        let start_of_sentence_adjective = SequenceExpr::with(AnchorStart)
            .t_aco("it's")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::ADJ]))
            .t_ws()
            .then_unless(UPOSSet::new(&[
                UPOS::VERB,
                UPOS::PART,
                UPOS::ADP,
                UPOS::NOUN,
                UPOS::PRON,
                UPOS::SCONJ,
                UPOS::CCONJ,
                UPOS::ADV,
            ]));

        map.insert(start_of_sentence_adjective, 0);

        let start_of_chunk_after_conjunction = SequenceExpr::with(AnchorStart)
            .then(UPOSSet::new(&[UPOS::CCONJ]))
            .t_ws()
            .t_aco("it's")
            .t_ws()
            .then(
                UPOSSet::new(&[UPOS::NOUN, UPOS::PROPN]).or(|tok: &Token, _: &[char]| {
                    tok.kind.as_number().is_some_and(|n| n.suffix.is_some())
                }),
            )
            .then_unless(SequenceExpr::default().t_ws().then(UPOSSet::new(&[
                UPOS::VERB,
                UPOS::PART,
                UPOS::ADP,
                UPOS::NOUN,
                UPOS::PRON,
                UPOS::SCONJ,
                UPOS::CCONJ,
                UPOS::ADV,
            ])));

        map.insert(start_of_chunk_after_conjunction, 2);

        let special = SequenceExpr::aco("it's").t_ws().t_aco("various");

        map.insert(special, 0);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for ItsPossessive {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = self.map.lookup(0, matched_tokens, source).unwrap();
        let span = matched_tokens[*offending_idx].span;

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "its",
                span.get_content(source),
            )],
            message: "Use the possessive pronoun `its` (without an apostrophe) to show ownership. The word `it's` (with an apostrophe) is a contraction of 'it is' or 'it has' and should not be used to indicate possession.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "In English, possessive pronouns never take an apostrophe. Use `its` to show ownership (e.g. “its texture”) and avoid confusing it with `it's`, which always means “it is” or “it has.”"
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    use super::ItsPossessive;

    #[test]
    fn corrects_its_various() {
        assert_suggestion_result(
            "I like it's various colors.",
            ItsPossessive::default(),
            "I like its various colors.",
        );
    }

    #[test]
    fn fixes_inspiration() {
        assert_suggestion_result(
            "I would just put `Orthography` and it's various function implementations in their own `orthography.rs` file.",
            ItsPossessive::default(),
            "I would just put `Orthography` and its various function implementations in their own `orthography.rs` file.",
        );
    }

    #[test]
    fn engine_lost_its_compression() {
        assert_lint_count(
            "The engine lost it's compression.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn admired_sculpture_for_its_intricacy() {
        assert_suggestion_result(
            "I admired the sculpture for it's intricacy.",
            ItsPossessive::default(),
            "I admired the sculpture for its intricacy.",
        );
    }

    #[test]
    fn paris_is_known_for_its_architecture() {
        assert_lint_count(
            "Paris is known for it's architecture.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn plain_sentence_with_apostrophe_s() {
        assert_suggestion_result(
            "It's benefits are numerous.",
            ItsPossessive::default(),
            "Its benefits are numerous.",
        );
    }

    #[test]
    fn device_reached_its_100th_cycle() {
        assert_lint_count(
            "The device reached it's 100th cycle.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn oddly_its_wheels_misaligned() {
        assert_lint_count(
            "Oddly, it's wheels were misaligned.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn leaking_oil_constant_issue() {
        assert_lint_count("It's leaking oil constantly.", ItsPossessive::default(), 0);
    }

    #[test]
    fn fiftyth_anniversary() {
        assert_lint_count(
            "The company celebrated it's 50th anniversary.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn second_attempt() {
        assert_lint_count("He failed it's 2nd attempt.", ItsPossessive::default(), 1);
    }

    #[test]
    fn third_iteration() {
        assert_lint_count(
            "The program finished it's 3rd iteration.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn tenth_milestone() {
        assert_lint_count(
            "They reached it's 10th milestone.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn seventh_chapter() {
        assert_lint_count(
            "The novel lost it's 7th chapter.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn fifth_version() {
        assert_lint_count(
            "Software updated to it's 5th version.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn eighth_floor() {
        assert_lint_count(
            "Elevator stopped at it's 8th floor.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn twelfth_episode() {
        assert_lint_count(
            "Series ended it's 12th episode.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn fourth_draft() {
        assert_lint_count("He completed it's 4th draft.", ItsPossessive::default(), 1);
    }

    #[test]
    fn ninth_revision() {
        assert_lint_count(
            "The report saved it's 9th revision.",
            ItsPossessive::default(),
            1,
        );
    }

    #[test]
    fn allows_hard_to_tell() {
        assert_no_lints("It's hard to tell from here.", ItsPossessive::default());
    }

    #[test]
    fn allows_illegible() {
        assert_no_lints(
            "When you write in cursive, its illegible",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allows_good_practice() {
        assert_no_lints(
            "it's good practice to review the general settings",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allows_understandable() {
        assert_no_lints(
            "It's understandable that you'd feel the weight of responsibility.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allows_insincere() {
        assert_no_lints(
            "But feel free to omit it if you feel it's insincere.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allows_its_possible() {
        assert_no_lints(
            "It's possible that a record was improperly handled. ",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allows_many_times_harder() {
        assert_no_lints(
            "It's many times harder to do this than that.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allow_issue_1658() {
        assert_no_lints(
            "It's kind of a nuisance, but it will work.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn allow_issue_2001() {
        assert_no_lints(
            "It's worth highlighting that while using a fork instead of a spoon is easy, it sometimes isn't.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn dont_flag_issue_1722_its_whats_accessible() {
        assert_no_lints(
            "The base execution context is the global execution context: it's what's accessible everywhere in your code.",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn dont_flag_issue_1722_its_early_and() {
        assert_no_lints(
            "it's early and there's plenty of room for novel and potentially",
            ItsPossessive::default(),
        );
    }

    #[test]
    fn dont_flag_issue_1722_its_big_enough() {
        assert_no_lints("It's big enough.", ItsPossessive::default());
    }

    #[test]
    fn flags_all_its_possessive_in_list() {
        assert_lint_count(
            "Understand it's code, it's values, and it's purpose.",
            ItsPossessive::default(),
            3,
        );
    }

    #[test]
    fn fixes_all_its_possessive_in_list() {
        assert_suggestion_result(
            "Understand it's code, it's values, and it's purpose.",
            ItsPossessive::default(),
            "Understand its code, its values, and its purpose.",
        );
    }
}
