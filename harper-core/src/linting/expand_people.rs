use crate::{
    Lint, Token, TokenStringExt,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct ExpandPeople {
    expr: Word,
}

impl Default for ExpandPeople {
    fn default() -> Self {
        Self {
            expr: Word::new_exact("ppl"),
        }
    }
}

impl ExprLinter for ExpandPeople {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;
        let lint_kind = LintKind::Style;
        let suggestions = vec![Suggestion::replace_with_match_case_str(
            "people",
            span.get_content(src),
        )];
        let message = "Use `people` instead of `ppl`.".to_string();
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
        "Expands the abbreviation `ppl` to the full word `people` for clarity."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ExpandPeople;

    #[test]
    fn fix_some_people() {
        assert_suggestion_result(
            "some ppl told my this problem from (ImGui_ImplWin32_WndProcHandler) and u need to add (WM_SIZE) but i don't know what should i do now :(",
            ExpandPeople::default(),
            "some people told my this problem from (ImGui_ImplWin32_WndProcHandler) and u need to add (WM_SIZE) but i don't know what should i do now :(",
        );
    }

    #[test]
    fn fix_all_people() {
        assert_suggestion_result(
            "Hi all, maybe all ppl with some experience on R, would know is not easy to debug or work with a language where there is no checks on types",
            ExpandPeople::default(),
            "Hi all, maybe all people with some experience on R, would know is not easy to debug or work with a language where there is no checks on types",
        );
    }

    #[test]
    fn dont_flag_protected_process_light() {
        assert_no_lints(
            "Processes started as an Anti Malware 'Protected Process-Light' (PPL) are restricted in what they can do, can only load signed code, but cannot be debugged",
            ExpandPeople::default(),
        );
    }

    #[test]
    fn dont_flag_paired_point_lifting() {
        assert_no_lints(
            "In this work, we present an alternative lightweight strategy called Paired-Point Lifting (PPL) for constructing 3D line clouds.",
            ExpandPeople::default(),
        );
    }
}
