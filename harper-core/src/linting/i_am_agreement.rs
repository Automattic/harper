use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token, TokenStringExt,
    expr::{AnchorStart, Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct IAmAgreement {
    expr: Box<dyn Expr>,
}

impl Default for IAmAgreement {
    fn default() -> Self {
        let i_are = Lrc::new(FixedPhrase::from_phrase("I are"));

        let nothing_before_i_are = SequenceExpr::with(AnchorStart).then(i_are.clone());

        let non_and_word_before_i_are = SequenceExpr::default()
            .then_word_except(&["and"])
            .t_ws()
            .then(i_are);

        let expr = FirstMatchOf::new(vec![
            Box::new(nothing_before_i_are),
            Box::new(non_and_word_before_i_are),
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for IAmAgreement {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let toks = &toks[toks.len() - 3..];
        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case(
                "I am".chars().collect(),
                toks.span()?.get_content(src),
            )],
            message: "The first-person singular pronoun `I` requires the verb form `am`; `are` belongs to second-person or plural contexts.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects `I are` to `I am`."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_i_are_simple() {
        assert_suggestion_result("I are", IAmAgreement::default(), "I am");
    }

    #[test]
    fn corrects_i_are() {
        assert_suggestion_result(
            "I are really happy about this release.",
            IAmAgreement::default(),
            "I am really happy about this release.",
        );
    }

    #[test]
    fn dont_flag_you_and_i_are() {
        assert_lint_count(
            "You know, you and I are sitting on the Titanic and the iceberg is over there.",
            IAmAgreement::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_mention_and_i_are() {
        assert_lint_count(
            "Hello, @another-rex and I are attempting to package packageurl-go for Debian as we need it for a build dependency.",
            IAmAgreement::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_z_and_i_are() {
        assert_lint_count(
            "The url is copied from a manual search, and Z and I are modified.",
            IAmAgreement::default(),
            0,
        )
    }

    #[test]
    fn dont_flag_name_and_i_are() {
        assert_lint_count(
            "Paper that Lena Baunaz and I are working on as part of my SNSF-funded 'Focus in diachrony'",
            IAmAgreement::default(),
            0,
        );
    }

    #[test]
    fn fix_so_i_are() {
        assert_suggestion_result(
            "I have not yet been able to reproduce this issue in my environment, so I are still trying to figure it out",
            IAmAgreement::default(),
            "I have not yet been able to reproduce this issue in my environment, so I am still trying to figure it out",
        );
    }

    #[test]
    fn fix_if_i_are() {
        assert_suggestion_result(
            "If i are on creative inventory, and try to clean my inventory holding shift is disconnected too.",
            IAmAgreement::default(),
            "If i am on creative inventory, and try to clean my inventory holding shift is disconnected too.",
        );
    }

    #[test]
    fn fix_what_i_are() {
        assert_suggestion_result(
            "in this situation I can't see what I are typing",
            IAmAgreement::default(),
            "in this situation I can't see what I am typing",
        );
    }

    #[test]
    fn fix_where_i_are() {
        assert_suggestion_result(
            "I have a logging application where I are append to a topic",
            IAmAgreement::default(),
            "I have a logging application where I am append to a topic",
        );
    }
}
