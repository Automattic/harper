use harper_brill::UPOS;

use crate::linting::expr_linter::Sentence;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::UPOSSet,
};

pub struct WereWhere {
    expr: SequenceExpr,
}

impl Default for WereWhere {
    fn default() -> Self {
        // === where → were ===

        // "they/we" are unambiguous plural subject pronouns — "where" directly after
        // them is almost certainly a typo for "were".
        // e.g. "they where going" → "they were going"
        let unambiguous_pronoun_where = SequenceExpr::word_set(&["they", "we"])
            .t_ws()
            .t_aco("where");

        // "you where" alone is ambiguous ("I'll show you where to go"), so only flag
        // it when followed by a verb, auxiliary, or adjective — confirming a verb slot.
        // e.g. "you where going" → "you were going"
        let you_where_verb = SequenceExpr::aco("you")
            .t_ws()
            .t_aco("where")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::VERB, UPOS::AUX, UPOS::ADJ]));

        // "where you ..." can be a typo for "were you ..." when it starts a question.
        let where_you_verb = SequenceExpr::aco("where")
            .t_ws()
            .t_aco("you")
            .t_ws()
            .then(UPOSSet::new(&[UPOS::VERB, UPOS::AUX, UPOS::ADJ]));

        // === were → where ===

        // A verb of cognition or motion followed directly by "were" and then a
        // pronoun, determiner, or proper noun indicates the start of a relative or
        // indirect question — where "were" should be "where".
        // e.g. "I know were they went"  → "I know where they went"
        // e.g. "I found were the book was" → "I found where the book was"
        //
        // "they were going" does NOT match: "they" (PRON) precedes "were", not VERB.
        // "I think they were going" does NOT match: "they" sits between "think" and "were".
        let verb_were_clause =
            SequenceExpr::with(|tok: &Token, _: &[char]| tok.kind.is_upos(UPOS::VERB))
                .t_ws()
                .t_aco("were")
                .t_ws()
                .then(UPOSSet::new(&[UPOS::PRON, UPOS::DET, UPOS::PROPN]));

        Self {
            expr: SequenceExpr::any_of(vec![
                Box::new(unambiguous_pronoun_where),
                Box::new(you_where_verb),
                Box::new(where_you_verb),
                Box::new(verb_were_clause),
            ]),
        }
    }
}

impl ExprLinter for WereWhere {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        const WHERE: &[char] = &['w', 'h', 'e', 'r', 'e'];
        const WERE: &[char] = &['w', 'e', 'r', 'e'];

        // Check if "where" appears in the match (where → were case)
        let where_tok = toks.iter().find(|tok| {
            matches!(tok.kind, TokenKind::Word(_)) && tok.span.get_content(src).eq_ch(WHERE)
        });

        // Check if "were" appears in the match (were → where case)
        let were_tok = toks.iter().find(|tok| {
            matches!(tok.kind, TokenKind::Word(_)) && tok.span.get_content(src).eq_ch(WERE)
        });

        if let Some(tok) = where_tok {
            if !crate::linting::expr_linter::at_start_of_sentence(context) {
                return None;
            }
            Some(Lint {
                span: tok.span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "were",
                    tok.span.get_content(src),
                )],
                message: "It looks like this is a typo, did you mean `were`?".to_string(),
                ..Default::default()
            })
        } else {
            were_tok.map(|tok| Lint {
                span: tok.span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "where",
                    tok.span.get_content(src),
                )],
                message: "It looks like this is a typo, did you mean `where`?".to_string(),
                ..Default::default()
            })
        }
    }

    fn description(&self) -> &'static str {
        "Detects mixing up `were` and `where`."
    }
}

#[cfg(test)]
mod tests {
    use super::WereWhere;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // ── where → were: unambiguous pronouns ──────────────────────────────────

    #[test]
    fn fix_they_where() {
        assert_suggestion_result(
            "They where going to the store.",
            WereWhere::default(),
            "They were going to the store.",
        );
    }

    #[test]
    fn fix_we_where() {
        assert_suggestion_result(
            "We where right about that.",
            WereWhere::default(),
            "We were right about that.",
        );
    }

    #[test]
    fn fix_they_where_happy() {
        assert_suggestion_result(
            "They where happy with the result.",
            WereWhere::default(),
            "They were happy with the result.",
        );
    }

    // ── where → were: "you where" with a following verb ─────────────────────

    #[test]
    fn fix_you_where_going() {
        assert_suggestion_result(
            "you where going in the right direction.",
            WereWhere::default(),
            "you were going in the right direction.",
        );
    }

    #[test]
    fn fix_you_where_right() {
        assert_suggestion_result(
            "you where right about that.",
            WereWhere::default(),
            "you were right about that.",
        );
    }

    // ── were → where: verb + were + pronoun/determiner ──────────────────────

    #[test]
    fn fix_know_were_they() {
        assert_suggestion_result(
            "Do you know were they went?",
            WereWhere::default(),
            "Do you know where they went?",
        );
    }

    #[test]
    fn fix_forgot_were_i() {
        assert_suggestion_result(
            "I forgot were I put my keys.",
            WereWhere::default(),
            "I forgot where I put my keys.",
        );
    }

    #[test]
    fn fix_found_were_the() {
        assert_suggestion_result(
            "I found were the book was.",
            WereWhere::default(),
            "I found where the book was.",
        );
    }

    #[test]
    fn fix_go_were_they() {
        assert_suggestion_result(
            "Go were they tell you.",
            WereWhere::default(),
            "Go where they tell you.",
        );
    }

    // ── where → were: more they/we variants ─────────────────────────────────

    #[test]
    fn fix_we_where_almost_done() {
        // No following-word check needed for "we/they" — the pair alone is enough
        assert_suggestion_result(
            "We where almost done with the task.",
            WereWhere::default(),
            "We were almost done with the task.",
        );
    }

