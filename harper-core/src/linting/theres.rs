use crate::{
    CharStringExt, Token,
    expr::SequenceExpr,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct Theres {
    expr: Box<dyn crate::expr::Expr>,
}

impl Default for Theres {
    fn default() -> Self {
        let expr = SequenceExpr::aco("their's")
            .t_ws()
            .then(|tok: &Token, src: &[char]| {
                tok.kind.is_determiner()
                    || tok.kind.is_quantifier()
                    || tok.span.get_content(src).eq_ignore_ascii_case_str("no")
                    || tok.span.get_content(src).eq_ignore_ascii_case_str("enough")
            });

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for Theres {
    fn expr(&self) -> &dyn crate::expr::Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offender = tokens.first()?;
        let span = offender.span;
        let template = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str("there's", template)],
            message: "Use `there's`—the contraction of “there is”—for this construction.".into(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Replaces the mistaken possessive `their's` before a determiner with the contraction `there's`."
    }
}

#[cfg(test)]
mod tests {
    use super::Theres;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_lowercase_before_the() {
        assert_suggestion_result(
            "We realized their's the clue we missed.",
            Theres::default(),
            "We realized there's the clue we missed.",
        );
    }

    #[test]
    fn corrects_sentence_start() {
        assert_suggestion_result(
            "Their's the solution on the table.",
            Theres::default(),
            "There's the solution on the table.",
        );
    }

    #[test]
    fn corrects_before_no() {
        assert_suggestion_result(
            "I promise their's no extra charge.",
            Theres::default(),
            "I promise there's no extra charge.",
        );
    }

    #[test]
    fn corrects_before_an() {
        assert_suggestion_result(
            "I suspect their's an error in the log.",
            Theres::default(),
            "I suspect there's an error in the log.",
        );
    }

    #[test]
    fn corrects_before_a() {
        assert_suggestion_result(
            "Maybe their's a better route available.",
            Theres::default(),
            "Maybe there's a better route available.",
        );
    }

    #[test]
    fn corrects_before_another() {
        assert_suggestion_result(
            "Their's another round after this.",
            Theres::default(),
            "There's another round after this.",
        );
    }

    #[test]
    fn corrects_before_enough() {
        assert_suggestion_result(
            "Their's enough context in the report.",
            Theres::default(),
            "There's enough context in the report.",
        );
    }

    #[test]
    fn allows_possessive_pronoun_form() {
        assert_lint_count("Theirs is the final draft.", Theres::default(), 0);
    }

    #[test]
    fn ignores_without_determiner_afterward() {
        assert_lint_count("I think their's better already.", Theres::default(), 0);
    }

    #[test]
    fn ignores_correct_contraction() {
        assert_lint_count("There's a bright sign ahead.", Theres::default(), 0);
    }
}
