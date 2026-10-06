use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct SuchShame {
    expr: SequenceExpr,
}

impl Default for SuchShame {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["is", "it's", "its", "that's", "thats", "was"])
                .t_ws()
                .then_word_seq(&["such", "shame"]),
        }
    }
}

impl ExprLinter for SuchShame {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let (first, such, ws) = (0, 2, 3);
        let (first, such, ws) = (toks.get(first)?, toks.get(such)?, toks.get(ws)?);

        if first
            .get_ch(src)
            .eq_any_ignore_ascii_case_chars(&[&['i', 's'], &['w', 'a', 's']])
            && preceded_by_word(ctx, |t| {
                t.get_ch(src).eq_any_ignore_ascii_case_chars(&[
                    &['t', 'h', 'e', 'r', 'e'],
                    &['t', 'h', 'e', 'i', 'r'],
                ])
            })
        {
            return None;
        }

        let c = *such.get_ch(src).get(2)?;
        let a = (c as u8 - 2) as char;

        let mut new_chars = vec![a];
        new_chars.extend(ws.get_ch(src));

        Some(Lint {
            span: ws.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::InsertAfter(new_chars)],
            message: "This expression requires `a shame`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `such shame` to `such a shame`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::SuchShame;

    #[test]
    fn its_apostrope() {
        assert_suggestion_result(
            "it's such shame that for large datasets the usage of this wonderful package becomes not pracitcal",
            SuchShame::default(),
            "it's such a shame that for large datasets the usage of this wonderful package becomes not pracitcal",
        );
    }

    #[test]
    fn it_is() {
        assert_suggestion_result(
            "It is such shame that the TV adaptation changed this fundamental part of the story for no reason",
            SuchShame::default(),
            "It is such a shame that the TV adaptation changed this fundamental part of the story for no reason",
        );
    }

    #[test]
    fn dont_flag_there_was() {
        assert_no_lints(
            "but I think there was such shame in that at the time most conversations on the subject would be ...",
            SuchShame::default(),
        );
    }

    #[test]
    fn dont_flag_there_is() {
        assert_no_lints(
            "There is such shame that seems to come from not being able to handle your house.",
            SuchShame::default(),
        );
    }

    #[test]
    fn it_was() {
        assert_suggestion_result(
            "It was such shame to lose such a talented composer.",
            SuchShame::default(),
            "It was such a shame to lose such a talented composer.",
        );
    }

    #[test]
    fn its_no_apostrophe() {
        assert_suggestion_result(
            "Its such shame to because this update had such potential",
            SuchShame::default(),
            "Its such a shame to because this update had such potential",
        );
    }

    #[test]
    fn dont_flag_their_was() {
        assert_no_lints(
            "Partially because their was such shame among black people if they had darker skin. ",
            SuchShame::default(),
        );
    }
}