    #[test]
    fn fix_they_where_able() {
        assert_suggestion_result(
            "They where able to fix the issue in time.",
            WereWhere::default(),
            "They were able to fix the issue in time.",
        );
    }

    #[test]
    fn fix_we_where_told() {
        assert_suggestion_result(
            "We where told about the change last week.",
            WereWhere::default(),
            "We were told about the change last week.",
        );
    }

    #[test]
    fn fix_they_where_supposed() {
        assert_suggestion_result(
            "They where supposed to be here by now.",
            WereWhere::default(),
            "They were supposed to be here by now.",
        );
    }

    // ── where → were: more "you where" variants ──────────────────────────────

    #[test]
    fn fix_you_where_supposed() {
        // "supposed" is ADJ — confirms verb slot
        assert_suggestion_result(
            "You where supposed to call me.",
            WereWhere::default(),
            "You were supposed to call me.",
        );
    }

    #[test]
    fn fix_you_where_asked() {
        // "asked" past participle used as VERB
        assert_suggestion_result(
            "you where asked to leave the room.",
            WereWhere::default(),
            "you were asked to leave the room.",
        );
    }

    #[test]
    fn fix_where_you_able() {
        assert_suggestion_result(
            "Where you able to make forward progress here?",
            WereWhere::default(),
            "Were you able to make forward progress here?",
        );
    }

    // ── were → where: more verbs and pronouns ────────────────────────────────

    #[test]
    fn fix_remember_were_i() {
        assert_suggestion_result(
            "Do you remember were I left the keys?",
            WereWhere::default(),
            "Do you remember where I left the keys?",
        );
    }

    #[test]
    fn fix_check_were_the() {
        assert_suggestion_result(
            "Check were the error occurred.",
            WereWhere::default(),
            "Check where the error occurred.",
        );
    }

    #[test]
    fn fix_asked_were_he() {
        assert_suggestion_result(
            "She asked were he lived.",
            WereWhere::default(),
            "She asked where he lived.",
        );
    }

    #[test]
    fn fix_know_were_the_bug() {
        assert_suggestion_result(
            "I know were the bug is.",
            WereWhere::default(),
            "I know where the bug is.",
        );
    }

    #[test]
    fn fix_find_were_it() {
        assert_suggestion_result(
            "Find were it crashed.",
            WereWhere::default(),
            "Find where it crashed.",
        );
    }

    // ── no false positives ───────────────────────────────────────────────────

    #[test]
    fn no_flag_where_they_are() {
        assert_no_lints("Do you know where they are going?", WereWhere::default());
    }

    #[test]
    fn no_flag_they_were_going() {
        assert_no_lints("They were going to the store.", WereWhere::default());
    }

    #[test]
    fn no_flag_we_were_right() {
        assert_no_lints("We were right about that.", WereWhere::default());
    }

    #[test]
    fn no_flag_show_you_where() {
        // "you" before "where" is legitimate — followed by "to" (PART), not a verb
        assert_no_lints("I'll show you where to go.", WereWhere::default());
    }

    #[test]
    fn no_flag_tell_you_where_the() {
        // "you where" followed by DET — not flagged (DET is not VERB/AUX/ADJ)
        assert_no_lints("I'll tell you where the exit is.", WereWhere::default());
    }

    #[test]
    fn no_flag_they_were_wrong() {
        // "they" (PRON) precedes "were", so VERB + "were" pattern does not fire
        assert_no_lints("I think they were wrong.", WereWhere::default());
    }

    #[test]
    fn no_flag_confirmed_they_were() {
        // "they" sits between "confirmed" and "were" — not adjacent, no match
        assert_no_lints("I confirmed they were correct.", WereWhere::default());
    }

    #[test]
    fn no_flag_found_they_were() {
        assert_no_lints("He found they were missing.", WereWhere::default());
    }

    #[test]
    fn no_flag_where_were_they() {
        // "Where" is an adverb or subordinating conjunction here, not VERB — the were→where pattern does not fire
        assert_no_lints("Where were they going?", WereWhere::default());
    }

    #[test]
    fn no_flag_showed_me_where() {
        // Object pronoun "me" sits between "showed" and "where" — no direct adjacency
        assert_no_lints("He showed me where the exit was.", WereWhere::default());
    }

    #[test]
    fn no_flag_where_you_go() {
        assert_no_lints("I wonder where you go from here.", WereWhere::default());
    }

    #[test]
    fn no_flag_where_you_can_customize() {
        assert_no_lints(
            "Click the menu item where you can customize the settings.",
            WereWhere::default(),
        );
    }

    #[test]
    fn no_flag_where_you_allocate() {
        assert_no_lints(
            "Use the panel where you allocate resources for the task.",
            WereWhere::default(),
        );
    }

    // ── known limitations (documented but not yet handled) ───────────────────

    #[test]
    #[ignore = "limitation: 'you where' followed by DET is not flagged; would need DET in the following-word set"]
    fn fix_you_where_the_only_one() {
        assert_suggestion_result(
            "you where the only one there.",
            WereWhere::default(),
            "you were the only one there.",
        );
    }

    #[test]
    #[ignore = "limitation: sentence-initial 'Where' as typo for 'Were' is not handled"]
    fn fix_where_they_going_sentence_start() {
        assert_suggestion_result(
            "Where they going to the party?",
            WereWhere::default(),
            "Were they going to the party?",
        );
    }

    #[test]
    #[ignore = "limitation: indirect object between verb and 'were' is not detected"]
    fn fix_showed_me_were() {
        assert_suggestion_result(
            "He showed me were the exit was.",
            WereWhere::default(),
            "He showed me where the exit was.",
        );
    }
}
