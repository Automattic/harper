use crate::{
    Lint, Token,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::{ModalVerb, WordSet},
};

pub struct EverEvery {
    expr: Box<dyn Expr>,
}

impl Default for EverEvery {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_of(vec![
                    Box::new(WordSet::new(&[
                        "are", "aren't", "arent", "did", "didn't", "didnt", "do", "does",
                        "doesn't", "doesnt", "dont", "don't", "had", "hadn't", "hadnt", "has",
                        "hasn't", "hasnt", "have", "haven't", "havent", "is", "isn't", "isnt",
                        "was", "wasn't", "wasnt", "were", "weren't", "werent",
                    ])),
                    Box::new(ModalVerb::with_common_errors()),
                ])
                .t_ws()
                .then_subject_pronoun()
                .t_ws()
                .t_aco("every")
                .and_not(SequenceExpr::anything().t_any().t_aco("it")),
            ),
        }
    }
}

impl ExprLinter for EverEvery {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks[4].span;
        let content = span.get_content(src);
        Some(Lint {
            span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                content[..content.len() - 1].to_vec(),
                content,
            )],
            message: "Is this `every` a typo that should be `ever`?".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Tries to correct typos of `every` instead of `ever`."
    }
}

#[cfg(test)]
mod tests {
    use super::EverEvery;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_can_i_every() {
        assert_suggestion_result(
            "Odd, how can i every become negative in that case?",
            EverEvery::default(),
            "Odd, how can i ever become negative in that case?",
        );
    }

    #[test]
    fn fix_can_they_every() {
        assert_suggestion_result(
            "if each component has its own instance of NameService, how can they every share state?",
            EverEvery::default(),
            "if each component has its own instance of NameService, how can they ever share state?",
        )
    }

    #[test]
    fn fix_can_we_every() {
        assert_suggestion_result(
            "can we every have a good dev UX?",
            EverEvery::default(),
            "can we ever have a good dev UX?",
        );
    }

    #[test]
    fn fix_did_we_every() {
        assert_suggestion_result(
            "Did we every fix that?",
            EverEvery::default(),
            "Did we ever fix that?",
        )
    }

    #[test]
    fn fix_did_you_every() {
        assert_suggestion_result(
            "Did you every get vtsls working properly?",
            EverEvery::default(),
            "Did you ever get vtsls working properly?",
        )
    }

    #[test]
    fn fix_do_i_every() {
        assert_suggestion_result(
            "Rarely do I every look forward to the new ui.",
            EverEvery::default(),
            "Rarely do I ever look forward to the new ui.",
        )
    }

    #[test]
    fn fix_do_we_every() {
        assert_suggestion_result(
            "do we every stop learning new things?",
            EverEvery::default(),
            "do we ever stop learning new things?",
        )
    }

    #[test]
    fn fix_do_you_every() {
        assert_suggestion_result(
            "Do you every faced the issue or have any idea why this could happen?",
            EverEvery::default(),
            "Do you ever faced the issue or have any idea why this could happen?",
        )
    }

    #[test]
    fn fix_dont_i_every() {
        assert_suggestion_result(
            "WHY DONT I EVERY SEE OR HEAR ABOUT THINGS HAPPENING IN SOUTHPORT?",
            EverEvery::default(),
            "WHY DONT I EVER SEE OR HEAR ABOUT THINGS HAPPENING IN SOUTHPORT?",
        )
    }

    #[test]
    fn fix_dont_they_every() {
        assert_suggestion_result(
            "And why dont they every smile first?",
            EverEvery::default(),
            "And why dont they ever smile first?",
        )
    }

    #[test]
    fn fix_dont_you_every() {
        assert_suggestion_result(
            "Dont you every forget this and believe nothing else.",
            EverEvery::default(),
            "Dont you ever forget this and believe nothing else.",
        )
    }

    #[test]
    fn fix_have_you_every() {
        assert_suggestion_result(
            "Have you every wanted to generate geometric structures from data.frames",
            EverEvery::default(),
            "Have you ever wanted to generate geometric structures from data.frames",
        )
    }

    #[test]
    fn fix_should_i_every() {
        assert_suggestion_result(
            "I.e. why would I every use deepcopy ?",
            EverEvery::default(),
            "I.e. why would I ever use deepcopy ?",
        )
    }

    #[test]
    fn fix_should_we_every() {
        assert_suggestion_result(
            "Should we every meet, I'll get you a beverage of your choosing!",
            EverEvery::default(),
            "Should we ever meet, I'll get you a beverage of your choosing!",
        )
    }

    #[test]
    fn fix_should_you_every() {
        assert_suggestion_result(
            "but you will always have a place in his home should you every truly desire it",
            EverEvery::default(),
            "but you will always have a place in his home should you ever truly desire it",
        )
    }

    #[test]
    fn fix_would_i_every() {
        assert_suggestion_result(
            "Why would I every do that?",
            EverEvery::default(),
            "Why would I ever do that?",
        )
    }

    #[test]
    fn fix_would_they_every() {
        assert_suggestion_result(
            "Would they every be installed together?",
            EverEvery::default(),
            "Would they ever be installed together?",
        )
    }

    // known false positive - future contributors: please feel free to tackle this!

    #[test]
    #[ignore = "unusual but not wrong position of time phrase, maybe should have commas?"]
    fn dont_flag_should_we_every() {
        assert_no_lints(
            "MM: should we every month or two have a roundup of what's been happening in WGSL",
            EverEvery::default(),
        )
    }
}
