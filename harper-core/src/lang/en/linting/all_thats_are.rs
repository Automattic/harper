use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
    token_string_ext::TokenStringExt,
};

pub struct AllThatsAre {
    expr: SequenceExpr,
}

impl Default for AllThatsAre {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("all")
                .t_ws()
                .then_any_of([
                    Box::new(Word::new("that's")) as Box<dyn Expr>,
                    Box::new(SequenceExpr::word_seq(&["that", "is"])),
                ])
                .t_ws()
                .then_adjective()
                .t_ws() // or past participle? should happen automatically
                .t_aco("are"),
        }
    }
}

impl ExprLinter for AllThatsAre {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // 7 for "that's" and 9 for "that is", otherwise the Expr is no longer in sync with this function
        match matched_tokens.len() {
            7 | 9 => {}
            _ => return None,
        }

        let span = matched_tokens.last()?.span;

        let mut are_to_is = span.get_content(source).to_vec();

        // if the last word is not "are" then the Expr is out of sync with this function
        if are_to_is.len() != 3 {
            return None;
        }

        // We can directly access the elements because we checked the length
        let a_to_i_delta = b'i' - b'a';
        are_to_is[0] = (are_to_is[0] as u8 + a_to_i_delta) as char;
        are_to_is[1] = (are_to_is[1] as u8 + 1) as char;
        are_to_is.pop();

        Some(Lint {
            span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::ReplaceWith(are_to_is)],
            message: format!(
                "`Are` must be singular to agree with `{}`.",
                matched_tokens.get_rel(-5)?.get_str(source)
            ),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Looks for agreement errors after `all that is` or `all that's`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::AllThatsAre;

    #[test]
    fn that_is_communicable() {
        assert_suggestion_result(
            "That is, all that is communicable are differences that make a difference, or information states.",
            AllThatsAre::default(),
            "That is, all that is communicable is differences that make a difference, or information states.",
        );
    }

    #[test]
    fn that_is_missing() {
        assert_suggestion_result(
            "All that is missing are the washerwomen",
            AllThatsAre::default(),
            "All that is missing is the washerwomen",
        );
    }

    #[test]
    fn that_is_needed() {
        assert_suggestion_result(
            "All that is needed are angle brackets.",
            AllThatsAre::default(),
            "All that is needed is angle brackets.",
        );
    }

    #[test]
    fn thats_needed() {
        assert_suggestion_result(
            "we now have a mechanically sound Golf GTI and all that's needed are some important finishing touches",
            AllThatsAre::default(),
            "we now have a mechanically sound Golf GTI and all that's needed is some important finishing touches",
        );
    }

    #[test]
    fn thats_required() {
        assert_suggestion_result(
            "All that's required are 7 ice cream sandwiches and a cup of milk.",
            AllThatsAre::default(),
            "All that's required is 7 ice cream sandwiches and a cup of milk.",
        );
    }
}
