use crate::{
    patterns::{Pattern, WordPatternGroup},
    Token, TokenStringExt,
};

use super::{Lint, LintKind, PatternLinter};

pub struct BoringWords {
    pattern: Box<dyn Pattern>,
}

impl Default for BoringWords {
    fn default() -> Self {
        let mut pattern = WordPatternGroup::default();

        pattern.add_word("very");
        pattern.add_word("interesting");
        pattern.add_word("several");
        pattern.add_word("most");
        pattern.add_word("many");

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for BoringWords {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let matched_word = matched_tokens.span()?.get_content_string(source);

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Enhancement,
            suggestions: vec![],
            message: format!(
                "“{}” is a boring word. Try something a little more exotic.",
                matched_word
            ),
            priority: 127,
        })
    }

    fn description(&self) -> &'static str {
        "This rule looks for particularly boring or overused words. Using varied language is an easy way to keep a reader's attention."
    }
}
