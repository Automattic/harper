use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

static TAKE_FORMS: &[&str] = &["take", "took", "taken", "takes", "taking"];
static HAVE_FORMS: &[&str] = &["have", "had", "has", "having"];

pub struct TakeALookTo {
    pub expr: Box<dyn Expr>,
}

impl Default for TakeALookTo {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_of(vec![
                    Box::new(WordSet::new(TAKE_FORMS)),
                    Box::new(WordSet::new(HAVE_FORMS)),
                ])
                .t_ws()
                .t_aco("a")
                .t_ws()
                .t_aco("look")
                .t_ws()
                .t_aco("to"),
            ),
        }
    }
}

impl ExprLinter for TakeALookTo {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `take a look to`/`have a look to` to correctly use `at`."
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
        let next_word_tok =
            ctx.and_then(|(_, after)| after.get(..2))
                .and_then(|tokens| match tokens {
                    [ws, next, ..] if ws.kind.is_whitespace() => Some(next),
                    _ => None,
                });

        // Exception 1. Have/take a look to see if everything is ok
        if next_word_tok.is_some_and(|nw| nw.kind.is_verb_lemma()) {
            return None;
        }

        // Exception 2. It has a look to it that I don't like
        if next_word_tok.is_some_and(|xyz| xyz.span.get_content(src).eq_ignore_ascii_case_str("it"))
            && toks.first().is_some_and(|tok| {
                tok.span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(HAVE_FORMS)
            })
        {
            return None;
        }

        let to_span = toks.last()?.span;

        Some(Lint {
            lint_kind: LintKind::Usage,
            span: to_span,
            suggestions: vec![Suggestion::replace_with_match_case(
                vec!['a', 't'],
                to_span.get_content(src),
            )],
            message: "This phrase uses `to` rather than `at`".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TakeALookTo;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn take_a_look_to_a_new() {
        assert_suggestion_result(
            "Hello, I am Drago and in this video we're going to take a look to a new AI CLI and VS Code extension tool",
            TakeALookTo::default(),
            "Hello, I am Drago and in this video we're going to take a look at a new AI CLI and VS Code extension tool",
        );
    }

    #[test]
    fn have_a_look_to_url() {
        assert_suggestion_result(
            "If you haven't yet, please have a look to https://docs.conan.io/2/devops/devops_local_recipes_index.html",
            TakeALookTo::default(),
            "If you haven't yet, please have a look at https://docs.conan.io/2/devops/devops_local_recipes_index.html",
        );
    }

    #[test]
    fn having_a_look_to_mode() {
        assert_suggestion_result(
            "Having a look to mode and overScaleMode , I see they are scriptable",
            TakeALookTo::default(),
            "Having a look at mode and overScaleMode , I see they are scriptable",
        );
    }

    #[test]
    fn taking_a_look_to_this() {
        assert_suggestion_result(
            "after taking a look to this issue and making some test I figure out that it likely to be an error",
            TakeALookTo::default(),
            "after taking a look at this issue and making some test I figure out that it likely to be an error",
        );
    }

    #[test]
    fn have_had_a_look_to_your() {
        assert_suggestion_result(
            "I have had a look to your conanfile.py and it is strange that it fails.",
            TakeALookTo::default(),
            "I have had a look at your conanfile.py and it is strange that it fails.",
        );
    }

    #[test]
    fn took_a_look_to_both() {
        assert_suggestion_result(
            "Since I have some knowledge in programing I took a look to both codes (LK and XCS)",
            TakeALookTo::default(),
            "Since I have some knowledge in programing I took a look at both codes (LK and XCS)",
        );
    }

    #[test]
    fn taken_a_look_to_that() {
        assert_suggestion_result(
            "Yeah I've taken a look to that, but I really need to use classes on this one",
            TakeALookTo::default(),
            "Yeah I've taken a look at that, but I really need to use classes on this one",
        );
    }

    #[test]
    fn takes_a_look_to_the() {
        assert_suggestion_result(
            "basically, it takes a look to the signing request",
            TakeALookTo::default(),
            "basically, it takes a look at the signing request",
        );
    }

    // Make sure we avoid potential false positives

    #[test]
    fn dont_flag_have_a_look_to_see_if() {
        assert_no_lints(
            "@budarin can you have a look to see if it addresses your concerns?",
            TakeALookTo::default(),
        );
    }

    #[test]
    fn dont_flag_taking_a_look_to_decide() {
        assert_no_lints(
            "Would be worth taking a look to decide which way to go.",
            TakeALookTo::default(),
        );
    }

    #[test]
    fn dont_flag_takes_a_look_to_see() {
        assert_no_lints(
            "It attempts to open the URL in a new window and then after 2s it takes a look to see if it can read the location.",
            TakeALookTo::default(),
        );
    }

    #[test]
    fn dont_flag_has_a_look_to_it() {
        assert_no_lints(
            "The ecosystem's UI certainly has a look to it but inside of your app you could implement a different look as long as it's consistent.",
            TakeALookTo::default(),
        );
    }

    #[test]
    fn but_dont_ignore_takes_a_look_to_it() {
        assert_suggestion_result(
            "When he gets back I hope he takes a look to it",
            TakeALookTo::default(),
            "When he gets back I hope he takes a look at it",
        );
    }
}
