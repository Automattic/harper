use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
    patterns::WordSet,
};

pub struct DoMistake {
    expr: Box<dyn Expr>,
}

impl Default for DoMistake {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["do", "did", "does", "doing", "done"])
                    .t_ws()
                    .then_longest_of(vec![
                        Box::new(WordSet::new(&[
                            "a", "an", "the", "that", "these", "this", "those", "another", "many",
                            "several", "some", "my", "our", "your", "his", "her", "its", "their",
                        ])),
                        Box::new(FixedPhrase::from_phrase("a lot of")),
                        Box::new(FixedPhrase::from_phrase("lots of")),
                        Box::new(FixedPhrase::from_phrase("that kind of")),
                        Box::new(FixedPhrase::from_phrase("these kinds of")),
                        Box::new(FixedPhrase::from_phrase("this kind of")),
                        Box::new(FixedPhrase::from_phrase("those kinds of")),
                        Box::new(FixedPhrase::from_phrase("so many")),
                        Box::new(FixedPhrase::from_phrase("too many")),
                        Box::new(FixedPhrase::from_phrase("tons of")),
                        Box::new(FixedPhrase::from_phrase("tonnes of")),
                    ])
                    .t_ws()
                    .then_word_set(&["mistake", "mistakes"]),
            ),
        }
    }
}

impl ExprLinter for DoMistake {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let tok = toks.first()?;
        let span = tok.span;
        let chars = span.get_content(src);

        if followed_by_word(ctx, |nw| {
            nw.kind.is_verb() && !nw.kind.is_verb_progressive_form()
        }) {
            return None;
        }

        let make = if chars.eq_ignore_ascii_case_str("do") {
            "make"
        } else if chars.eq_ignore_ascii_case_str("did") || chars.eq_ignore_ascii_case_str("done") {
            "made"
        } else if chars.eq_ignore_ascii_case_str("does") {
            "makes"
        } else if chars.eq_ignore_ascii_case_str("doing") {
            "making"
        } else {
            return None;
        }
        .chars()
        .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(make, chars)],
            message: "In English we `make` mistakes, not `do` them".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `do a mistake` to `make a mistake`."
    }

    fn expr(&self) -> &dyn Expr {
        &*self.expr
    }
}

#[cfg(test)]
mod tests {
    use super::DoMistake;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn did_a_mistake() {
        assert_suggestion_result(
            "Hi, I did a mistake in my NGINX config file and so once the container is launched, it logs the error...",
            DoMistake::default(),
            "Hi, I made a mistake in my NGINX config file and so once the container is launched, it logs the error...",
        );
    }

    #[test]
    fn did_my_mistakes() {
        assert_suggestion_result(
            "Where i did my mistakes?",
            DoMistake::default(),
            "Where i made my mistakes?",
        );
    }

    #[test]
    fn did_several_mistakes() {
        assert_suggestion_result(
            "Maybe I did several mistakes, but I can only find a message about one?",
            DoMistake::default(),
            "Maybe I made several mistakes, but I can only find a message about one?",
        );
    }

    #[test]
    fn did_some_mistakes() {
        assert_suggestion_result(
            "I made this program to learn goto use. but did some mistakes somewhere",
            DoMistake::default(),
            "I made this program to learn goto use. but made some mistakes somewhere",
        );
    }

    #[test]
    fn did_that_mistake() {
        assert_suggestion_result(
            "and believe me, I did that mistake too",
            DoMistake::default(),
            "and believe me, I made that mistake too",
        );
    }

    #[test]
    fn did_the_mistake() {
        assert_suggestion_result(
            "The issue describe is the person who did the mistake in the past & that same person is NOW correcting other people",
            DoMistake::default(),
            "The issue describe is the person who made the mistake in the past & that same person is NOW correcting other people",
        );
    }

