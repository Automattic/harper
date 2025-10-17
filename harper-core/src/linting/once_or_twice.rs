use crate::CharStringExt;
use crate::Token;
use crate::expr::{Expr, SequenceExpr};

use super::{ExprLinter, Lint, LintKind, Suggestion};

pub struct OnceOrTwice {
    expr: Box<dyn Expr>,
}

impl Default for OnceOrTwice {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("once")
            .then_whitespace()
            .t_aco("a")
            .then_whitespace()
            .t_aco("twice");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for OnceOrTwice {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let article = matched_tokens.iter().find(|token| {
            token.kind.is_word() && token.span.get_content(source).eq_ignore_ascii_case_str("a")
        })?;

        let span = article.span;
        let original = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str("or", original)],
            message: "Did you mean “or”?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Detects the mistaken phrase `once a twice` and suggests `once or twice`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::OnceOrTwice;

    #[test]
    fn corrects_once_a_twice() {
        assert_suggestion_result(
            "He wants to do it once a twice a month.",
            OnceOrTwice::default(),
            "He wants to do it once or twice a month.",
        );
    }

    #[test]
    fn allows_once_or_twice() {
        assert_no_lints(
            "He wants to do it once or twice a month.",
            OnceOrTwice::default(),
        );
    }

    #[test]
    fn corrects_once_a_twice_sentence_start() {
        assert_suggestion_result(
            "Once a twice, we gathered for coffee.",
            OnceOrTwice::default(),
            "Once or twice, we gathered for coffee.",
        );
    }

    #[test]
    fn corrects_once_a_twice_uppercase() {
        assert_suggestion_result(
            "ONCE A TWICE WE MET.",
            OnceOrTwice::default(),
            "ONCE OR TWICE WE MET.",
        );
    }

    #[test]
    fn corrects_once_a_twice_mixed_case() {
        assert_suggestion_result(
            "once a Twice sounds odd.",
            OnceOrTwice::default(),
            "once or Twice sounds odd.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_exclamation() {
        assert_suggestion_result(
            "Let's do it once a twice!",
            OnceOrTwice::default(),
            "Let's do it once or twice!",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_question_mark() {
        assert_suggestion_result(
            "You really tried once a twice?",
            OnceOrTwice::default(),
            "You really tried once or twice?",
        );
    }

    #[test]
    fn corrects_once_a_twice_inside_quotes() {
        assert_suggestion_result(
            "He said, \"once a twice\" without thinking.",
            OnceOrTwice::default(),
            "He said, \"once or twice\" without thinking.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_comma() {
        assert_suggestion_result(
            "We planned it once a twice, but never finished.",
            OnceOrTwice::default(),
            "We planned it once or twice, but never finished.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_parentheses() {
        assert_suggestion_result(
            "Try it (just once a twice) before judging.",
            OnceOrTwice::default(),
            "Try it (just once or twice) before judging.",
        );
    }

    #[test]
    fn corrects_once_a_twice_after_colon() {
        assert_suggestion_result(
            "My answer is simple: once a twice is too many.",
            OnceOrTwice::default(),
            "My answer is simple: once or twice is too many.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_double_space() {
        assert_suggestion_result(
            "We tested once a twice  before launch.",
            OnceOrTwice::default(),
            "We tested once or twice  before launch.",
        );
    }

    #[test]
    fn corrects_once_a_twice_before_semicolon() {
        assert_suggestion_result(
            "They tried once a twice; it still failed.",
            OnceOrTwice::default(),
            "They tried once or twice; it still failed.",
        );
    }

    #[test]
    fn corrects_once_a_twice_newline_split() {
        assert_suggestion_result(
            "We met once a twice\nwhen the cafe was quiet.",
            OnceOrTwice::default(),
            "We met once or twice\nwhen the cafe was quiet.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_tab() {
        assert_suggestion_result(
            "Schedule it once a twice\tfor testing.",
            OnceOrTwice::default(),
            "Schedule it once or twice\tfor testing.",
        );
    }

    #[test]
    fn corrects_once_a_twice_multiple_sentences() {
        assert_suggestion_result(
            "Do it once a twice. Then rest.",
            OnceOrTwice::default(),
            "Do it once or twice. Then rest.",
        );
    }

    #[test]
    fn corrects_once_a_twice_before_period() {
        assert_suggestion_result(
            "He rehearsed once a twice.",
            OnceOrTwice::default(),
            "He rehearsed once or twice.",
        );
    }

    #[test]
    fn corrects_once_a_twice_with_trailing_space() {
        assert_suggestion_result(
            "Practice once a twice .",
            OnceOrTwice::default(),
            "Practice once or twice .",
        );
    }

    #[test]
    fn corrects_once_a_twice_before_dash() {
        assert_suggestion_result(
            "He called once a twice—no response.",
            OnceOrTwice::default(),
            "He called once or twice—no response.",
        );
    }

    #[test]
    fn corrects_once_a_twice_around_em_dash() {
        assert_suggestion_result(
            "She visits once a twice—maybe thrice.",
            OnceOrTwice::default(),
            "She visits once or twice—maybe thrice.",
        );
    }

    #[test]
    fn corrects_once_a_twice_before_quote() {
        assert_suggestion_result(
            "We heard once a twice, \"she's late.\"",
            OnceOrTwice::default(),
            "We heard once or twice, \"she's late.\"",
        );
    }

    #[test]
    fn corrects_once_a_twice_all_caps_sentence() {
        assert_suggestion_result(
            "DO IT ONCE A TWICE RIGHT NOW!",
            OnceOrTwice::default(),
            "DO IT ONCE OR TWICE RIGHT NOW!",
        );
    }

    #[test]
    fn allows_once_a_time_story() {
        assert_no_lints("Once a time, in a distant land...", OnceOrTwice::default());
    }

    #[test]
    fn allows_once_a_week_routine() {
        assert_no_lints("We meet once a week to sync up.", OnceOrTwice::default());
    }

    #[test]
    fn allows_once_a_while_phrase() {
        assert_no_lints(
            "Check in every once a while to stay updated.",
            OnceOrTwice::default(),
        );
    }

    #[test]
    fn allows_once_or_twice_uppercase() {
        assert_no_lints("ONCE OR TWICE, WE MADE IT WORK.", OnceOrTwice::default());
    }

    #[test]
    fn allows_twice_without_once() {
        assert_no_lints(
            "We only managed it twice this year.",
            OnceOrTwice::default(),
        );
    }

    #[test]
    fn allows_once_and_twice_separated() {
        assert_no_lints("Once I tried; twice I failed.", OnceOrTwice::default());
    }

    #[test]
    fn allows_oncemisatypo() {
        assert_no_lints("oncemisatypo appears once a line.", OnceOrTwice::default());
    }

    #[test]
    fn allows_spaced_words() {
        assert_no_lints(
            "We say once at twice distance to be safe.",
            OnceOrTwice::default(),
        );
    }
}
