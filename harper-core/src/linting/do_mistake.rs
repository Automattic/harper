use crate::{
    Lint, Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, expr_linter::Chunk},
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
                    .then_any_of(vec![
                        Box::new(WordSet::new(&[
                            "a", "an", "the", "that", "these", "this", "those", "another", "many",
                            "my", "our", "your", "his", "her", "its", "their",
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
        eprintln!(
            "💥 {}",
            crate::linting::debug::format_lint_match(toks, ctx, src)
        );
        None
    }

    fn description(&self) -> &str {
        "Corrects `do a mistake` to `make a mistake`"
    }

    fn expr(&self) -> &dyn Expr {
        &*self.expr
    }
}

#[cfg(test)]
mod tests {
    use super::DoMistake;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    #[test]
    fn did_a_mistake() {
        assert_lint_count(
            "Hi, I did a mistake in my NGINX config file and so once the container is launched, it logs the error...",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn did_my_mistakes() {
        assert_lint_count("Where i did my mistakes?", DoMistake::default(), 1);
    }

    #[test]
    fn did_several_mistakes() {
        assert_lint_count(
            "Maybe I did several mistakes, but I can only find a message about one?",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn did_some_mistakes() {
        assert_lint_count(
            "I made this program to learn goto use. but did some mistakes somewhere",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn did_that_mistake() {
        assert_lint_count(
            "and believe me, I did that mistake too",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn did_the_mistake() {
        assert_lint_count(
            "The issue describe is the person who did the mistake in the past & that same person is NOW correcting other people",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn did_this_mistake() {
        assert_lint_count(
            "Are there famous mathematicians who did this mistake?",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_many_mistakes() {
        assert_lint_count(
            "I observed that my coworkers do many mistakes using the field calculator",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_mistake() {
        assert_lint_count(
            "If you do a mistake that causes alot of problems, please use the command to redo",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_some_mistakes() {
        assert_lint_count(
            "so probably if my colleagues do some mistakes I tend to learn them as well",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_the_mistake() {
        assert_lint_count(
            "do I need to explicitly mention that I did not do the mistake to do not lose the point?",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_this_mistake() {
        assert_lint_count(
            "I barely remember any frontend developer that wouldn't do this mistake at least once.",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_this_mistakes() {
        assert_lint_count(
            "I do this mistakes to check the command detekt with type resolution",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn do_those_mistakes() {
        assert_lint_count(
            "An experienced developer could do those mistakes as well",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn doing_a_mistake() {
        assert_lint_count(
            "Here at work, a colleague asked if we were doing a mistake by using the ReactDOM.renderToStaticMarkup on the client side.",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn doing_several_mistakes() {
        assert_lint_count(
            "I realized I was doing several mistakes",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn doing_the_mistake() {
        assert_lint_count("where am i doing the mistake?", DoMistake::default(), 1);
    }

    #[test]
    fn done_some_mistake() {
        assert_lint_count(
            "Might be I have done some mistake, that I do not know.",
            DoMistake::default(),
            1,
        );
    }

    #[test]
    fn done_this_mistake() {
        assert_lint_count(
            "how many more users have done this mistake?",
            DoMistake::default(),
            1,
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
