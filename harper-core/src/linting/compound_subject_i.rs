use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenKind,
    expr::{AnchorStart, Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

const POSSESSIVE_DETERMINERS: &[&str] = &["my", "your", "her", "his", "their", "our"];

pub struct CompoundSubjectI {
    expr: Box<dyn Expr>,
}

impl Default for CompoundSubjectI {
    fn default() -> Self {
        let expr = SequenceExpr::with(AnchorStart)
            .then_optional(
                SequenceExpr::default()
                    .then_quote()
                    .then_optional(SequenceExpr::default().t_ws()),
            )
            .then_optional(
                SequenceExpr::default()
                    .then_punctuation()
                    .then_optional(SequenceExpr::default().t_ws()),
            )
            .then_word_set(POSSESSIVE_DETERMINERS)
            .t_ws()
            .then_nominal()
            .t_ws()
            .t_aco("and")
            .t_ws()
            .t_aco("me")
            .t_ws()
            .then_kind_either(TokenKind::is_verb, TokenKind::is_auxiliary_verb);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for CompoundSubjectI {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let pronoun = matched_tokens.iter().find(|tok| {
            tok.kind.is_word()
                && tok
                    .span
                    .get_content_string(source)
                    .eq_ignore_ascii_case("me")
        })?;
        Some(Lint {
            span: pronoun.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith("I".chars().collect())],
            message: "Use `I` when this pronoun is part of a compound subject.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Promotes `I` in compound subjects headed by a possessive determiner."
    }
}

#[cfg(test)]
mod tests {
    use super::CompoundSubjectI;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_my_mother_and_me() {
        assert_suggestion_result(
            "My mother and me went to California.",
            CompoundSubjectI::default(),
            "My mother and I went to California.",
        );
    }

    #[test]
    fn corrects_my_brother_and_me() {
        assert_suggestion_result(
            "My brother and me would often go to the cinema.",
            CompoundSubjectI::default(),
            "My brother and I would often go to the cinema.",
        );
    }

    #[test]
    fn corrects_your_friend_and_me() {
        assert_suggestion_result(
            "Your friend and me are heading out.",
            CompoundSubjectI::default(),
            "Your friend and I are heading out.",
        );
    }

    #[test]
    fn corrects_her_manager_and_me() {
        assert_suggestion_result(
            "Her manager and me have talked about it.",
            CompoundSubjectI::default(),
            "Her manager and I have talked about it.",
        );
    }

    #[test]
    fn corrects_his_cat_and_me() {
        assert_suggestion_result(
            "His cat and me were inseparable.",
            CompoundSubjectI::default(),
            "His cat and I were inseparable.",
        );
    }

    #[test]
    fn corrects_their_kids_and_me() {
        assert_suggestion_result(
            "Their kids and me will play outside.",
            CompoundSubjectI::default(),
            "Their kids and I will play outside.",
        );
    }

    #[test]
    fn corrects_our_neighbor_and_me() {
        assert_suggestion_result(
            "Our neighbor and me can help tomorrow.",
            CompoundSubjectI::default(),
            "Our neighbor and I can help tomorrow.",
        );
    }

    #[test]
    fn corrects_with_quote_prefix() {
        assert_suggestion_result(
            "\"My mother and me went to California,\" she said.",
            CompoundSubjectI::default(),
            "\"My mother and I went to California,\" she said.",
        );
    }

    #[test]
    fn corrects_all_caps() {
        assert_suggestion_result(
            "MY BROTHER AND ME WILL HANDLE IT.",
            CompoundSubjectI::default(),
            "MY BROTHER AND I WILL HANDLE IT.",
        );
    }

    #[test]
    fn ignores_between_you_and_me() {
        assert_lint_count(
            "Between you and me, this stays here.",
            CompoundSubjectI::default(),
            0,
        );
    }

    #[test]
    fn ignores_comma_after_me() {
        assert_lint_count(
            "My mother and me, as usual, went to the park.",
            CompoundSubjectI::default(),
            0,
        );
    }
}
