use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct HaveTheNerve {
    expr: Box<dyn Expr>,
}

impl Default for HaveTheNerve {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["had", "have", "having", "has"])
                    .t_ws()
                    .t_aco("the")
                    .t_ws()
                    .then_word_set(&["nerve", "nerves"])
                    .t_ws()
                    .t_aco("to"),
            ),
        }
    }
}

impl ExprLinter for HaveTheNerve {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Flags `have the nerves` used for audacity or `have the nerve` used for patience."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let apprehensive_emoji = "😰";
        let rude_emoji = "😠";

        // I can't believe they had the nerve to do that! -> rude emoji
        // Oh I don't have the nerves for that kind of thing! -> apprehensive emoji
        let emoji = if toks
            .get_rel(-3)?
            .span
            .get_content(src)
            .ends_with_ignore_ascii_case_chars(&['s'])
        {
            apprehensive_emoji
        } else {
            rude_emoji
        };

        eprintln!("{emoji} {}", format_lint_match(toks, ctx, src));
        None
    }
}

#[cfg(test)]
mod tests {
    use super::HaveTheNerve;
    use crate::linting::tests::{assert_lint_count, assert_no_lints};

    // Flag audacity with plural "nerves"

    #[test]
    fn flag_the_nerves_to_ask() {
        assert_lint_count(
            "How the hell he has the nerves to ask more than triple?!",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_the_nerves_to_tell_us() {
        assert_lint_count(
            "And yet you have the nerves to tell us that \"there is no EVE community\".",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_she_had_the_nerves() {
        assert_lint_count(
            "She had the nerves to t-pose on me.",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_has_the_nerves_to_say() {
        assert_lint_count(
            "Nagumo has the Nerves to say this when he cracks the most unfunniest joke",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_had_the_nerves_to_mock() {
        assert_lint_count(
            "That Frank even had the nerves to mock us towards the end",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_they_still_have_the_nerve() {
        assert_lint_count(
            "and they still have the nerves to say that \"Carti carried\"",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn flag_the_nerves_to_thumbs_down() {
        assert_lint_count(
            "Bro had the nerves to thumbs down afterwards.",
            HaveTheNerve::default(),
            1,
        );
    }

    // Dont' flag lacking courage or patience

    #[test]
    fn dont_flag_wont_have_the_nerves() {
        assert_no_lints(
            "But it's likely that you won't have the nerves to change the whole DAO layer afterwards",
            HaveTheNerve::default(),
        );
    }

    #[test]
    fn dont_flag_might_have_the_nerves() {
        assert_lint_count(
            "Nevermind, someone might have the nerves to go through it",
            HaveTheNerve::default(),
            1,
        );
    }

    #[test]
    fn dont_flag_have_need_to_have_the_nerves() {
        assert_no_lints(
            "The only thing you need is to have the nerves to embrace OpenGL.",
            HaveTheNerve::default(),
        );
    }

    #[test]
    fn dont_flag_didnt_had_the_nerves() {
        assert_no_lints("i didnt had the nerves to try ZF2", HaveTheNerve::default());
    }

    #[test]
    fn dont_flag_has_the_nerves_to_help() {
        assert_no_lints(
            "It surely still needs work, and possibly someone has the nerves to help editing it.",
            HaveTheNerve::default(),
        );
    }

    #[test]
    fn dont_flag_the_nerves_to_see_it_through() {
        assert_no_lints(
            "This one is relatively simple just to see if I had the nerves to see it through to the end.",
            HaveTheNerve::default(),
        );
    }

    // Don't flag literal uses

    #[test]
    fn dont_flag_the_nerves_to_their_ears() {
        assert_no_lints(
            "Some dressage horses have the nerves to their ears blocked/cut",
            HaveTheNerve::default(),
        );
    }
}
