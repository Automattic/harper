use std::sync::Arc;

use crate::TokenKind;
use crate::expr::AnchorStart;
use crate::expr::Expr;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::{Token, patterns::WordSet};

use crate::Lint;
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, LintKind, Suggestion};

/// See also:
/// harper-core/src/linting/compound_nouns/implied_ownership_compound_nouns.rs
/// harper-core/src/linting/lets_confusion/mod.rs
/// harper-core/src/linting/lets_confusion/let_us_redundancy.rs
/// harper-core/src/linting/lets_confusion/no_contraction_with_verb.rs
pub struct ShouldContract {
    expr: Box<dyn Expr>,
}

impl Default for ShouldContract {
    fn default() -> Self {
        let cap = Arc::new(
            SequenceExpr::word_set(&["your", "were"])
                .then_whitespace()
                .then_kind_is_but_is_not(
                    TokenKind::is_non_quantifier_determiner,
                    TokenKind::is_pronoun,
                )
                .then_whitespace()
                .then_adjective(),
        );

        let start = SequenceExpr::with(AnchorStart).then(cap.clone());
        let mid = SequenceExpr::unless(WordSet::new(&["what"]))
            .t_ws()
            .then(cap);

        Self {
            expr: Box::new(start.or(mid)),
        }
    }
}

impl ShouldContract {
    fn mistake_to_correct(mistake: &str) -> Option<Vec<Vec<char>>> {
        let words = match mistake.to_lowercase().as_str() {
            "your" => vec!["you're", "you are"],
            "were" => vec!["we're", "we are"],
            _ => return None,
        }
        .into_iter()
        .map(|v| v.chars().collect())
        .collect();

        Some(words)
    }
}

impl ExprLinter for ShouldContract {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        // Locate the mistake
        let possible_mistakes = [matched_tokens[0].span, matched_tokens[1].span];

        let mut correct = None;
        let mut span = None;

        for p_mist in possible_mistakes {
            let mistake = p_mist.get_content_string(source);
            let correct_cand = Self::mistake_to_correct(&mistake);
            if correct_cand.is_some() {
                correct = correct_cand;
                span = Some(p_mist);
            }
        }

        let correct = correct?;
        let span = span?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: correct
                .into_iter()
                .map(|v| Suggestion::replace_with_match_case(v, span.get_content(source)))
                .collect(),
            message: "Use the contraction or separate the words instead.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Neglecting the apostrophe when contracting pronouns with \"are\" (like \"your\" and \"you are\") is a fatal, but extremely common mistake to make."
    }
}

#[cfg(test)]
mod tests {
    use super::ShouldContract;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn contracts_your_correctly() {
        assert_suggestion_result(
            "your the best",
            ShouldContract::default(),
            "you're the best",
        );
    }

    #[test]
    fn contracts_were_complex_correctly() {
        assert_suggestion_result(
            "were a good team",
            ShouldContract::default(),
            "we're a good team",
        );
    }

    #[test]
    fn case_insensitive_handling() {
        assert_suggestion_result(
            "Your the best",
            ShouldContract::default(),
            "You're the best",
        );
    }

    #[test]
    fn no_match_without_the() {
        assert_lint_count("your best", ShouldContract::default(), 0);
        assert_lint_count("were best", ShouldContract::default(), 0);
    }

    #[test]
    fn no_match_with_punctuation() {
        assert_lint_count("your, the best", ShouldContract::default(), 0);
    }

    #[test]
    fn allow_norm() {
        assert_lint_count(
            "Let's start this story by going back to the dark ages before internet applications were the norm.",
            ShouldContract::default(),
            0,
        );
    }

    #[test]
    fn allow_issue_1508() {
        assert_no_lints("Were any other toys fun?", ShouldContract::default());
        assert_no_lints("You were his closest friend.", ShouldContract::default());
    }

    #[test]
    fn allows_issue_1673() {
        assert_no_lints("What were the action items?", ShouldContract::default());
    }
}
