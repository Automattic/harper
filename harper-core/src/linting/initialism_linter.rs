use crate::expr::Expr;
use itertools::Itertools;

use crate::{Token, patterns::Word};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

/// A struct that can be composed to expand initialisms, respecting the capitalization of each
/// item.
pub struct InitialismLinter {
    expr: Box<dyn Expr>,
    /// The lowercase-normalized expansion of the initialism.
    expansion_lower: Vec<Vec<char>>,
}

impl InitialismLinter {
    /// Construct a linter that can correct an initialism to
    pub fn new(initialism: &str, expansion: &str) -> Self {
        let expansion_lower = expansion
            .split(' ')
            .map(|s| s.chars().map(|v| v.to_ascii_lowercase()).collect())
            .collect();

        Self {
            expr: Box::new(Word::from_char_string(initialism.chars().collect())),
            expansion_lower,
        }
    }
}

impl ExprLinter for InitialismLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let tok = matched_tokens.first()?;
        let source = tok.span.get_content(source);

        let mut expansion_lower = self.expansion_lower.to_owned();
        let first_letter = &mut expansion_lower[0][0];

        *first_letter = if source[0].is_ascii_uppercase() {
            first_letter.to_ascii_uppercase()
        } else {
            first_letter.to_ascii_lowercase()
        };

        let phrase = Itertools::intersperse_with(expansion_lower.into_iter(), || vec![' '])
            .reduce(|mut left, mut right| {
                left.append(&mut right);
                left
            })
            .unwrap();

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::ReplaceWith(phrase)],
            message: "Try expanding this initialism.".to_owned(),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "Expands an initialism."
    }
}

#[cfg(test)]
mod tests {}
