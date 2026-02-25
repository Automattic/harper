use crate::expr::Expr;
use crate::expr::FixedPhrase;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::{Lrc, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct OneAndTheSame {
    expr: Box<dyn Expr>,
}

impl Default for OneAndTheSame {
    fn default() -> Self {
        let one_in_the_same = Lrc::new(FixedPhrase::from_phrase("one in the same"));

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(
                    SequenceExpr::word_set(&["are", "were"])
                        .t_ws()
                        .then(one_in_the_same.clone()),
                ),
                Box::new(
                    SequenceExpr::with(one_in_the_same.clone())
                        .t_ws()
                        .t_aco("as"),
                ),
            ])),
        }
    }
}

fn ws_word(word: &'static str) -> SequenceExpr {
    SequenceExpr::default().t_ws().t_aco(word)
}

impl ExprLinter for OneAndTheSame {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let phrase = if matched_tokens.last()?.span.get_content(source) == ['a', 's'] {
            matched_tokens[0..matched_tokens.len() - 2].span()?
        } else {
            matched_tokens[2..].span()?
        };

        Some(Lint {
            span: phrase,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "one and the same".chars().collect(),
                phrase.get_content(source),
            )],
            message: "The actual idiom is with the word `and`.".to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "This linter flags instances of the nonstandard phrase `one in the same`. The correct, more accepted form is `one and the same`"
    }
}

#[cfg(test)]
mod tests {
    use super::OneAndTheSame;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_after_are_atomic() {
        assert_suggestion_result(
            "... are one in the same ...",
            OneAndTheSame::default(),
            "... are one and the same ...",
        );
    }

    #[test]
    fn corrects_after_were_atomic() {
        assert_suggestion_result(
            "... were one in the same ...",
            OneAndTheSame::default(),
            "... were one and the same ...",
        );
    }

    #[test]
    fn doesnt_flag_after_other_words_atomic() {
        assert_lint_count(
            "... and another one in the same place ...",
            OneAndTheSame::default(),
            0,
        );
    }

    #[test]
    fn corrects_github_are() {
        assert_suggestion_result(
            "Yes, I believe they are one in the same.",
            OneAndTheSame::default(),
            "Yes, I believe they are one and the same.",
        );
    }

    #[test]
    fn corrects_github_were() {
        assert_suggestion_result(
            "As prior to OpenShift 4.0, OAuth and Kubernetes REST API were one in the same, option (2) above should still work there.",
            OneAndTheSame::default(),
            "As prior to OpenShift 4.0, OAuth and Kubernetes REST API were one and the same, option (2) above should still work there.",
        );
    }

    #[test]
    fn corrects_before_as_atomic() {
        assert_suggestion_result(
            "... one in the same as ...",
            OneAndTheSame::default(),
            "... one and the same as ...",
        );
    }

    #[test]
    fn corrects_before_as_github() {
        assert_suggestion_result(
            "In our case the slicedState is one in the same as the featureState",
            OneAndTheSame::default(),
            "In our case the slicedState is one and the same as the featureState",
        );
    }

    #[test]
    #[ignore = "needs zero-width end-of-chunk pattern akin to regex `$`"]
    fn corrects_at_end() {
        assert_suggestion_result(
            "I think this is one in the same.",
            OneAndTheSame::default(),
            "I think this is one and the same.",
        );
    }

    #[test]
    fn corrects_is_as() {
        assert_suggestion_result(
            "I believe this and this issue is one in the same as Next.js uses cloudflare workers for it's edge infra.",
            OneAndTheSame::default(),
            "I believe this and this issue is one and the same as Next.js uses cloudflare workers for it's edge infra.",
        );
    }

    #[test]
    fn avoids_false_positive() {
        assert_lint_count(
            "If there is no postgresql.pg_hba either there is one in the same section of patroni.yaml or pg_hba.conf is not managed by Patroni.",
            OneAndTheSame::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Cannot detect unexpected ungrammatical `same of`"]
    fn corrects_is_of() {
        assert_suggestion_result(
            "R3 that Stephan Buhre noted is one-in-the-same of what I posted.",
            OneAndTheSame::default(),
            "R3 that Stephan Buhre noted is one and the same of what I posted.",
        );
    }

    #[test]
    fn doesnt_flag_ambiguous_before_noun() {
        assert_lint_count(
            "I'm guessing this is one in the same request.",
            OneAndTheSame::default(),
            0,
        );
    }
}
