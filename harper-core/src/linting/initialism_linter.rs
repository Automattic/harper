use itertools::Itertools;

use crate::{
    Token,
    patterns::{Pattern, Word},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

/// A struct that can be composed to expand initialisms, respecting the capitalization of each
/// item.
pub struct InitialismLinter {
    pattern: Box<dyn Pattern>,
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
            pattern: Box::new(Word::from_char_string(initialism.chars().collect())),
            expansion_lower,
        }
    }
}

impl PatternLinter for InitialismLinter {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let tok = matched_tokens.first()?;
        let source = tok.span.get_content(source);

        let caps = source
            .iter()
            .map(char::is_ascii_uppercase)
            .chain([false].into_iter().cycle());

        let mut expansion_lower = self.expansion_lower.to_owned();

        for (word, cap) in expansion_lower.iter_mut().zip(caps) {
            word[0] = if cap {
                word[0].to_ascii_uppercase()
            } else {
                word[0].to_ascii_lowercase()
            }
        }

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
mod tests {
    use super::InitialismLinter;
}
