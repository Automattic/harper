use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

use crate::{
    Document, FatToken,
    linting::{Lint, LintKind, Suggestion},
};

/// A location-agnostic structure that attempts to captures the context and content that a [`Lint`]
/// occurred.
#[derive(Debug, Serialize, Deserialize)]
pub struct LintContext {
    pub lint_kind: LintKind,
    pub suggestions: Vec<Suggestion>,
    pub message: String,
    pub priority: u8,
    pub tokens: Vec<FatToken>,
}

impl Hash for LintContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.lint_kind.hash(state);
        self.suggestions.hash(state);
        self.message.hash(state);
        self.priority.hash(state);
        self.tokens.hash(state);
    }
}

impl LintContext {
    pub fn from_lint(lint: &Lint, document: &Document) -> Self {
        let Lint {
            lint_kind,
            suggestions,
            message,
            priority,
            ..
        } = lint.clone();

        let problem_tokens = document.token_indices_intersecting(lint.span);
        let prequel_tokens = lint
            .span
            .with_len(2)
            .pulled_by(2)
            .map(|v| document.token_indices_intersecting(v))
            .unwrap_or_default();
        let sequel_tokens = document.token_indices_intersecting(lint.span.with_len(2).pushed_by(2));

        let tokens = prequel_tokens
            .into_iter()
            .chain(problem_tokens)
            .chain(sequel_tokens)
            .flat_map(|idx| document.get_token(idx))
            .map(|t| t.to_fat(document.get_source()))
            .collect();

        Self {
            lint_kind,
            suggestions,
            message,
            priority,
            tokens,
        }
    }
}
