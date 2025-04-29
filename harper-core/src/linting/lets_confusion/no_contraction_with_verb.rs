use crate::{
    Token,
    linting::{Lint, LintKind, Suggestion},
    patterns::{EitherPattern, Pattern, SequencePattern, WordSet},
};

use crate::linting::PatternLinter;

/// See also:
/// harper-core/src/linting/compound_nouns/implied_ownership_compound_nouns.rs
/// harper-core/src/linting/lets_confusion/mod.rs
/// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
/// harper-core/src/linting/pronoun_contraction/should_contract.rs
pub struct NoContractionWithVerb {
    pattern: Box<dyn Pattern>,
}

impl Default for NoContractionWithVerb {
    fn default() -> Self {
        // Only tests "let".
        let let_ws = SequencePattern::default()
            .then(WordSet::new(&["lets", "let"]))
            .then_whitespace();

        // Word is only a verb, and not the gerund/present participle form.
        // Only tests the next word after "let".
        let non_ing_verb = SequencePattern::default().then(|tok: &Token, source: &[char]| {
            let Some(Some(meta)) = tok.kind.as_word() else {
                return false;
            };

            if !meta.is_verb() || meta.is_noun() || meta.is_adjective() {
                return false;
            }

            let tok_chars = tok.span.get_content(source);

            // If it ends with 'ing' and is at least 5 chars long, it could be a gerund or past participle
            // TODO: replace with metadata check when affix system supports verb forms
            if tok_chars.len() < 5 {
                return true;
            }

            let is_ing_form = tok_chars
                .iter()
                .skip(tok_chars.len() - 3)
                .map(|&c| c.to_ascii_lowercase())
                .collect::<Vec<_>>()
                .ends_with(&['i', 'n', 'g']);

            !is_ing_form
        });

        // Ambiguous word is a verb determined by heuristic of following word's part of speech
        // Tests the next two words after "let".
        let verb_due_to_following_pos = SequencePattern::default()
            .then(|tok: &Token, source: &[char]| {
                tok.kind.is_verb()
                // TODO: because 'US' is a noun, 'us' also gets marked as a noun
                || (tok.kind.is_noun() && tok.span.get_content(source) != ['u', 's'])
            })
            .then_whitespace()
            .then(|tok: &Token, _source: &[char]| {
                tok.kind.is_determiner() || tok.kind.is_pronoun() || tok.kind.is_conjunction()
            });

        let let_then_verb = let_ws.then(EitherPattern::new(vec![
            Box::new(non_ing_verb),
            Box::new(verb_due_to_following_pos),
        ]));

        Self {
            pattern: Box::new(let_then_verb),
        }
    }
}

impl PatternLinter for NoContractionWithVerb {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
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

    // Corrections

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

    // Disambiguated noun/verb by following determiner

    #[test]
    fn corrects_lets_make_this() {
        assert_suggestion_result(
            "Lets make this joke repo into one of the best.",
            NoContractionWithVerb::default(),
            "Let's make this joke repo into one of the best.",
        );
    }

    // Disambiguated verb by following pronoun

    #[test]
    fn corrects_lets_mock_them() {
        assert_suggestion_result(
            "Then lets mock them using Module._load based mocker.",
            NoContractionWithVerb::default(),
            "Then let's mock them using Module._load based mocker.",
        );
    }

    // False positives / edge cases files on GitHub

    #[test]
    fn dont_flag_let_us() {
        assert_lint_count("Let us do this.", NoContractionWithVerb::default(), 0);
    }
}
