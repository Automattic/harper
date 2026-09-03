use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct EverPronounRelPronoun {
    expr: SequenceExpr,
}

impl Default for EverPronounRelPronoun {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                "whatever",
                "whenever",
                "wherever",
                "whichever",
                "whoever",
            ])
            .t_ws()
            .then_any_of([
                Box::new(SequenceExpr::aco("that").t_ws().then_any_of([
                    Box::new(SequenceExpr::default().then_subject_pronoun()) as Box<dyn Expr>,
                    Box::new(WordSet::new(&[
                        "I've", "we've", "you've", "he's", "she's", "it's", "they've",
                    ])),
                ])) as Box<dyn Expr>,
                Box::new(WordSet::new(&["which", "who", "whom"])),
            ]),
        }
    }
}

impl ExprLinter for EverPronounRelPronoun {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks.get(1..=2)?.span()?,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![Suggestion::Remove],
            message: "The relative pronoun is not needed here.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Removes unnecessary redundant relative pronoun after `whatever`, `whoever`, etc."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::EverPronounRelPronoun;

    #[test]
    fn whatever_that_she() {
        assert_suggestion_result(
            "She had no evidence of colitis flare up, meaning, whatever that she has now, is what she had at the first  emergency room visit.",
            EverPronounRelPronoun::default(),
            "She had no evidence of colitis flare up, meaning, whatever she has now, is what she had at the first  emergency room visit.",
        );
    }

    #[test]
    fn whatever_that_i() {
        assert_suggestion_result(
            "Whatever that I have already tried gives back attribute errors etc.",
            EverPronounRelPronoun::default(),
            "Whatever I have already tried gives back attribute errors etc.",
        );
    }

    #[test]
    fn whatever_that_ive_transcribed() {
        assert_suggestion_result(
            "the history in the handy app gets updated with whatever that I've transcribed",
            EverPronounRelPronoun::default(),
            "the history in the handy app gets updated with whatever I've transcribed",
        );
    }

    #[test]
    fn whatever_that_i_experience() {
        assert_suggestion_result(
            "whatever that I experience is, I want it to \"just work\" with VS Code",
            EverPronounRelPronoun::default(),
            "whatever I experience is, I want it to \"just work\" with VS Code",
        );
    }

    #[test]
    fn use_whatever_which() {
        assert_suggestion_result(
            "The git hooks would use whatever which dvc is set on the shell.",
            EverPronounRelPronoun::default(),
            "The git hooks would use whatever dvc is set on the shell.",
        );
    }

    #[test]
    fn do_whatever_which() {
        assert_suggestion_result(
            "When I load my page I have an auto focus and I can type and do whatever which works fine.",
            EverPronounRelPronoun::default(),
            "When I load my page I have an auto focus and I can type and do whatever works fine.",
        );
    }

    #[test]
    fn whatever_who() {
        assert_suggestion_result(
            "You can use cURL for that (or whatever who can make a HTTP request).",
            EverPronounRelPronoun::default(),
            "You can use cURL for that (or whatever can make a HTTP request).",
        );
    }

    #[test]
    fn whenever_that_ive_created_reference() {
        assert_suggestion_result(
            "or whenever that i create a reference to a class and that Class adds tons of stacks",
            EverPronounRelPronoun::default(),
            "or whenever i create a reference to a class and that Class adds tons of stacks",
        );
    }

    #[test]
    fn whenever_that_i_want() {
        assert_suggestion_result(
            "Any special scenario, or like JSON whenever that I want to transport data?",
            EverPronounRelPronoun::default(),
            "Any special scenario, or like JSON whenever I want to transport data?",
        );
    }

    #[test]
    fn whoever_who_needs() {
        assert_suggestion_result(
            "A public motivation site for whoever who needs inspiration to get some PIZZA!!!",
            EverPronounRelPronoun::default(),
            "A public motivation site for whoever needs inspiration to get some PIZZA!!!",
        );
    }

    #[test]
    fn whoever_who_owns() {
        assert_suggestion_result(
            "We'd need permission from whoever who owns the legal/moral rights to the Python logo if we want to use it",
            EverPronounRelPronoun::default(),
            "We'd need permission from whoever owns the legal/moral rights to the Python logo if we want to use it",
        );
    }

    #[test]
    fn whoever_who_read() {
        assert_suggestion_result(
            "Hi, I'm asking to whoever who read this for help.",
            EverPronounRelPronoun::default(),
            "Hi, I'm asking to whoever read this for help.",
        );
    }

    // Known false positives

    #[test]
    #[ignore = "'whatever' means 'etc.', hopefully rare"]
    fn dont_flag_whatever_that() {
        assert_no_lints(
            "My actual goal is to have a handful of names, locations or whatever that I have to type",
            EverPronounRelPronoun::default(),
        );
    }

    #[test]
    #[ignore = "does the writer mean 'no matter which'?"]
    fn whatever_which_manager() {
        assert_no_lints(
            "We don't need something more complex but the behavior should be the same wherever which manager extract the dependency.",
            EverPronounRelPronoun::default(),
        );
    }

    #[test]
    #[ignore = "probably needs to change 'whatever who' to 'whoever'"]
    fn whatever_who_you_are() {
        assert_no_lints(
            "Hi friends, Whatever who you are and what your title is if you're reading this it means the internal infrastructure of your company is fully or partially broken.",
            EverPronounRelPronoun::default(),
        );
    }
}
