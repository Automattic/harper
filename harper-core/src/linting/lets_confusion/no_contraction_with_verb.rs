use crate::TokenKind;
use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::{
    Token,
    linting::{Lint, LintKind, Suggestion},
};

use crate::linting::ExprLinter;
use crate::linting::expr_linter::Chunk;

/// See also:
/// harper-core/src/linting/compound_nouns/implied_ownership_compound_nouns.rs
/// harper-core/src/linting/lets_confusion/mod.rs
/// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
/// harper-core/src/linting/pronoun_contraction/should_contract.rs
pub struct NoContractionWithVerb {
    expr: Box<dyn Expr>,
}

impl Default for NoContractionWithVerb {
    fn default() -> Self {
        // Only tests "let".
        let let_ws = SequenceExpr::word_set(&["lets", "let"]).then_whitespace();

        let non_ing_verb = SequenceExpr::default().then_kind_is_but_isnt_any_of(
            TokenKind::is_verb,
            &[
                TokenKind::is_noun,
                TokenKind::is_adjective,
                TokenKind::is_verb_progressive_form,
            ] as &[_],
        );

        // Ambiguous word is a verb determined by heuristic of following word's part of speech
        // Tests the next two words after "let".
        let verb_due_to_following_pos = SequenceExpr::default()
            .then_verb()
            .then_whitespace()
            .then_kind_any(&[
                TokenKind::is_determiner,
                TokenKind::is_pronoun,
                TokenKind::is_conjunction,
            ] as &[_]);

        let let_then_verb = let_ws.then(LongestMatchOf::new(vec![
            Box::new(non_ing_verb),
            Box::new(verb_due_to_following_pos),
        ]));

        Self {
            expr: Box::new(let_then_verb),
        }
    }
}

impl ExprLinter for NoContractionWithVerb {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let (let_string, verb_string) = (
            matched_tokens[0].span.get_content_string(source),
            matched_tokens[2].span.get_content_string(source),
        );

        // "to let go" is a phrasal verb but "lets go" is quite a common mistake for "let's go"
        if let_string == "let" && verb_string == "go" {
            return None;
        }

        let problem_span = matched_tokens.first()?.span;
        let template = problem_span.get_content(source);

        Some(Lint {
            span: problem_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("let's", template),
                Suggestion::replace_with_match_case_str("let us", template),
            ],
            message: "To suggest an action, use 'let's' or 'let us'.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Checks for `lets` meaning `permits` when the context is about suggesting an action."
    }
}

#[cfg(test)]
mod tests {
    use super::NoContractionWithVerb;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // Correct unambiguous verb

    #[test]
    fn fix_lets_inspect() {
        assert_suggestion_result(
            "In the end lets inspect with git-blame the results.",
            NoContractionWithVerb::default(),
            "In the end let's inspect with git-blame the results.",
        );
    }

    // False positives where verb is also a noun

    #[test]
    fn dont_flag_let_chance() {
        assert_lint_count("Let chance decide", NoContractionWithVerb::default(), 0);
    }

    #[test]
    fn dont_flag_let_time() {
        assert_lint_count(
            "Let time granularity be parametrized",
            NoContractionWithVerb::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lets_staff() {
        assert_lint_count(
            "A plugin that backs up player's inventories and lets staff restore them or export it as a shulker.",
            NoContractionWithVerb::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lets_time() {
        assert_lint_count(
            "This is very different than demo recording, which just simulates a network level connection and lets time move at its own rate.",
            NoContractionWithVerb::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lets_play() {
        assert_lint_count(
            "Sometimes the umpire lets play continue",
            NoContractionWithVerb::default(),
            0,
        );
    }

    // False positives where verb is a gerund/past participle

    #[test]
    fn dont_flag_let_sleeping() {
        assert_lint_count(
            "Let sleeping logs lie.",
            NoContractionWithVerb::default(),
            0,
        );
    }

    // False positives where verb is also an adjective

    #[test]
    fn dont_flag_let_processed() {
        assert_lint_count(
            "Let processed response be a new structure analogous to server auction response.",
            NoContractionWithVerb::default(),
            0,
        );
    }

    // Correct disambiguated noun/verb by following determiner

    #[test]
    fn corrects_lets_make_this() {
        assert_suggestion_result(
            "Lets make this joke repo into one of the best.",
            NoContractionWithVerb::default(),
            "Let's make this joke repo into one of the best.",
        );
    }

    // Correct disambiguated verb by following pronoun

    #[test]
    fn corrects_lets_mock_them() {
        assert_suggestion_result(
            "Then lets mock them using Module._load based mocker.",
            NoContractionWithVerb::default(),
            "Then let's mock them using Module._load based mocker.",
        );
    }

    // False positives / edge cases filed on GitHub

    #[test]
    fn dont_flag_let_us() {
        assert_lint_count("Let us do this.", NoContractionWithVerb::default(), 0);
    }

    #[test]
    fn dont_flag_let_go_1202() {
        assert_lint_count(
            "... until you hit your opponent, then let go and quickly retap",
            NoContractionWithVerb::default(),
            0,
        );
    }

    // False positive wrongly flagged by previous version of this linter

    #[test]
    fn dont_flag_let_in_and() {
        assert_lint_count(
            "Japanese is good enough to be let in and.",
            NoContractionWithVerb::default(),
            0,
        );
    }
}
