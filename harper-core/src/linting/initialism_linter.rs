use crate::expr::Expr;

use crate::{Token, patterns::Word};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

/// Alias for a word in an initialism expansion
type InitialismWord = Vec<char>;
/// Alias for a phrase in an initialism expansion
type InitialismPhrase = Vec<InitialismWord>;

/// A struct that can be composed to expand initialisms, respecting the capitalization of each
/// item.
pub struct InitialismLinter {
    expr: Box<dyn Expr>,
    /// The lowercase-normalized expansion of the initialism.
    expansions_lower: Vec<InitialismPhrase>,
}

impl InitialismLinter {
    /// Construct a linter that can correct an initialism to
    pub fn new(initialism: &str, expansions: &[&str]) -> Self {
        let mut expansions_lower: Vec<InitialismPhrase> = Vec::new();
        for expansion in expansions {
            let expansion_lower: InitialismPhrase = expansion
                .split(' ')
                .map(|s| s.chars().map(|v| v.to_ascii_lowercase()).collect())
                .collect();

            expansions_lower.push(expansion_lower);
        }

        Self {
            expr: Box::new(Word::from_char_string(initialism.chars().collect())),
            expansions_lower,
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

        let mut phrases = Vec::new();
        for expansion_lower in &self.expansions_lower {
            let mut expansion = expansion_lower.clone();
            let first_letter = &mut expansion[0][0];

            *first_letter = if source[0].is_ascii_uppercase() {
                first_letter.to_ascii_uppercase()
            } else {
                first_letter.to_ascii_lowercase()
            };

            let phrase = expansion
                .iter()
                .flat_map(|word| std::iter::once(' ').chain(word.iter().copied()))
                .skip(1)
                .collect::<Vec<_>>();

            phrases.push(phrase);
        }

        let suggestions = phrases.into_iter().map(Suggestion::ReplaceWith).collect();

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
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
