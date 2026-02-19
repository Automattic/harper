use super::{ExprLinter, Lint, LintKind};
use crate::expr::{All, Expr, FirstMatchOf, FixedPhrase, SequenceExpr};
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::patterns::{Invert, Word, WordSet};
use crate::{CharStringExt, Token, TokenKind};

/// Corrects the misuse of `then` to `than`.
pub struct ThenThan {
    expr: Box<dyn Expr>,
}

impl ThenThan {
    pub fn new() -> Self {
        let comparison = All::new(vec![
            Box::new(FirstMatchOf::new(vec![
                // Comparative form of adjective
                Box::new(
                    SequenceExpr::with(Box::new(|tok: &Token, source: &[char]| {
                        is_comparative(tok, source)
                    }))
                    .t_ws()
                    .t_aco("then")
                    .t_ws()
                    .then_unless(Word::new("that")),
                ),
                // Positive form of adjective following "more" or "less"
                Box::new(
                    SequenceExpr::word_set(&["more", "less"])
                        .t_ws()
                        .then_kind_either(TokenKind::is_adjective, TokenKind::is_adverb)
                        .t_ws()
                        .t_aco("then")
                        .t_ws()
                        .then_unless(Word::new("that")),
                ),
            ])),
            // Exceptions to the rule.
            Box::new(Invert::new(WordSet::new(&["back", "this", "so", "but"]))),
        ]);

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(comparison),
                Box::new(FixedPhrase::from_phrase("easier said then done")),
                Box::new(FixedPhrase::from_phrase("now and than")),
                Box::new(FixedPhrase::from_phrase("other then")),
                Box::new(FixedPhrase::from_phrase("rather then")),
                Box::new(FixedPhrase::from_phrase("than again")),
                Box::new(FixedPhrase::from_phrase("until than")),
            ])),
        }
    }
}

fn is_comparative(tok: &Token, source: &[char]) -> bool {
    tok.kind.is_comparative_adjective()
        || tok
            .span
            .get_content(source)
            .eq_any_ignore_ascii_case_chars(&[&['l', 'e', 's', 's'], &['m', 'o', 'r', 'e']])
}

impl Default for ThenThan {
    fn default() -> Self {
        Self::new()
    }
}

impl ExprLinter for ThenThan {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut thans_and_thens = matched_tokens.iter().filter(|tok| {
            tok.span
                .get_content(source)
                .eq_any_ignore_ascii_case_chars(&[&['t', 'h', 'a', 'n'], &['t', 'h', 'e', 'n']])
        });

        // Get the first match and ensure there's exactly one
        let span = match (thans_and_thens.next(), thans_and_thens.next()) {
            (Some(token), None) => token.span,
            _ => return None,
        };

        let offending_text = span.get_content(source);

        let new_text = if offending_text.eq_ignore_ascii_case_chars(&['t', 'h', 'e', 'n']) {
            "than"
        } else {
            "then"
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                new_text.chars().collect(),
                offending_text,
            )],
            message: format!("Did you mean `{new_text}`?"),
            priority: 31,
        })
    }
    fn description(&self) -> &'static str {
        "Corrects mixing up `then` and `than`."
    }
}

#[cfg(test)]
mod tests {
    use super::ThenThan;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn allows_back_then() {
        assert_lint_count("I was a gross kid back then.", ThenThan::default(), 0);
    }

    #[test]
    fn catches_shorter_then() {
        assert_suggestion_result(
            "One was shorter then the other.",
            ThenThan::default(),
            "One was shorter than the other.",
        );
    }

    #[test]
    fn catches_better_then() {
        assert_suggestion_result(
            "One was better then the other.",
            ThenThan::default(),
            "One was better than the other.",
        );
    }

    #[test]
    fn catches_longer_then() {
        assert_suggestion_result(
            "One was longer then the other.",
            ThenThan::default(),
            "One was longer than the other.",
        );
    }

    #[test]
    fn catches_less_then() {
        assert_suggestion_result(
            "I eat less then you.",
            ThenThan::default(),
            "I eat less than you.",
        );
    }

    #[test]
    fn catches_more_then() {
        assert_suggestion_result(
            "I eat more then you.",
            ThenThan::default(),
            "I eat more than you.",
        );
    }

    #[test]
    fn stronger_should_change() {
        assert_suggestion_result(
            "a chain is no stronger then its weakest link",
            ThenThan::default(),
            "a chain is no stronger than its weakest link",
        );
    }

    #[test]
    fn half_a_loaf_should_change() {
        assert_suggestion_result(
            "half a loaf is better then no bread",
            ThenThan::default(),
            "half a loaf is better than no bread",
        );
    }

    #[test]
    fn then_everyone_clapped_should_be_allowed() {
        assert_lint_count("and then everyone clapped", ThenThan::default(), 0);
    }

    #[test]
    fn crazier_than_rat_should_change() {
        assert_suggestion_result(
            "crazier then a shithouse rat",
            ThenThan::default(),
            "crazier than a shithouse rat",
        );
    }

    #[test]
    fn poke_in_eye_should_change() {
        assert_suggestion_result(
            "better then a poke in the eye with a sharp stick",
            ThenThan::default(),
            "better than a poke in the eye with a sharp stick",
        );
    }

    #[test]
    fn other_then_should_change() {
        assert_suggestion_result(
            "There was no one other then us at the campsite.",
            ThenThan::default(),
            "There was no one other than us at the campsite.",
        );
    }

