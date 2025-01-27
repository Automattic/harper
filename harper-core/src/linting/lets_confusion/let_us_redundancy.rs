use crate::{
    patterns::{Pattern, SequencePattern},
    Token, TokenStringExt,
};

use crate::linting::{Lint, LintKind, PatternLinter, Suggestion};

pub struct LetUsRedundancy {
    pattern: Box<dyn Pattern>,
}

impl Default for LetUsRedundancy {
    fn default() -> Self {
        let pattern = SequencePattern::aco("let's")
            .then_whitespace()
            .then_pronoun();

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for LetUsRedundancy {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Lint {
        let template = matched_tokens.span().unwrap().get_content(source);
        let pronoun = matched_tokens
            .last()
            .unwrap()
            .span
            .get_content_string(source);

        Lint {
            span: matched_tokens.span().unwrap(),
            lint_kind: LintKind::Repetition,
            suggestions: vec![
                Suggestion::replace_with_match_case(
                    format!("lets {pronoun}").chars().collect(),
                    template,
                ),
                Suggestion::replace_with_match_case(
                    "let's".to_string().chars().collect(),
                    template,
                ),
            ],
            message: "`let's` stands for `let us`, so including another pronoun is redundant."
                .to_owned(),
            priority: 31,
        }
    }

    fn description(&self) -> &'static str {
        "Many are not aware that the contraction `let's` is short for `let us`. As a result, many will incorrectly use it before a pronoun, such as in the phrase `let's us do`."
    }
}
