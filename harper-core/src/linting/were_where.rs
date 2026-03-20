use harper_brill::UPOS;

use crate::linting::expr_linter::Chunk;
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
                Box::new(verb_were_clause),
            ]),
        }
    }
}

impl ExprLinter for WereWhere {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        const WHERE: &[char] = &['w', 'h', 'e', 'r', 'e'];
        const WERE: &[char] = &['w', 'e', 'r', 'e'];

        // Check if "where" appears in the match (where → were case)
        let where_tok = toks.iter().find(|tok| {
            matches!(tok.kind, TokenKind::Word(_))
                && tok.span.get_content(src).eq_ignore_ascii_case_chars(WHERE)
        });

        // Check if "were" appears in the match (were → where case)
        let were_tok = toks.iter().find(|tok| {
            matches!(tok.kind, TokenKind::Word(_))
                && tok.span.get_content(src).eq_ignore_ascii_case_chars(WERE)
        });

        if let Some(tok) = where_tok {
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
        } else if let Some(tok) = were_tok {
            Some(Lint {
                span: tok.span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "where",
                    tok.span.get_content(src),
                )],
                message: "It looks like this is a typo, did you mean `where`?".to_string(),
                ..Default::default()
            })
        } else {
            None
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

    // ── no false positives ───────────────────────────────────────────────────

    #[test]
    fn no_flag_where_they_are() {
        assert_no_lints(
            "Do you know where they are going?",
            WereWhere::default(),
        );
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
        // "you" before "where" is legitimate here — followed by "to" (PART), not a verb
        assert_no_lints("I'll show you where to go.", WereWhere::default());
    }

    #[test]
    fn no_flag_tell_you_where_the() {
        assert_no_lints(
            "I'll tell you where the exit is.",
            WereWhere::default(),
        );
    }

    #[test]
    fn no_flag_they_were_wrong() {
        assert_no_lints("I think they were wrong.", WereWhere::default());
    }
}
