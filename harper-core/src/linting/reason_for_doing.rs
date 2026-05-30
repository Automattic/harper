use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct ReasonForDoing {
    expr: SequenceExpr,
}

impl Default for ReasonForDoing {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["reason", "reasons"])
                .t_ws()
                .t_aco("of")
                .t_ws()
                .then_verb_progressive_form(),
        }
    }
}

impl ExprLinter for ReasonForDoing {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `reason of doing` to `reason for doing` etc."
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if toks.len() != 5 {
            return None;
        }
        const REASON: usize = 0;
        const OF: usize = 2;
        // let reasontok = &toks[REASON];
        // let reasonspan = reasontok.span;
        let reasonchars = toks[REASON].get_ch(src);
        // let oftok = &toks[OF];
        let ofspan = toks[OF].span;
        // let ofchars = ofspan.get_content(src);

        // "for reasons of doing" is a legit construction. TODO: Usually, but not always!
        if reasonchars.last()? == &'s'
            && preceded_by_word(ctx, |t| t.get_ch(src).eq_ch(&['f', 'o', 'r']))
        {
            return None;
        }

        Some(Lint {
            span: ofspan,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "for",
                ofspan.get_content(src),
            )],
            message: "Use 'for' instead of 'of' with 'reason' and progressive verbs.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ReasonForDoing;

    #[test]
    fn fix_sg_of_doing() {
        assert_suggestion_result(
            "However i could not find any valid reason of doing this in one project, it's still possible.",
            ReasonForDoing::default(),
            "However i could not find any valid reason for doing this in one project, it's still possible.",
        );
    }

    #[test]
    fn fix_pl_of_doing() {
        assert_suggestion_result(
            "It actually helped me a lot understanding what is your recommended way of implementing safe-listing and your reasons of doing it this way.",
            ReasonForDoing::default(),
            "It actually helped me a lot understanding what is your recommended way of implementing safe-listing and your reasons for doing it this way.",
        );
    }

    #[test]
    fn fix_sg_of_having() {
        assert_suggestion_result(
            "what's the reason of having USE_INTERPOLATION_TABLES in UserParams then?",
            ReasonForDoing::default(),
            "what's the reason for having USE_INTERPOLATION_TABLES in UserParams then?",
        );
    }

    #[test]
    fn fix_pl_of_having() {
        assert_suggestion_result(
            "Any reasons of having other implementation than specified in docs?",
            ReasonForDoing::default(),
            "Any reasons for having other implementation than specified in docs?",
        );
    }

    #[test]
    fn ignore_for_reasons_of_logging() {
        assert_no_lints(
            "whether for reasons of logging, monitoring, etc.",
            ReasonForDoing::default(),
        );
    }

    #[test]
    fn fix_sg_of_making() {
        assert_suggestion_result(
            "The fact is that I am seeing where is it being used and I cannot understand the reason of making it boolean.",
            ReasonForDoing::default(),
            "The fact is that I am seeing where is it being used and I cannot understand the reason for making it boolean.",
        );
    }

    #[test]
    fn fix_pl_of_making() {
        assert_suggestion_result(
            "So for the reasons of making it self describable as much as possible, I think it is important to express version there.",
            ReasonForDoing::default(),
            "So for the reasons for making it self describable as much as possible, I think it is important to express version there.",
        );
    }

    #[test]
    fn allow_for_reasons_of_making() {
        assert_no_lints(
            "That said, and just for reasons of making the simple-openai code more robust, one could verify that it is not null and throw the appropriate exceptions",
            ReasonForDoing::default(),
        );
    }

    #[test]
    #[ignore = "Known false negative where we should't allow 'reasons of doing' just because it follows 'for'"]
    fn fix_for_reasons_of_doing_this() {
        assert_suggestion_result(
            "For reasons of doing this, it is because I have some ops that need to carefully sort their execution orders in not only forward pass",
            ReasonForDoing::default(),
            "For reasons for doing this, it is because I have some ops that need to carefully sort their execution orders in not only forward pass",
        );
    }

    #[test]
    fn fix_sg_of_needing() {
        assert_suggestion_result(
            "I just came to the same conclusion for the same reason of needing a scoped dependency in my DbContext to support global query filtering.",
            ReasonForDoing::default(),
            "I just came to the same conclusion for the same reason for needing a scoped dependency in my DbContext to support global query filtering.",
        );
    }

    #[test]
    fn fix_sg_of_opening() {
        assert_suggestion_result(
            "Hi, What is reason of opening DB exception \"UnknownError: Internal error opening backing store for indexedDB\"?",
            ReasonForDoing::default(),
            "Hi, What is reason for opening DB exception \"UnknownError: Internal error opening backing store for indexedDB\"?",
        );
    }

    #[test]
    fn fix_sg_of_saving() {
        assert_suggestion_result(
            "be notified about document save events with source/reason of saving",
            ReasonForDoing::default(),
            "be notified about document save events with source/reason for saving",
        );
    }

    #[test]
    fn fix_sg_of_wanting() {
        assert_suggestion_result(
            "How do you use Severity and what is the impact/reason of wanting to having different values to you (i.e. what difference does it make to you)?",
            ReasonForDoing::default(),
            "How do you use Severity and what is the impact/reason for wanting to having different values to you (i.e. what difference does it make to you)?",
        );
    }

    #[test]
    #[ignore = "We don't yet handle words between 'for' and 'reasons of'"]
    fn allow_for_reasons_of_wanting() {
        assert_no_lints(
            "Ideally we don't want to reduce the idleTimeout (currently set to 120s) for the standard reasons of wanting to be able to handle bursty traffic.",
            ReasonForDoing::default(),
        );
    }
}
