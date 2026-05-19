//! Rig: A stateful token-matching engine with captures and proper boundary handling.
//!
//! Rig is designed as a parallel experiment to Expr, focusing on:
//! - Numbered capture groups (like regex)
//! - Proper chunk/sentence boundary anchors
//! - Non-greedy quantifiers
//! - Zero-width assertions that work correctly in sequences
//!
//! The design uses regex-like terminology: Atom, CaptureGroup, Concat, Alternation, etc.

use crate::{LSend, Span, Token};
use hashbrown::HashMap;

pub mod alternation;
pub mod anchors;
pub mod atom;
pub mod capture_group;
pub mod concat;
pub mod quantifier;
pub mod rig_linter;
pub mod spacing;

pub use alternation::Alternation;
pub use anchors::{AnchorEnd, AnchorStart};
pub use atom::Atom;
pub use capture_group::CaptureGroup;
pub use concat::Concat;
pub use quantifier::{Quantifier, QuantifierMode};
pub use rig_linter::RigLinter;
pub use spacing::Spacing;

/// Result of executing a Rig pattern, containing capture information.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Maps capture IDs to their token spans
    pub captures: HashMap<usize, Span<Token>>,
    /// Total number of tokens consumed by the match
    pub tokens_consumed: usize,
}

/// A rich match object similar to regex::Match, providing access to captures.
#[derive(Debug, Clone)]
pub struct RigMatch<'a> {
    /// The overall span of the match
    pub span: Span<Token>,
    /// Capture groups indexed by ID
    pub captures: HashMap<usize, Span<Token>>,
    /// Reference to the matched tokens
    pub tokens: &'a [Token],
    /// Reference to the source characters
    pub source: &'a [char],
}

impl<'a> RigMatch<'a> {
    /// Get the tokens for a specific capture group by ID
    pub fn get_capture_tokens(&self, capture_id: usize) -> Option<&'a [Token]> {
        self.captures
            .get(&capture_id)
            .map(|span| &self.tokens[span.start..span.end])
    }

    /// Get the text content of a specific capture group
    pub fn get_capture_text(&self, capture_id: usize) -> Option<String> {
        self.get_capture_tokens(capture_id).map(|tokens| {
            tokens
                .iter()
                .map(|t| t.get_str(self.source))
                .collect::<String>()
        })
    }
}

/// The core trait for Rig pattern nodes.
///
/// Each node knows how to execute itself against a token stream and return
/// capture information.
pub trait RegexNode: LSend {
    /// Execute the pattern starting at the given index.
    ///
    /// Returns `Some(MatchResult)` if the pattern matches, containing:
    /// - The captures populated during matching
    /// - The number of tokens consumed
    ///
    /// Returns `None` if the pattern does not match.
    fn exec(&self, tokens: &[Token], source: &[char], start_idx: usize) -> Option<MatchResult>;
}
