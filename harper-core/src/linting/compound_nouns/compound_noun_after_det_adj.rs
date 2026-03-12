use crate::expr::All;
use crate::expr::Expr;
use crate::expr::MergeableWords;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::InflectionOfBe;
use crate::{CharStringExt, TokenStringExt, linting::ExprLinter};

use super::{Lint, LintKind, Suggestion, is_content_word, predicate};
use crate::linting::expr_linter::Chunk;

use crate::{Lrc, Token};

/// Two adjacent words separated by whitespace that if joined would be a valid noun.
pub struct CompoundNounAfterDetAdj {
    expr: Box<dyn Expr>,
    split_expr: Lrc<MergeableWords>,
}

// This heuristic identifies potential compound nouns by:
// 1. Looking for a determiner or adjective (e.g., "a", "big", "red")
// 2. Followed by two content words (not determiners, adverbs, or prepositions)
// 3. Finally, checking if the combination forms a noun in the dictionary
//    that is not also an adjective
impl Default for CompoundNounAfterDetAdj {
    fn default() -> Self {
        let context_expr = SequenceExpr::with(|tok: &Token, src: &[char]| {
            tok.kind.is_determiner()
                || (tok.kind.is_adjective() && *tok.span.get_content(src).to_lower() != ['g', 'o'])
        })
        .t_ws()
        .then(is_content_word)
        .t_ws()
        .then(is_content_word.and_not(InflectionOfBe::default()));

        let split_expr = Lrc::new(MergeableWords::new(|meta_closed, meta_open| {
            predicate(meta_closed, meta_open)
        }));

        let mut expr = All::default();
        expr.add(context_expr);
        expr.add(SequenceExpr::anything().t_any().then(split_expr.clone()));

        Self {
            expr: Box::new(expr),
            split_expr,
        }
    }
}

impl ExprLinter for CompoundNounAfterDetAdj {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens
            .first()?
            .span
            .get_content(source)
            .eq_ignore_ascii_case_str("that")
        {
            return None;
        }

        let span = matched_tokens[2..].span()?;
        let orig = span.get_content(source);
        // If the pattern matched, this will not return `None`.
        let word =
            self.split_expr
                .get_merged_word(&matched_tokens[2], &matched_tokens[4], source)?;

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(word.to_vec(), orig)],
            message: format!(
                "Did you mean the closed compound noun “{}”?",
                word.to_string()
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Detects compound nouns split by a space and suggests merging them when both parts form a valid noun. Has checks to avoid erroneous cases."
    }
}
