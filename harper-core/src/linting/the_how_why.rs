use crate::expr::Expr;
use crate::expr::FirstMatchOf;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Suggests removing `the` when followed by how/why/who/when/what,
/// skipping cases like `how to` and `who's who`.
pub struct TheHowWhy {
    expr: FirstMatchOf,
}

impl Default for TheHowWhy {
    fn default() -> Self {
        let the_how = SequenceExpr::default()
            .t_aco("the")
            .then_whitespace()
            .t_aco("how")
            .then_unless(SequenceExpr::whitespace().t_aco("to"));

        let the_who = SequenceExpr::default()
            .t_aco("the")
            .then_whitespace()
            .t_aco("who")
            .then_unless(
                SequenceExpr::whitespace()
                    .t_aco("'s")
                    .then_whitespace()
                    .t_aco("who"),
            );

        let the_why = SequenceExpr::default()
            .t_aco("the")
            .then_whitespace()
            .t_aco("why");

        let the_when = SequenceExpr::default()
            .t_aco("the")
            .then_whitespace()
            .t_aco("when");

        let the_what = SequenceExpr::default()
            .t_aco("the")
            .then_whitespace()
            .t_aco("what");

        let expr = FirstMatchOf::new(vec![
            Box::new(the_how),
            Box::new(the_who),
            Box::new(the_why),
            Box::new(the_when),
            Box::new(the_what),
        ]);

        Self { expr }
    }
}

impl ExprLinter for TheHowWhy {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let the_token_span = matched_tokens[0..2].span()?;
        let question_word_token = matched_tokens.get(2)?;
        let question_word = question_word_token.span.get_content(source);

        Some(Lint {
            span: the_token_span,
            lint_kind: LintKind::Miscellaneous,
            message: format!(
                "Remove `the` before `{}`. In most contexts, `{}` alone is clearer.",
                question_word.iter().collect::<String>(),
                question_word.iter().collect::<String>()
            ),
            suggestions: vec![Suggestion::Remove],
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Removes the extra `the` from expressions like `the how`, skipping `how to` and `who's who`."
    }
}

#[cfg(test)]
mod tests {
    use super::TheHowWhy;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn basic_the_how() {
        assert_suggestion_result(
            "This is the how it all started.",
            TheHowWhy::default(),
            "This is how it all started.",
        );
    }

    #[test]
    fn the_why() {
        assert_suggestion_result(
            "The important part is the why it matters.",
            TheHowWhy::default(),
            "The important part is why it matters.",
        );
    }

    #[test]
    fn skip_how_to() {
        assert_lint_count(
            "I'd like to explain the how to install this properly.",
            TheHowWhy::default(),
            0,
        );
    }

    #[test]
    fn skip_whos_who() {
        assert_lint_count(
            "We covered the who's who of corporate leadership last time.",
            TheHowWhy::default(),
            0,
        );
    }

    #[test]
    fn the_who() {
        assert_suggestion_result(
            "We must identify the who is responsible.",
            TheHowWhy::default(),
            "We must identify who is responsible.",
        );
    }

    #[test]
    fn the_when() {
        assert_suggestion_result(
            "He outlined the when the new phase will start.",
            TheHowWhy::default(),
            "He outlined when the new phase will start.",
        );
    }

    #[test]
    fn the_what() {
        assert_suggestion_result(
            "The presentation clarifies the what we intend to build.",
            TheHowWhy::default(),
            "The presentation clarifies what we intend to build.",
        );
    }

    #[test]
    fn no_false_positive() {
        assert_lint_count(
            "These tips examine the how to fix your code quickly, plus the what's next.",
            TheHowWhy::default(),
            0,
        );
    }
}
