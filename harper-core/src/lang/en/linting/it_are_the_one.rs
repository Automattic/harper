use crate::{
    Lint, Token,
    char_string::CharStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ItAreTheOne {
    expr: SequenceExpr,
}

impl Default for ItAreTheOne {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_word_except(&["you", "they"])
                .t_ws()
                .t_set(["are", "were"])
                .t_ws()
                .t_aco("the")
                .t_ws()
                .then_optional(SequenceExpr::aco("only").t_ws())
                .t_aco("one")
                .t_ws()
                .t_set(["which", "who", "whose", "that", "whom"]),
        }
    }
}

impl ExprLinter for ItAreTheOne {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks
            .get(if toks.get(6)?.get_ch(src).eq_str("only") {
                8
            } else {
                6
            })?
            .span;

        let mut one = span.get_content(src).to_vec();
        one.push((*one.last()? as u8 + (b's' - b'e')) as char);

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::ReplaceWith(one)],
            message: "Since `are` is plural, `ones` also needs to be plural.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "A linter skeleton for contributors to copy into `harper_core/src/linting/` and rename."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ItAreTheOne;

    #[test]
    fn are_the_one_who() {
        assert_suggestion_result(
            "Amateurs are the one who argue about syntax.",
            ItAreTheOne::default(),
            "Amateurs are the ones who argue about syntax.",
        );
    }

    #[test]
    fn are_the_only_one_who() {
        assert_suggestion_result(
            "those approved addressed are the only one who can change the rate of the token",
            ItAreTheOne::default(),
            "those approved addressed are the only ones who can change the rate of the token",
        );
    }

    #[test]
    fn are_the_one_which() {
        assert_suggestion_result(
            "graphics and gfx folder are the one which have different svgs and jpg files",
            ItAreTheOne::default(),
            "graphics and gfx folder are the ones which have different svgs and jpg files",
        );
    }

    #[test]
    fn were_the_only_one_that() {
        assert_suggestion_result(
            "SGN were the only one that actually sent me a sensible map",
            ItAreTheOne::default(),
            "SGN were the only ones that actually sent me a sensible map",
        );
    }

    #[test]
    fn we_are_the_one_who() {
        assert_suggestion_result(
            "They call us Dremers, but we are the one who don't sleep.",
            ItAreTheOne::default(),
            "They call us Dremers, but we are the ones who don't sleep.",
        );
    }

    #[test]
    fn are_the_one_that() {
        assert_suggestion_result(
            "things you don't want to share are the one that would interest your friends the most",
            ItAreTheOne::default(),
            "things you don't want to share are the ones that would interest your friends the most",
        );
    }
}