    #[test]
    fn allows_and_then() {
        assert_lint_count("And then we left.", ThenThan::default(), 0);
    }

    #[test]
    fn allows_this_then() {
        assert_lint_count("Do this then that.", ThenThan::default(), 0);
    }

    #[test]
    fn allows_issue_720() {
        assert_lint_count(
            "And if just one of those is set incorrectly or it has the tiniest bit of dirt inside then that will wreak havoc with the engine's running ability.",
            ThenThan::default(),
            0,
        );
        assert_lint_count("So let's check it out then.", ThenThan::default(), 0);
        assert_lint_count(
            "And if just the tiniest bit of dirt gets inside then that will wreak havoc.",
            ThenThan::default(),
            0,
        );

        assert_lint_count(
            "He was always a top student in school but then his argument is that grades don't define intelligence.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn allows_issue_744() {
        assert_lint_count(
            "So then after talking about how he would, he didn't.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn issue_720_school_but_then_his() {
        assert_lint_count(
            "She loved the atmosphere of the school but then his argument is that it lacks proper resources for students.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "The teacher praised the efforts of the school but then his argument is that the curriculum needs to be updated.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "They were excited about the new program at school but then his argument is that it won't be effective without proper training.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "The community supported the school but then his argument is that funding is still a major issue.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn issue_720_so_then_these_resistors() {
        assert_lint_count(
            "So then these resistors are connected up in parallel to reduce the overall resistance.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "So then these resistors are connected up to ensure the current flows properly.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "So then these resistors are connected up to achieve the desired voltage drop.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "So then these resistors are connected up to demonstrate the principles of series and parallel circuits.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "So then these resistors are connected up to optimize the circuit's performance.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn issue_720_yes_so_then_sorry() {
        assert_lint_count(
            "Yes so then sorry you didn't receive the memo about the meeting changes.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "Yes so then sorry you had to wait so long for a response from our team.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "Yes so then sorry you felt left out during the discussion; we value your input.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "Yes so then sorry you missed the deadline; we can discuss an extension.",
            ThenThan::default(),
            0,
        );
        assert_lint_count(
            "Yes so then sorry you encountered issues with the software; let me help you troubleshoot.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn more_talented_then_her_issue_720() {
        assert_suggestion_result(
            "He was more talented then her at writing code.",
            ThenThan::default(),
            "He was more talented than her at writing code.",
        );
    }

    #[test]
    fn simpler_then_hers_issue_720() {
        assert_suggestion_result(
            "The design was simpler then hers in layout and color scheme.",
            ThenThan::default(),
            "The design was simpler than hers in layout and color scheme.",
        );
    }

    #[test]
    fn earlier_then_him_issue_720() {
        assert_suggestion_result(
            "We arrived earlier then him at the event.",
            ThenThan::default(),
            "We arrived earlier than him at the event.",
        );
    }

    #[test]
    fn more_robust_then_his_issue_720() {
        assert_suggestion_result(
            "This approach is more robust then his for handling edge cases.",
            ThenThan::default(),
            "This approach is more robust than his for handling edge cases.",
        );
    }

    #[test]
    fn patch_more_recently_then_last_week_issue_720() {
        assert_suggestion_result(
            "We submitted the patch more recently then last week, so they should have it already.",
            ThenThan::default(),
            "We submitted the patch more recently than last week, so they should have it already.",
        );
    }

    #[test]
    fn allows_well_then() {
        assert_lint_count(
            "Well then we're just going to raise all of these taxes",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn allows_nervous_then() {
        assert_lint_count(
            "I think both of us were getting nervous then because the system would have automatically aborted.",
            ThenThan::default(),
            0,
        );
    }

    #[test]
    fn flags_stupider_then_and_more_and_less_stupid_then() {
        assert_lint_count(
            "He was stupider then her but she was more stupid then some. Then again he was less stupid then some too.",
            ThenThan::default(),
            3,
        );
    }

    #[test]
    fn patch_worse_then() {
        assert_suggestion_result(
            "He was worse then her at writing code.",
            ThenThan::default(),
            "He was worse than her at writing code.",
        );
    }

    #[test]
    fn patch_rather_then() {
        assert_suggestion_result(
            "If copy-paste has to be prevented, I'd prefer it if paste rather then copy would be disabled",
            ThenThan::default(),
            "If copy-paste has to be prevented, I'd prefer it if paste rather than copy would be disabled",
        );
    }

    #[test]
    fn patch_easier_said_then_done() {
        assert_suggestion_result(
            "This is currently easier said then done because you cannot press Ctrl+A in the debug console",
            ThenThan::default(),
            "This is currently easier said than done because you cannot press Ctrl+A in the debug console",
        );
    }

    #[test]
    fn patch_every_now_and_than() {
        assert_suggestion_result(
            "I was testing every now and than after an upgrade on the home assistant plugin.",
            ThenThan::default(),
            "I was testing every now and then after an upgrade on the home assistant plugin.",
        );
    }

    #[test]
    fn patch_until_than() {
        assert_suggestion_result(
            "For the case anyone else ever hits this and the problem is not solved until than, this is a working workaround for the problem",
            ThenThan::default(),
            "For the case anyone else ever hits this and the problem is not solved until then, this is a working workaround for the problem",
        );
    }

    #[test]
    fn patch_now_and_than() {
        assert_suggestion_result(
            "sounds good if golang-set becomes an issue between now and than…just let me know!",
            ThenThan::default(),
            "sounds good if golang-set becomes an issue between now and then…just let me know!",
        );
    }
}
