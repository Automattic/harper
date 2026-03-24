use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FixedPhrase},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word, preceded_by_word},
    },
};

pub struct ExceptOf {
    expr: FixedPhrase,
}

impl Default for ExceptOf {
    fn default() -> Self {
        Self {
            expr: FixedPhrase::from_phrase("except of"),
        }
    }
}

impl ExprLinter for ExceptOf {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if followed_by_word(ctx, |t| {
            t.span.get_content(src).eq_ignore_ascii_case_str("course")
        }) {
            return None;
        }

        let (span, replacement, msg) = if preceded_by_word(ctx, |t| {
            t.span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(&["the", "notable", "possible"])
        }) {
            (
                toks[0].span,
                "exception",
                "This usage requires the noun 'exception'. Use 'exception of' instead of 'except of'",
            )
        } else {
            (
                toks[2].span,
                "for",
                "Use `except for` instead of `except of`",
            )
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                span.get_content(src),
            )],
            message: msg.to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `except of` to `except for` or `exception of`."
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ExceptOf;

    #[test]
    fn fix_the_except_of() {
        assert_suggestion_result(
            "With the except of one particular sender, all the other senders are sending correctly to the configured smarthost.",
            ExceptOf::default(),
            "With the exception of one particular sender, all the other senders are sending correctly to the configured smarthost.",
        );
    }

    #[test]
    fn fix_possible_except_of() {
        assert_suggestion_result(
            "With the possible except of two vertexes with one having one more in-degree than out-degree",
            ExceptOf::default(),
            "With the possible exception of two vertexes with one having one more in-degree than out-degree",
        );
    }

    #[test]
    fn fix_notable_except_of() {
        assert_suggestion_result(
            "identical at the end of the day to never logging into it again (with the notable except of potentially identifying info you've left in your profile, of course)",
            ExceptOf::default(),
            "identical at the end of the day to never logging into it again (with the notable exception of potentially identifying info you've left in your profile, of course)",
        );
    }

    #[test]
    fn ignore_except_of_course() {
        assert_no_lints(
            "much like the existing config option access_log_format, except of course for the error log (option errorlog) output format.",
            ExceptOf::default(),
        );
    }

    #[test]
    fn fix_except_of() {
        assert_suggestion_result(
            "There are no usable way to restart worker except of supervisorctl restart workername",
            ExceptOf::default(),
            "There are no usable way to restart worker except for supervisorctl restart workername",
        );
    }
}
