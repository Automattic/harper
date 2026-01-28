use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::{Span, render_markdown::render_markdown};

use super::{LintKind, Suggestion};

/// An error found in text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Lint {
    /// The location in the source text the error lies.
    /// Important for automatic lint resolution through [`Self::suggestions`].
    pub span: Span<char>,
    /// The general category the lint belongs to.
    /// Mostly used for UI elements in integrations.
    pub lint_kind: LintKind,
    /// A list of zero or more suggested edits that would resolve the underlying problem.
    /// See [`Suggestion`].
    pub suggestions: Vec<Suggestion>,
    /// A message to be displayed to the user describing the specific error found.
    ///
    /// You may use the [`format`] macro to generate more complex messages.
    pub message: String,
    /// A numerical value for the importance of a lint.
    /// Lower = more important.
    pub priority: u8,
}

impl Lint {
    /// Creates a SHA-3 hash of all elements of the lint, sans [`Self::span`].
    /// This is useful for comparing lints while ignoring their position within the document.
    ///
    /// Do not assume that these hash values are stable across Harper versions.
    pub fn spanless_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.lint_kind.hash(&mut hasher);
        self.suggestions.hash(&mut hasher);
        self.message.hash(&mut hasher);
        self.priority.hash(&mut hasher);

        hasher.finish()
    }

    /// Interpret the message as Markdown and render it to HTML.
    pub fn message_html(&self) -> String {
        render_markdown(&self.message)
    }
}

impl Default for Lint {
    fn default() -> Self {
        Self {
            span: Default::default(),
            lint_kind: Default::default(),
            suggestions: Default::default(),
            message: Default::default(),
            priority: 127,
        }
    }
}
