use itertools::Itertools;

use crate::{
    Token,
    patterns::{Pattern, Word},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

pub struct ByTheWay {
    pattern: Box<dyn Pattern>,
}

impl Default for ByTheWay {
    fn default() -> Self {
        Self {
            pattern: Box::new(Word::new("btw")),
        }
    }
}

impl PatternLinter for ByTheWay {
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

        let mut phrase: Vec<Vec<char>> = ["by", "the", "way"]
            .iter()
            .map(|v| v.chars().collect())
            .collect();

        for (word, cap) in phrase.iter_mut().zip(caps) {
            word[0] = if cap {
                word[0].to_ascii_uppercase()
            } else {
                word[0].to_ascii_lowercase()
            }
        }

        let phrase = Itertools::intersperse_with(phrase.into_iter(), || vec![' '])
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
        "Expands the initialism, `btw`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::ByTheWay;

    #[test]
    fn corrects_shopping() {
        assert_suggestion_result(
            "Btw, are you ready to go shopping soon?",
            ByTheWay::default(),
            "By the way, are you ready to go shopping soon?",
        );
    }

    #[test]
    fn corrects_style() {
        assert_suggestion_result(
            "I love the fit, btw.",
            ByTheWay::default(),
            "I love the fit, by the way.",
        );
    }
}
