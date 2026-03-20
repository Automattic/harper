use crate::{
    CharStringExt, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word},
    },
    patterns::{Word, WordSet},
};

pub struct BeWorried {
    expr: All,
}

impl Default for BeWorried {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_any_of(vec![
                    Box::new(
                        SequenceExpr::default()
                            .then_subject_pronoun()
                            .t_ws()
                            .t_set(&["am", "are", "is", "was", "were"]),
                    ),
                    Box::new(WordSet::new(&[
                        "i'm", "we're", "you're", "he's", "she's", "they're",
                    ])),
                ])
                .t_ws()
                .t_aco("worry")
                .and_not(Word::new("it")),
        }
    }
}

impl ExprLinter for BeWorried {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let wtok = toks.last()?;

        if followed_by_hyphen(ctx)
            || followed_by_word(ctx, |w| {
                w.span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(&["free", "warts"])
            })
        {
            return None;
        }

        Some(Lint {
            span: wtok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "worried",
                wtok.span.get_content(src),
            )],
            message: "Use 'worried' instead of 'worry'.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Detects incorrect use of 'be worry' instead of `be worried`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::BeWorried;

    #[test]
    fn he_is() {
        assert_suggestion_result(
            "I guess he is worry about \" * user * \" tag.",
            BeWorried::default(),
            "I guess he is worried about \" * user * \" tag.",
        );
    }

    #[test]
    fn he_was() {
        assert_suggestion_result(
            "So he was worry about her. Especially, when he got no response by calling her on her phone nor ranging her doorbell.",
            BeWorried::default(),
            "So he was worried about her. Especially, when he got no response by calling her on her phone nor ranging her doorbell.",
        );
    }

    #[test]
    fn i_am() {
        assert_suggestion_result(
            "I didn't see any section dedicated to this so I am worry about:",
            BeWorried::default(),
            "I didn't see any section dedicated to this so I am worried about:",
        );
    }

    #[test]
    fn i_was() {
        assert_suggestion_result(
            "So that's why I was worry.",
            BeWorried::default(),
            "So that's why I was worried.",
        );
    }

    #[test]
    fn i_were() {
        assert_suggestion_result(
            "The only things that I were worry about is the data that could be lost using this deletion.",
            BeWorried::default(),
            "The only things that I were worried about is the data that could be lost using this deletion.",
        );
    }

    #[test]
    fn they_are() {
        assert_suggestion_result(
            "at the same time they are worry about the price for the upgrade each 3 years",
            BeWorried::default(),
            "at the same time they are worried about the price for the upgrade each 3 years",
        );
    }

    #[test]
    fn theyre_worry() {
        assert_suggestion_result(
            "Because they're worry this link is spam or they scare have to pay more money.",
            BeWorried::default(),
            "Because they're worried this link is spam or they scare have to pay more money.",
        );
    }

    #[test]
    fn we_are() {
        assert_suggestion_result(
            "We are analised this and we are worry because when our platform go to market",
            BeWorried::default(),
            "We are analised this and we are worried because when our platform go to market",
        );
    }

    #[test]
    fn were() {
        assert_suggestion_result(
            "We're worry about all kinds of minority representation in TV.",
            BeWorried::default(),
            "We're worried about all kinds of minority representation in TV.",
        );
    }

    #[test]
    fn you_are() {
        assert_suggestion_result(
            "You are worry because we are not annotating view interface itself, right?",
            BeWorried::default(),
            "You are worried because we are not annotating view interface itself, right?",
        );
    }

    #[test]
    fn youre() {
        assert_suggestion_result(
            "You're worry about memory usage and wanna be sure that a Sequence-class won't hold your activity against GC — declare this class as static",
            BeWorried::default(),
            "You're worried about memory usage and wanna be sure that a Sequence-class won't hold your activity against GC — declare this class as static",
        );
    }

    #[test]
    fn dont_flag_it_is() {
        assert_no_lints(
            "Part of it is worry that my bosses will get angry and fire me.",
            BeWorried::default(),
        );
    }

    #[test]
    fn dont_flag_it_was() {
        assert_no_lints(
            "Because what followed wasn't indifference, it was worry.",
            BeWorried::default(),
        );
    }

    #[test]
    fn dont_flag_she_was_worry_free() {
        assert_no_lints("textFinally, she was worry-free.", BeWorried::default());
    }

    #[test]
    fn dont_flag_theyre_worry_free() {
        assert_no_lints(
            "They don't pretend they're worry-free.",
            BeWorried::default(),
        );
    }

    #[test]
    fn dont_flag_worry_warts() {
        assert_no_lints(
            "Thanks to jQuery, we're worry warts from browser compatibility.",
            BeWorried::default(),
        );
    }

    #[test]
    fn dont_flag_were_worry_space_free() {
        assert_no_lints(
            "Thanks to jQuery, we're worry free from browser compatibility.",
            BeWorried::default(),
        );
    }

    #[test]
    #[ignore = "edge case not yet handled"]
    fn cant_fix_edge_case_yet() {
        assert_good_and_bad_suggestions(
            "Myself along with others are using it on an iPad successfully, so it is worry to hear that is broken for you.",
            BeWorried::default(),
            &[
                "Myself along with others are using it on an iPad successfully, so it is worrying to hear that is broken for you.",
                "Myself along with others are using it on an iPad successfully, so it is a worry to hear that is broken for you.",
            ],
            &[],
        );
    }
}
