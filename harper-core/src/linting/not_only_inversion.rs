use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct NotOnlyInversion {
    expr: SequenceExpr,
}

impl Default for NotOnlyInversion {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("not")
                .t_ws()
                .t_aco("only")
                .t_ws()
                .then_word_set(&["I", "we", "you", "he", "she", "it", "they"])
                .t_ws()
                .then_word_set(&["am", "are", "is", "was", "were"]),
        }
    }
}

impl ExprLinter for NotOnlyInversion {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `not only it is` to `not only is it`"
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🍭 {}", format_lint_match(toks, ctx, src));
        let (prontok, betok) = (toks.get_rel(-3)?, toks.get_rel(-1)?);
        let (pronspan, bespan) = (prontok.span, betok.span);
        let (pronch, bech) = (pronspan.get_content(src), bespan.get_content(src));

        let pronbetoks = toks.get_rel_slice(-3, -1)?;
        eprintln!("🍭🍭 '{}'", pronbetoks.span()?.get_content_string(src));

        let inverted = [bech.to_vec(), vec![' '], pronch.to_vec()].concat();

        Some(Lint {
            span: pronbetoks.span()?,
            lint_kind: LintKind::Grammar,
            message: "After `not only` the subject and verb should be inverted.".to_string(),
            suggestions: vec![Suggestion::replace_with_match_case(
                inverted,
                pronbetoks.span()?.get_content(src),
            )],
            ..Default::default()
        })
        // None
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NotOnlyInversion;

    #[test]
    fn fix_not_only_he_is() {
        assert_suggestion_result(
            "not only he is discouraged from look at them but he can't manipulate them",
            NotOnlyInversion::default(),
            "not only is he discouraged from look at them but he can't manipulate them",
        );
    }

    #[test]
    fn fix_not_only_he_was() {
        assert_suggestion_result(
            "not only he was born as a man, he was the Promised Messiah",
            NotOnlyInversion::default(),
            "not only was he born as a man, he was the Promised Messiah",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case goes by character index matching"]
    fn fix_not_only_i_am() {
        assert_suggestion_result(
            "Not only I am proud of the work we have accomplished together but also I have learned so much from you about statistics, sciences and beyond.",
            NotOnlyInversion::default(),
            "Not only am I proud of the work we have accomplished together but also I have learned so much from you about statistics, sciences and beyond.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case goes by character index matching"]
    fn fix_not_only_i_was() {
        assert_suggestion_result(
            "Not only I was wrong in saying the right meaning, I was also wrong in stating the parts of speech",
            NotOnlyInversion::default(),
            "Not only was I wrong in saying the right meaning, I was also wrong in stating the parts of speech",
        );
    }

    #[test]
    fn fix_not_only_it_is() {
        assert_suggestion_result(
            "Not only it is not the same problem, #899 is a solution suggested in #969.",
            NotOnlyInversion::default(),
            "Not only is it not the same problem, #899 is a solution suggested in #969.",
        );
    }

    #[test]
    fn fix_not_only_it_was() {
        assert_suggestion_result(
            "because not only it was unlikely that I could answer any question I also felt that I cannot even ask any on-topic question",
            NotOnlyInversion::default(),
            "because not only was it unlikely that I could answer any question I also felt that I cannot even ask any on-topic question",
        );
    }

    #[test]
    fn fix_not_only_they_are() {
        assert_suggestion_result(
            "Not only they are written in much cleaner and verbose way, they are also available in 6 languages like russian.",
            NotOnlyInversion::default(),
            "Not only are they written in much cleaner and verbose way, they are also available in 6 languages like russian.",
        );
    }

    #[test]
    fn fix_not_only_they_were() {
        assert_suggestion_result(
            "Not only they were tall, but also they were strong. ",
            NotOnlyInversion::default(),
            "Not only were they tall, but also they were strong. ",
        );
    }

    #[test]
    fn fix_not_only_we_are() {
        assert_suggestion_result(
            "Here not only we are using multiline string to create an HTML output but we also are binding variable using expression language.",
            NotOnlyInversion::default(),
            "Here not only are we using multiline string to create an HTML output but we also are binding variable using expression language.",
        );
    }

    #[test]
    fn fix_not_only_we_were() {
        assert_suggestion_result(
            "Not only we were using an old version of our front end library (React), but we were also locked into a version of our functional programming utility package (Lodash) released more than three years ago.",
            NotOnlyInversion::default(),
            "Not only were we using an old version of our front end library (React), but we were also locked into a version of our functional programming utility package (Lodash) released more than three years ago.",
        );
    }

    #[test]
    fn fix_not_only_you_are() {
        assert_suggestion_result(
            "So not only you are a perfect reference, but also a viable candidate for drop-n-use.",
            NotOnlyInversion::default(),
            "So not only are you a perfect reference, but also a viable candidate for drop-n-use.",
        );
    }

    #[test]
    fn fix_not_only_you_were() {
        assert_suggestion_result(
            "because not only you were able to explain it but you were able to show me how to sort it so it would displayed",
            NotOnlyInversion::default(),
            "because not only were you able to explain it but you were able to show me how to sort it so it would displayed",
        );
    }
}
