use crate::{
    Lint, Token,
    expr::{All, AnchorStart, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct MeanBy {
    expr: All,
}

impl Default for MeanBy {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_subject_pronoun()
                .t_ws()
                .t_set(&["mean", "means", "meant", "meaning"])
                .t_ws()
                .t_aco("with")
                .but_not(AnchorStart),
        }
    }
}

impl ExprLinter for MeanBy {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));

        if matched_tokens.len() < 5 {
            return None;
        }

        let span = matched_tokens[4].span;
        let lint_kind = LintKind::Miscellaneous;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "by",
            span.get_content(source),
        )];
        let message = "Use `by` instead of `with`.".to_string();

        Some(Lint {
            span,
            lint_kind,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Use `mean by` instead of `mean with`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::MeanBy;

    // True positive tests

    #[test]
    fn fix_i_mean() {
        assert_suggestion_result(
            "This is what I mean with that it is not ok for the standard library to depend on UB.",
            MeanBy::default(),
            "This is what I mean by that it is not ok for the standard library to depend on UB.",
        );
    }

    #[test]
    fn fix_i_meant() {
        assert_suggestion_result(
            "Thats what i meant with strong extrapolation.",
            MeanBy::default(),
            "Thats what i meant by strong extrapolation.",
        );
    }

    #[test]
    fn fix_we_mean() {
        assert_suggestion_result(
            "state-machine: what do we mean with \"processed\"?",
            MeanBy::default(),
            "state-machine: what do we mean by \"processed\"?",
        );
    }

    #[test]
    fn fix_you_mean() {
        assert_suggestion_result(
            "I'm sorry but I don't understand what do you mean with this, each run I need to download a new file",
            MeanBy::default(),
            "I'm sorry but I don't understand what do you mean by this, each run I need to download a new file",
        );
    }

    #[test]
    fn fix_you_meant() {
        assert_suggestion_result(
            "I think I didn't understand what you meant with \"did you try with RecyclableMemoryStream?\".",
            MeanBy::default(),
            "I think I didn't understand what you meant by \"did you try with RecyclableMemoryStream?\".",
        );
    }

    #[test]
    fn flag_it_mean_with() {
        assert_suggestion_result(
            "What does it mean with 'components are valid', it is valid, it is React.",
            MeanBy::default(),
            "What does it mean by 'components are valid', it is valid, it is React.",
        );
    }

    #[test]
    fn fix_they_mean() {
        assert_suggestion_result(
            "I'm not sure what they mean with \"bidirectional\".",
            MeanBy::default(),
            "I'm not sure what they mean by \"bidirectional\".",
        );
    }

    // Potential false positive

    #[test]
    fn dont_flag_do_what_i_mean_with() {
        // Note: This succeeds due to the hyphen and would surely fail otherwise.
        assert_no_lints(
            "Do-what-I-mean with the port when copying URL and changing the scheme",
            MeanBy::default(),
        );
    }

    #[test]
    fn dont_flag_i_mean_with_at_start() {
        assert_no_lints(
            "i mean with Premier Pro and Final Cut Pro, i can add videos and audio at once and start to edit and etc right away",
            MeanBy::default(),
        );
    }

    #[test]
    fn dont_flag_does_it_mean_with_at_start() {
        // Could handle this by "does he/she/it" at start OR by "or without" after "with"
        assert_no_lints("Does it mean with or without backend?", MeanBy::default());
    }

    #[test]
    fn dont_flag_what_they_mean_with_eq_what_they_mean_using() {
        // Note: This one is very tricky. Here "with" means "using".
        assert_no_lints(
            "Let's discuss what they mean with three famous monoids, addition ( + ), multiplication ( * ), and string concatenation ( <> ).",
            MeanBy::default(),
        );
    }

    #[test]
    fn dont_flag_what_it_means_with_eq_what_it_means_using() {
        // Note: This one is very tricky. Here "with" means "using".
        assert_no_lints(
            "Let's discuss what it means with three famous monoids, addition ( + ), multiplication ( * ), and string concatenation ( <> ).",
            MeanBy::default(),
        );
    }

    #[test]
    fn dont_flag_it_means_with_3_proxy() {
        assert_no_lints(
            "It means with 3 proxy, there are 6 zero tier containers and a total of 9 containers?",
            MeanBy::default(),
        );
    }

    // More examples can be found with this Google search:
    // `"i OR we OR you OR he OR she OR it OR they mean OR means OR meant OR meaning with" site:github.com -"we OR you OR i mean" -"i OR you meant"`
}
