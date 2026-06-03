use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Sentence, at_start_of_sentence, followed_by_word, preceded_by_word},
    },
};

pub struct AspireTo {
    expr: SequenceExpr,
}

impl Default for AspireTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["aspire", "aspired", "aspires", "aspiring"])
                .t_ws()
                .t_aco("for"),
        }
    }
}

impl ExprLinter for AspireTo {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `aspire for` to `aspire to`."
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let (aspire_i, prep_i) = (0, 2);
        let (aspire_t, prep_t) = (&toks[aspire_i], &toks[prep_i]);
        let (aspire_s, prep_s) = (aspire_t.span, prep_t.span);

        if aspire_s.get_content(src) == ['A', 's', 'p', 'i', 'r', 'e'] && !at_start_of_sentence(ctx)
        {
            return None;
        }

        if preceded_by_word(ctx, |wt| {
            // .NET is unlintable and we can't use `.get_content()` on unlintable
            if wt.kind.is_preposition() || wt.kind.is_unlintable() {
                true
            } else {
                let chars = wt.span.get_content(src);

                chars == ['N', 'E', 'T']
                    || chars.eq_any_ignore_ascii_case_str(&[
                        // verbs that indicate Aspire is a tool or product
                        "use", "used", "uses", "using",
                        // other words that precede Aspire when it's not a verb
                        "dotnet",
                    ])
            }
        }) || followed_by_word(ctx, |wt| wt.span.get_content(src) == ['A', 'W', 'S'])
        {
            return None;
        }

        Some(Lint {
            span: prep_s,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                prep_s.get_content(src),
            )],
            message: "Use `aspire to` instead of `aspire for`.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AspireTo;

    #[test]
    fn test_aspire_to() {
        assert_suggestion_result("aspire for", AspireTo::default(), "aspire to");
    }

    #[test]
    fn ignore_after_using() {
        assert_no_lints(
            "I am developing an application on ASP.NET Core 8 using Aspire for infrastructure management.",
            AspireTo::default(),
        );
    }

    #[test]
    fn ignore_after_all_caps_net() {
        assert_no_lints(
            "This repositry contains the integrations with .NET Aspire for AWS.",
            AspireTo::default(),
        );
    }

    #[test]
    fn ignore_after_all_caps_net_no_dot() {
        // NOTE the .NET gets tokenized as Unlintable!?
        assert_no_lints(
            "This repositry contains the integrations with NET Aspire for AWS.",
            AspireTo::default(),
        );
    }

    #[test]
    fn dont_ignore_after_lowercase_net() {
        assert_suggestion_result(
            "my net aspires for catch a bug",
            AspireTo::default(),
            "my net aspires to catch a bug",
        );
    }

    #[test]
    fn flag_at_start_of_doc_when_capitalized() {
        assert_suggestion_result(
            "Aspire for greatness, even at the start of a sentence.",
            AspireTo::default(),
            "Aspire to greatness, even at the start of a sentence.",
        );
    }

    #[test]
    fn flag_at_the_start_of_a_sentence_mid_document() {
        assert_suggestion_result(
            "This is a sentence. Aspire for greatness, even at the start of a mid-document sentence.",
            AspireTo::default(),
            "This is a sentence. Aspire to greatness, even at the start of a mid-document sentence.",
        );
    }

    #[test]
    fn dont_flag_capitalized_mid_document() {
        assert_no_lints(
            "I reccumend Aspire for your next project.",
            AspireTo::default(),
        );
    }
}
