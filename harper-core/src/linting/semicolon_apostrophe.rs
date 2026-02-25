use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

const CONTRACTION_AND_POSSESSIVE_ENDINGS: [&str; 7] = ["d", "ll", "m", "re", "s", "t", "ve"];

pub struct SemicolonApostrophe {
    expr: Box<dyn Expr>,
}

impl Default for SemicolonApostrophe {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_word()
                    .then_semicolon()
                    .then_word_set(&CONTRACTION_AND_POSSESSIVE_ENDINGS),
            ),
        }
    }
}

impl ExprLinter for SemicolonApostrophe {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let whole_span = toks.span()?;
        let base = &toks.first()?;
        let ending = &toks.last()?;

        let replacement_str = format!(
            "{}'{}",
            base.span.get_content_string(src).to_lowercase(),
            ending.span.get_content_string(src).to_lowercase()
        );

        let mut lettercase_template = base.span.get_content(src).to_vec();
        lettercase_template.extend_from_slice(ending.span.get_content(src));

        Some(Lint {
            span: whole_span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                replacement_str.chars().collect(),
                &lettercase_template,
            )],
            message: format!("Did you mean `{replacement_str}`?"),
            priority: 57,
        })
    }

    fn description(&self) -> &str {
        "Corrects semicolons accidentally typed instead of apostrophes."
    }
}

#[cfg(test)]
mod tests {
    use super::SemicolonApostrophe;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_dont_with_semicolon_to_apostrophe() {
        assert_suggestion_result(
            "It's better if you don;t type like this.",
            SemicolonApostrophe::default(),
            "It's better if you don't type like this.",
        );
    }

    #[test]
    fn ignore_correct() {
        assert_lint_count("I don't doubt it.", SemicolonApostrophe::default(), 0);
    }

    #[test]
    fn fix_title_case() {
        assert_suggestion_result(
            "Don;t type like this.",
            SemicolonApostrophe::default(),
            "Don't type like this.",
        );
    }

    #[test]
    fn fix_all_caps() {
        assert_suggestion_result(
            "DON;T TRY THIS AT HOME.",
            SemicolonApostrophe::default(),
            "DON'T TRY THIS AT HOME.",
        );
    }

    #[test]
    #[ignore = "replace_with_match_case has a bug turning `I'll` into `I'LL`"]
    fn fix_ill_and_monkeys() {
        assert_suggestion_result(
            "Well I;ll be a monkey;s uncle!",
            SemicolonApostrophe::default(),
            "Well I'll be a monkey's uncle!",
        )
    }

    #[test]
    fn fix_other_contractions_and_possessives() {
        assert_suggestion_result(
            "Let;s see if we;ve fixed patrakov;s bug. Fun wasn;t it?",
            SemicolonApostrophe::default(),
            "Let's see if we've fixed patrakov's bug. Fun wasn't it?",
        )
    }

    #[test]
    fn corrects_ive_with_correct_capitalization() {
        assert_suggestion_result("I;ve", SemicolonApostrophe::default(), "I've");
    }
}
