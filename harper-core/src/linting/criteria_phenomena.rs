use super::Suggestion;
use super::expr_linter::ExprLinter;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::LintKind;
use crate::linting::expr_linter::Chunk;
use crate::patterns::WordSet;
use crate::{Lint, Lrc, Token, TokenStringExt};

/// Linter that checks if 'criteria' or 'phenomena' is used as singular.
pub struct CriteriaPhenomena {
    expr: Box<dyn Expr>,
    plural_words: Lrc<WordSet>,
    singular_modifiers: Lrc<WordSet>,
}

impl CriteriaPhenomena {
    fn new() -> Self {
        let plural_words = Lrc::new(WordSet::new(&["criteria", "phenomena"]));

        let singular_modifiers = Lrc::new(WordSet::new(&["this", "that", "a", "one"]));

        Self {
            expr: Box::new(
                SequenceExpr::with(singular_modifiers.clone())
                    .then_whitespace()
                    .then(plural_words.clone()),
            ),
            plural_words,
            singular_modifiers,
        }
    }
}

impl ExprLinter for CriteriaPhenomena {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mut second_word: String = matched_tokens[2]
            .span
            .get_content(source)
            .iter()
            .copied()
            .collect();
        second_word.make_ascii_lowercase();

        let suggestions = match second_word.as_str() {
            "criteria" => vec![Suggestion::ReplaceWith("criterion".chars().collect())],
            "phenomena" => vec![Suggestion::ReplaceWith("phenomenon".chars().collect())],
            _ => panic!("Don't know what to say about '{second_word}'!"),
        };

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Repetition,
            message: "You used a plural noun as singular.".to_owned(),
            priority: 63,
            suggestions,
        })
    }

    fn description(&self) -> &'static str {
        "The words “criteria” and “phenomena” are the plurals of “criterion” and “phenomenon”, respectively. They are often incorrectly used as singular."
    }
}

impl Default for CriteriaPhenomena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::CriteriaPhenomena;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn can_detect_incorrect_criteria() {
        assert_lint_count(
            "...One criteria is essential...",
            CriteriaPhenomena::new(),
            1,
        )
    }

    #[test]
    fn can_detect_incorrect_phenomena() {
        assert_lint_count(
            "...I would like to see that phenomena.",
            CriteriaPhenomena::new(),
            1,
        )
    }

    #[test]
    fn allows_correct_criteria() {
        assert_lint_count(
            "...She disagrees with those criteria.",
            CriteriaPhenomena::new(),
            0,
        )
    }

    #[test]
    fn allows_correct_phenomena() {
        assert_lint_count(
            "...Many phenomena were on display.",
            CriteriaPhenomena::new(),
            0,
        )
    }
}
