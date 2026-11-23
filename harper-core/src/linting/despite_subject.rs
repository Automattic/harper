use crate::{
    Lint, Token, TokenKind,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct DespiteSubject {
    expr: SequenceExpr,
}

impl Default for DespiteSubject {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("despite").t_ws().then_kind_is_but_is_not(
                TokenKind::is_subject_pronoun,
                TokenKind::is_object_pronoun,
            ),
        }
    }
}

impl ExprLinter for DespiteSubject {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = toks.first()?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("although", tok.get_ch(src)),
                Suggestion::replace_with_match_case_str("even though", tok.get_ch(src)),
            ],
            message: "Avoid using `despite` as a conjunction. `Despite` is a preposition and must be followed by a noun phrase or gerund (-ing verb). Use `although` to introduce a clause, or add `the fact that` after `despite`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags incorrect usage of the preposition `despite` when it is mistakenly used as a conjunction introducing a subjective clause (e.g., `despite I tried` instead of `although I tried`, `despite trying`, or `despite the fact that I tried`)."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::DespiteSubject;

    #[test]
    fn i() {
        assert_suggestion_result(
            "Why LSTM give different output despite I use the same code?",
            DespiteSubject::default(),
            "Why LSTM give different output even though I use the same code?",
        );
    }

    #[test]
    fn we() {
        assert_suggestion_result(
            "Why do we need dynamic memory allocation despite we can use variable-length arrays instead?",
            DespiteSubject::default(),
            "Why do we need dynamic memory allocation even though we can use variable-length arrays instead?",
        );
    }

    #[test]
    fn he() {
        assert_suggestion_result(
            "More then 30% of copilot responses are utter nonsense despite he has created more then 10",
            DespiteSubject::default(),
            "More then 30% of copilot responses are utter nonsense although he has created more then 10",
        );
    }

    #[test]
    fn she() {
        assert_lint_count(
            "insists that she is a normal girl despite she have a lots of robotic gear",
            DespiteSubject::default(),
            1,
        );
    }

    #[test]
    fn they() {
        assert_suggestion_result(
            "Jobs marked failed despite they succeeded",
            DespiteSubject::default(),
            "Jobs marked failed although they succeeded",
        );
    }
}
