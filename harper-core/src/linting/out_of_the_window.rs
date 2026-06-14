use crate::{
    EnglishDialect, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct OutOfTheWindow {
    expr: SequenceExpr,
    dialect: EnglishDialect,
}

impl OutOfTheWindow {
    pub fn new(dialect: EnglishDialect) -> Self {
        Self {
            expr: SequenceExpr::aco("out")
                .then_optional(SequenceExpr::whitespace().t_aco("of"))
                .t_ws()
                .t_aco("the")
                .t_ws()
                .t_aco("window"),
            dialect,
        }
    }
}

impl ExprLinter for OutOfTheWindow {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let (span, sugg, msg) = match (toks.len(), self.dialect) {
            (7, EnglishDialect::American | EnglishDialect::Australian | EnglishDialect::Canadian) => {
                (
                    toks[1..=2].span()?,
                    Suggestion::Remove,
                    format!("If this is the idiom about abandoning a plan, it's more usual to leave out `of` in {} English", self.dialect),
                )
            }
            (5, EnglishDialect::British) => {
                (
                    toks[0].span,
                    Suggestion::InsertAfter(vec![' ', 'o', 'f']),
                    "If this is the idiom about abandoning a plan, consider using `of` in British English".to_string(),
                )
            }
            _ => return None,
        };

        Some(Lint {
            span,
            lint_kind: LintKind::Regionalism,
            suggestions: vec![sugg],
            message: msg,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter for the idiom `out (of) the window`."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        EnglishDialect,
        linting::tests::{assert_lint_message, assert_no_lints, assert_suggestion_result},
    };

    use super::OutOfTheWindow;

    #[test]
    fn us_english() {
        assert_suggestion_result(
            "The whole Gnome session goes out of the window instantly, both upon explicit screen locking and upon automatic locking after a timeout.",
            OutOfTheWindow::new(EnglishDialect::American),
            "The whole Gnome session goes out the window instantly, both upon explicit screen locking and upon automatic locking after a timeout.",
        );
    }

    #[test]
    fn uk_english() {
        assert_suggestion_result(
            "determinism went out the window, everything is terrible",
            OutOfTheWindow::new(EnglishDialect::British),
            "determinism went out of the window, everything is terrible",
        );
    }

    #[test]
    fn in_english_doesnt_flag_with_of() {
        assert_no_lints(
            "errors and orchestration in general all go out of the window",
            OutOfTheWindow::new(EnglishDialect::Indian),
        );
    }

    #[test]
    fn in_english_doesnt_flag_without_of() {
        assert_no_lints(
            "I was so excited to develop my new app in .NET MAUI but then all this excitement went out the window due to two things stability and tooling.",
            OutOfTheWindow::new(EnglishDialect::Indian),
        );
    }

    #[test]
    fn gives_the_right_message_for_canadian_english() {
        assert_lint_message(
            "This all works great unless I use middlware and then it all goes out the window.",
            OutOfTheWindow::new(EnglishDialect::Canadian),
            "If this is the idiom about abandoning a plan, it's more usual to leave out `of` in Canadian English",
        );
    }
}
