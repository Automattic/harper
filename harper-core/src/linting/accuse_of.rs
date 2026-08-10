use crate::{
    CharStringExt, Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct AccuseOf {
    expr: All,
}

impl Default for AccuseOf {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::optional(SequenceExpr::any_word().t_ws())
                .t_set(&["accuse", "accused", "accuses", "accusing"])
                .then_optional(SequenceExpr::whitespace().then_object_pronoun())
                .t_ws()
                .t_aco("for")
                .but_not(SequenceExpr::word_seq(&["the", "accused"])),
        }
    }
}

impl ExprLinter for AccuseOf {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((_, after)) = ctx
            && let [ws, the, ws2, nth, ws3, time, ..] = after
            && ws.kind.is_whitespace()
            && the.kind.is_word()
            && the.get_ch(src).eq_str("the")
            && ws2.kind.is_whitespace()
            && (nth.kind.is_ordinal_number()
                || nth.kind.is_word()
                    && nth.get_ch(src).eq_any_ignore_ascii_case_str(&[
                        "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
                        "eighth", "ninth", "tenth",
                    ]))
            && ws3.kind.is_whitespace()
            && time.kind.is_word()
            && time.get_ch(src).eq_str("time")
        {
            return None;
        }

        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of",
                span.get_content(src),
            )],
            message: "The correct preposition is `of`, not `for`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `accuse for` to `accuse of`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AccuseOf;

    #[test]
    fn fix_accused_for_expressing() {
        assert_suggestion_result(
            "he's been accused for expressing personal opinions, on his personal blog.",
            AccuseOf::default(),
            "he's been accused of expressing personal opinions, on his personal blog.",
        );
    }

    #[test]
    fn fix_accused_for_at_end() {
        assert_suggestion_result(
            "That's not what they were accused for.",
            AccuseOf::default(),
            "That's not what they were accused of.",
        );
    }

    #[test]
    fn fix_accused_them_for() {
        assert_suggestion_result(
            "nobody accused them for being rational",
            AccuseOf::default(),
            "nobody accused them of being rational",
        );
    }

    #[test]
    fn fix_accused_you_of() {
        assert_suggestion_result(
            "I have never accused you for placing virus.",
            AccuseOf::default(),
            "I have never accused you of placing virus.",
        );
    }

    #[test]
    fn fix_got_accused_for() {
        assert_suggestion_result(
            "So anti-phishing folks got accused for phishing.",
            AccuseOf::default(),
            "So anti-phishing folks got accused of phishing.",
        );
    }

    #[test]
    fn fix_accuse_for_being() {
        assert_suggestion_result(
            "Emacs is not the SW application I would accuse for being bloated, quite the opposite in fact.",
            AccuseOf::default(),
            "Emacs is not the SW application I would accuse of being bloated, quite the opposite in fact.",
        );
    }

    #[test]
    fn fix_accuse_him_for() {
        assert_suggestion_result(
            "Lots of people accuse him for it and compare it to running things on WINE.",
            AccuseOf::default(),
            "Lots of people accuse him of it and compare it to running things on WINE.",
        );
    }

    #[test]
    fn fix_accuse_you_for() {
        assert_suggestion_result(
            "do you mean that someone accuse you for distributing virus in your open-sourced code",
            AccuseOf::default(),
            "do you mean that someone accuse you of distributing virus in your open-sourced code",
        );
    }

    #[test]
    fn fix_accusing_me_for() {
        assert_suggestion_result(
            "Once I dreamed about receiving an anonymous hate mail accusing me for wasting other developer's precious time.",
            AccuseOf::default(),
            "Once I dreamed about receiving an anonymous hate mail accusing me of wasting other developer's precious time.",
        );
    }

    // Potential false positives

    #[test]
    fn dont_flag_the_accused() {
        assert_no_lints(
            "You are a defense lawyer defending the accused for a murder case.",
            AccuseOf::default(),
        );
    }

    #[test]
    fn dont_flag_for_the_nth_time() {
        assert_no_lints(
            "I was accused for the first time of my writing looking AI-generated.",
            AccuseOf::default(),
        )
    }
}
