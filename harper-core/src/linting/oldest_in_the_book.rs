use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, Repeating, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct OldestInTheBook {
    expr: Box<dyn Expr>,
}

impl Default for OldestInTheBook {
    fn default() -> Self {
        let adj = |t: &Token, s: &[char]| {
            let k = &t.kind;
            (k.is_np_member() || k.is_adjective())
                && !k.is_noun()
                && !t
                    .span
                    .get_content(s)
                    .eq_ignore_ascii_case_chars(&['i', 'n'])
        };

        // Zero or more adjectives
        let adjseq = Repeating::new(Box::new(SequenceExpr::with(adj).t_ws()), 0);

        let noun = |t: &Token, s: &[char]| {
            let k = &t.kind;
            (k.is_np_member() || k.is_noun() || k.is_oov())
                && !t
                    .span
                    .get_content(s)
                    .eq_ignore_ascii_case_chars(&['i', 'n'])
        };

        // One or more nouns
        let nounseq = SequenceExpr::with(noun).then_optional(Repeating::new(
            Box::new(SequenceExpr::default().t_ws().then(noun)),
            1,
        ));

        let noun_phrase = SequenceExpr::optional(adjseq).then(nounseq);

        Self {
            expr: Box::new(
                SequenceExpr::fixed_phrase("oldest ")
                    .then(noun_phrase)
                    .then_fixed_phrase(" in the books"),
            ),
        }
    }
}

impl ExprLinter for OldestInTheBook {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        _ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let np = &toks[2..toks.len() - 4];
        let tricky = np.iter().any(|n| {
            n.span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(&["trick", "tricks"])
        });

        let message = if tricky {
            "This idiom should use singular `book` instead of plural `books`."
        } else {
            "If this is a play on the idiom `oldest trick in the book`, it should use singular `book` instead of plural `books`."
        }
        .to_string();

        Some(Lint {
            span: toks.last()?.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "book",
                toks.last()?.span.get_content(src),
            )],
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects the idiom `oldest X in the books`, which should use singular `book`."
    }
}

#[cfg(test)]
mod tests {
    use super::OldestInTheBook;
    use crate::linting::tests::{assert_lint_message, assert_suggestion_result};

    // Probable references to the idiom "oldest trick in the book"

    #[test]
    fn fix_delphi_mistake() {
        assert_suggestion_result(
            "This is the oldest Delphi mistake in the books and I'm sure you've made it before (we all have), and I'm sure you recognise it when you see it.",
            OldestInTheBook::default(),
            "This is the oldest Delphi mistake in the book and I'm sure you've made it before (we all have), and I'm sure you recognise it when you see it.",
        );
    }

    #[test]
    fn fix_trick() {
        assert_suggestion_result(
            "... oldest trick in the books, a restart and it works all the times(for now).",
            OldestInTheBook::default(),
            "... oldest trick in the book, a restart and it works all the times(for now).",
        );
    }

    #[test]
    fn fix_virus_trick() {
        assert_suggestion_result(
            "Once the OS is started the MBR is typically protected for virus reasons - this is one of the oldest virus tricks in the books - goes back to ...",
            OldestInTheBook::default(),
            "Once the OS is started the MBR is typically protected for virus reasons - this is one of the oldest virus tricks in the book - goes back to ...",
        )
    }

    #[test]
    fn fix_mistake() {
        assert_suggestion_result(
            "Ok, I realized now that I was making the oldest mistake in the books with my code, dividing my v by 2 instead of dividing it by 5.",
            OldestInTheBook::default(),
            "Ok, I realized now that I was making the oldest mistake in the book with my code, dividing my v by 2 instead of dividing it by 5.",
        );
    }

    #[test]
    fn fix_tricks() {
        assert_suggestion_result(
            "He enables the oldest tricks in the books, create fear from thing like prosperity (we really don't need Foxconn?)",
            OldestInTheBook::default(),
            "He enables the oldest tricks in the book, create fear from thing like prosperity (we really don't need Foxconn?)",
        );
    }

    #[test]
    fn fix_military_plays() {
        assert_suggestion_result(
            "Isnt that like one of the oldest military plays in the books?",
            OldestInTheBook::default(),
            "Isnt that like one of the oldest military plays in the book?",
        );
    }

    // Test messages

    #[test]
    fn is_oldest_trick_in_the_books_ref_to_idom() {
        assert_lint_message(
            "This is one of the oldest trick in the books",
            OldestInTheBook::default(),
            "This idiom should use singular `book` instead of plural `books`.",
        );
    }

    #[test]
    fn is_chromatic_alterations_ref_to_idom() {
        assert_lint_message(
            "One of the oldest chromatic alterations in the books is the raising of the leading tone",
            OldestInTheBook::default(),
            "If this is a play on the idiom `oldest trick in the book`, it should use singular `book` instead of plural `books`.",
        );
    }
}
