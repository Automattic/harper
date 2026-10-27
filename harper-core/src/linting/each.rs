use crate::{
    Lint, Token, TokenStringExt,
    char_string::CharStringExt,
    expr::{Expr, LongestMatchOf, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct Each {
    expr: SequenceExpr,
}

impl Default for Each {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_capitalization_of("each")
                .t_ws()
                .then_any_of([
                    Box::new(
                        SequenceExpr::default().then_plural_noun_only().but_not(
                            LongestMatchOf::new([
                                // don't flag "each drinks" because that's valid when "drinks" is a verb
                                Box::new(
                                    SequenceExpr::default()
                                        .then_verb_third_person_singular_present_form(),
                                ),
                                // don't flag "each data point" because "data" is part of a compound noun/noun phrase
                                Box::new(SequenceExpr::anything().t_ws().then_noun()),
                            ]),
                        ),
                    ) as Box<dyn Expr>,
                    Box::new(
                        SequenceExpr::word_seq(&["of", "the"])
                            .t_ws()
                            .then_singular_noun_only(),
                    ),
                ]),
        }
    }
}

fn each_of_the_noun(
    _toks: &[Token],
    _src: &[char],
    _ctx: Option<(&[Token], &[Token])>,
) -> Option<Lint> {
    todo!()
}

fn each_nouns(toks: &[Token], src: &[char], _ctx: Option<(&[Token], &[Token])>) -> Option<Lint> {
    let span = toks.span()?;

    let lint_kind = LintKind::Miscellaneous;
    let suggestions = vec![Suggestion::replace_with_match_case_str(
        "correction",
        span.get_content(src),
    )];
    let message = "Fix this erorr".to_owned();

    Some(Lint {
        span,
        lint_kind,
        suggestions,
        message,
        ..Default::default()
    })
}

impl ExprLinter for Each {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        let (each, of, the) = (0, 2, 4);
        let (_each, of, the) = (toks.get(each)?, toks.get(of)?, toks.get(the)?);

        // Which pattern did we match?
        if of.get_ch(src).eq_ch(&['o', 'f']) && the.get_ch(src).eq_ch(&['t', 'h', 'e']) {
            each_of_the_noun(toks, src, ctx)
        } else {
            each_nouns(toks, src, ctx)
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects wrong noun number after `each`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::Each;

    #[test]
    fn fix_each_data() {
        assert_suggestion_result(
            "Remember that each data has its own trade-offs.",
            Each::default(),
            "correction",
        );
    }

    #[test]
    fn dont_flag_each_data_point() {
        assert_no_lints(
            "Each data point is a pair comprising a topic and the user’s goal for conducting deep search on the topic.",
            Each::default(),
        );
    }

    #[test]
    #[ignore = "this is mass noun rather than plural. confusing it with plural is #407"]
    fn each_news() {
        assert_suggestion_result(
            "implement TFIDF on each news to find out the key",
            Each::default(),
            "implement TFIDF on all news to find out the key",
        );
    }
}
