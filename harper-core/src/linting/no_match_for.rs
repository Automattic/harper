use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{InflectionOfBe, WordSet},
};

pub struct NoMatchFor {
    expr: Box<dyn Expr>,
}

impl Default for NoMatchFor {
    fn default() -> Self {
        let pre_context = FirstMatchOf::new(vec![
            Box::new(InflectionOfBe::default()),
            Box::new(WordSet::new(&[
                "I'm", "we're", "you're", "he's", "she's", "it's", "they're", "Im", "were",
                "youre", "hes", "shes", "its", "theyre",
            ])),
        ]);

        let expr = SequenceExpr::with(pre_context)
            .then_whitespace()
            .t_aco("no")
            .then_whitespace()
            .t_aco("match")
            .then_whitespace()
            .then_preposition();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for NoMatchFor {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_tok = toks.last()?;
        let prep_chars = prep_tok.span.get_content(src);
        if prep_chars.eq_ignore_ascii_case_chars(&['f', 'o', 'r']) {
            return None;
        }

        let phrase_toks = &toks[2..];
        let phrase_span = phrase_toks.span()?;

        let suggestion =
            Suggestion::replace_with_match_case_str("no match for", phrase_span.get_content(src));

        Some(Lint {
            span: phrase_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![suggestion],
            message: "If you mean the idiom, it's `no match for`.".to_owned(),
            priority: 55,
        })
    }

    fn description(&self) -> &str {
        "No match for"
    }
}

#[cfg(test)]
pub mod tests {
    use super::NoMatchFor;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_against() {
        assert_suggestion_result(
            "Erlang was no match against my sweeping gale.",
            NoMatchFor::default(),
            "Erlang was no match for my sweeping gale.",
        );
    }

    #[test]
    fn fix_to() {
        assert_suggestion_result(
            "My BW5 was no match to his BW7.",
            NoMatchFor::default(),
            "My BW5 was no match for his BW7.",
        );
    }

    #[test]
    fn fix_of() {
        assert_suggestion_result(
            "This Attack Plane Was No Match Of Me So I Did This To Him",
            NoMatchFor::default(),
            "This Attack Plane Was No Match For Me So I Did This To Him",
        );
    }

    #[test]
    fn fix_its_to() {
        assert_suggestion_result(
            "cuz AI is bull crap and its no match to human voice",
            NoMatchFor::default(),
            "cuz AI is bull crap and its no match for human voice",
        );
    }

    #[test]
    fn fix_im_to() {
        assert_suggestion_result(
            "Im no match to you but like let me no what u think",
            NoMatchFor::default(),
            "Im no match for you but like let me no what u think",
        );
    }

    #[test]
    fn theyre_to() {
        assert_suggestion_result(
            "Theyre no match to late 60s early 70s sansuis.",
            NoMatchFor::default(),
            "Theyre no match for late 60s early 70s sansuis.",
        );
    }

    #[test]
    fn fix_hes_to() {
        assert_suggestion_result(
            "Even ouki on drinks with renpa said hes no match to him.",
            NoMatchFor::default(),
            "Even ouki on drinks with renpa said hes no match for him.",
        );
    }

    #[test]
    fn fix_shes_to() {
        assert_suggestion_result(
            "Izma tries to struggle but she's no match to your superior strength",
            NoMatchFor::default(),
            "Izma tries to struggle but she's no match for your superior strength",
        );
    }

    #[test]
    fn dont_fix_for() {
        assert_lint_count(
            "Type to search appears even there is no match for search term when autoFocus is true.",
            NoMatchFor::default(),
            0,
        );
    }
}