    #[test]
    fn did_this_mistake() {
        assert_suggestion_result(
            "Are there famous mathematicians who did this mistake?",
            DoMistake::default(),
            "Are there famous mathematicians who made this mistake?",
        );
    }

    #[test]
    fn do_many_mistakes() {
        assert_suggestion_result(
            "I observed that my coworkers do many mistakes using the field calculator",
            DoMistake::default(),
            "I observed that my coworkers make many mistakes using the field calculator",
        );
    }

    #[test]
    fn do_mistake() {
        assert_suggestion_result(
            "If you do a mistake that causes alot of problems, please use the command to redo",
            DoMistake::default(),
            "If you make a mistake that causes alot of problems, please use the command to redo",
        );
    }

    #[test]
    fn do_some_mistakes() {
        assert_suggestion_result(
            "so probably if my colleagues do some mistakes I tend to learn them as well",
            DoMistake::default(),
            "so probably if my colleagues make some mistakes I tend to learn them as well",
        );
    }

    #[test]
    fn do_the_mistake() {
        assert_suggestion_result(
            "do I need to explicitly mention that I did not do the mistake to do not lose the point?",
            DoMistake::default(),
            "do I need to explicitly mention that I did not make the mistake to do not lose the point?",
        );
    }

    #[test]
    fn do_this_mistake() {
        assert_suggestion_result(
            "I barely remember any frontend developer that wouldn't do this mistake at least once.",
            DoMistake::default(),
            "I barely remember any frontend developer that wouldn't make this mistake at least once.",
        );
    }

    #[test]
    fn do_this_mistakes() {
        assert_suggestion_result(
            "I do this mistakes to check the command detekt with type resolution",
            DoMistake::default(),
            "I make this mistakes to check the command detekt with type resolution",
        );
    }

    #[test]
    fn do_those_mistakes() {
        assert_suggestion_result(
            "An experienced developer could do those mistakes as well",
            DoMistake::default(),
            "An experienced developer could make those mistakes as well",
        );
    }

    #[test]
    fn doing_a_mistake() {
        assert_suggestion_result(
            "Here at work, a colleague asked if we were doing a mistake by using the ReactDOM.renderToStaticMarkup on the client side.",
            DoMistake::default(),
            "Here at work, a colleague asked if we were making a mistake by using the ReactDOM.renderToStaticMarkup on the client side.",
        );
    }

    #[test]
    fn doing_several_mistakes() {
        assert_suggestion_result(
            "I realized I was doing several mistakes",
            DoMistake::default(),
            "I realized I was making several mistakes",
        );
    }

    #[test]
    fn doing_the_mistkae() {
        assert_suggestion_result(
            "where am i doing the mistake?",
            DoMistake::default(),
            "where am i making the mistake?",
        );
    }

    #[test]
    fn done_some_mistake() {
        assert_suggestion_result(
            "Might be I have done some mistake, that I do not know.",
            DoMistake::default(),
            "Might be I have made some mistake, that I do not know.",
        );
    }

    #[test]
    fn done_this_mistake() {
        assert_suggestion_result(
            "how many more users have done this mistake?",
            DoMistake::default(),
            "how many more users have made this mistake?",
        );
    }

    // False positives

    #[test]
    fn dont_flag_when_does_a_mistake() {
        assert_no_lints(
            "When does a mistake become standard usage? ",
            DoMistake::default(),
        );
    }

    #[test]
    fn dont_flag_did_that_mistake_verb() {
        assert_no_lints(
            "Did that mistake occurred before or after the day 2 backup?",
            DoMistake::default(),
        );
    }

    #[test]
    fn dont_flag_does_this_mistake_verb() {
        assert_no_lints(
            "Does this mistake invalidate your thesis?",
            DoMistake::default(),
        );
    }

    #[test]
    fn dont_flag_does_the_mistake_verb() {
        assert_no_lints(
            "Does the mistake change the meaning of the quotation?",
            DoMistake::default(),
        );
    }
}
