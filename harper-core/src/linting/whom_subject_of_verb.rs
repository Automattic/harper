use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::ModalVerb,
};

pub struct WhomSubjectOfVerb {
    expr: Box<dyn Expr>,
}

impl Default for WhomSubjectOfVerb {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["whom", "whomever", "whomsoever"])
                    .t_ws()
                    // .then_verb()
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_kind_where(|k| {
                            k.is_verb_third_person_singular_present_form()
                                || k.is_verb_simple_past_form()
                        })),
                        Box::new(ModalVerb::with_common_errors()),
                    ]),
            ),
        }
    }
}

impl ExprLinter for WhomSubjectOfVerb {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Detects whom and its variants used as the subject of a verb instead of who."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((before, _)) = ctx
            && let [.., word, ws1, prep, ws2] = before
            && ws2.kind.is_whitespace()
            && prep
                .span
                .get_content(src)
                .eq_ignore_ascii_case_chars(&['o', 'f'])
            && ws1.kind.is_whitespace()
            && word.span.get_content(src).eq_ignore_ascii_case_str("many")
        {
            return None;
        }

        let whom_span = toks.first()?.span;
        let whom_chars = whom_span.get_content(src);

        let who_vec = [&whom_chars[..3], &whom_chars[4..]].concat();

        Some(Lint {
            span: whom_span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(who_vec, whom_chars)],
            message: "“Whom” is used for the object of a verb and “who” is used for the subject of a verb.".to_owned(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::WhomSubjectOfVerb;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn flag_whom_has() {
        assert_suggestion_result(
            "there is no course to whom has opened the most PRs",
            WhomSubjectOfVerb::default(),
            "there is no course to who has opened the most PRs",
        );
    }

    #[test]
    fn flag_whomever_wrote() {
        assert_suggestion_result(
            "To whomever wrote this course, I truly am not trying to be a jerk or ungrateful",
            WhomSubjectOfVerb::default(),
            "To whoever wrote this course, I truly am not trying to be a jerk or ungrateful",
        );
    }

    #[test]
    #[ignore = "wrong kind of error"]
    fn dont_flag_wrong_kind_of_error() {
        assert_lint_count(
            "self service ticket view is not showing to whom is the ticket assigned to",
            //   "self service ticket view is not showing to whom this ticket is assigned"
            //   "self service ticket view is not showing whom this ticket is assigned to"
            WhomSubjectOfVerb::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_whom_can() {
        assert_suggestion_result(
            "Whom can of course build a helper, but that is only a workaround.",
            WhomSubjectOfVerb::default(),
            "Who can of course build a helper, but that is only a workaround.",
        );
    }

    #[test]
    fn flag_whomever_is() {
        assert_suggestion_result(
            "Whomever is making those harassing phone calls to me after I post something on Github - consider yourself put on notice.",
            WhomSubjectOfVerb::default(),
            "Whoever is making those harassing phone calls to me after I post something on Github - consider yourself put on notice.",
        );
    }

    #[test]
    fn flag_whom_is() {
        assert_suggestion_result(
            "I thought it might be good idea to address the topic of whom is \"allowed\" to merge.",
            WhomSubjectOfVerb::default(),
            "I thought it might be good idea to address the topic of who is \"allowed\" to merge.",
        );
    }

    #[test]
    fn flag_whomsoever_will() {
        assert_suggestion_result(
            "This is a quick record of my discoveries and solution for whomsoever will be fixing the issue.",
            WhomSubjectOfVerb::default(),
            "This is a quick record of my discoveries and solution for whosoever will be fixing the issue.",
        );
    }

    #[test]
    fn dont_flag_many_of_whom() {
        assert_no_lints(
            "it's far from straightforward for new users, many of whom will likely have a lot to learn",
            WhomSubjectOfVerb::default(),
        );
    }
}
