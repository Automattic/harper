use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        debug::format_lint_match,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct TryingToDoing {
    expr: SequenceExpr,
}

impl Default for TryingToDoing {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_verb_progressive_form()
                .t_ws()
                .t_set(&["to", "too"])
                .t_ws()
                .then_verb_progressive_form(),
        }
    }
}

impl ExprLinter for TryingToDoing {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));

        if preceded_by_word(context, |t| {
            t.get_ch(source)
                .eq_any_ignore_ascii_case_chars(&[&['f', 'r', 'o', 'm'], &['w', 'h', 'e', 'n']])
        }) {
            return None;
        }

        let span = matched_tokens.last()?.span;

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "correction",
            span.get_content(source),
        )];

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions,
            message: "When the verb before `to` ends with `-ing`, the verb after `to` should be the base form.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags when verbs both before and after `to` end in `-ing`, such as `trying to doing` instead of `trying to do`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::TryingToDoing;

    #[test]
    fn need_think() {
        assert_suggestion_result(
            "When You Are Tired of Always Needing To Thinking About Your Weight.",
            TryingToDoing::default(),
            "When You Are Tired of Always Needing To Think About Your Weight.",
        );
    }

    #[test]
    fn try_compete() {
        assert_suggestion_result(
            "if this is trying to competing with v8/llm",
            TryingToDoing::default(),
            "if this is trying to compete with v8/llm",
        );
    }

    #[test]
    fn try_live() {
        assert_suggestion_result(
            "We spend a lot of time trying to living in the world and surviving in it",
            TryingToDoing::default(),
            "We spend a lot of time trying to live in the world and survive in it",
        );
    }

    #[test]
    fn dont_flag_after_from() {
        assert_no_lints("From needing to wanting.", TryingToDoing::default());
    }
}
