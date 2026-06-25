use crate::{CharStringExt, Token, TokenKind};

use super::{MatchResult, RegexNode};

/// Type alias for the predicate function used by Atom.
type AtomPredicate = Box<dyn Fn(&Token, &[char]) -> bool + Send + Sync>;

/// An atomic matching unit - the basic building block of Rig patterns.
///
/// Atom uses a predicate function to check if a single token matches.
/// This is analogous to character classes or literals in regex.
pub struct Atom {
    /// Predicate function that returns true if the token matches
    check: AtomPredicate,
}

impl Atom {
    /// Create a new Atom from a predicate function.
    pub fn new<F>(check: F) -> Self
    where
        F: Fn(&Token, &[char]) -> bool + Send + Sync + 'static,
    {
        Self {
            check: Box::new(check),
        }
    }

    /// Create an Atom that matches any token.
    pub fn any() -> Self {
        Self::new(|_token, _source| true)
    }

    /// Create an Atom that matches a specific word (case-insensitive).
    pub fn word(word: &'static str) -> Self {
        Self::new(move |token, source| token.kind.is_word() && token.get_ch(source).eq_str(word))
    }

    /// Create an Atom that matches whitespace tokens.
    pub fn whitespace() -> Self {
        Self::new(|token, _source| token.kind.is_whitespace())
    }

    /// Create an Atom that matches tokens of a specific kind.
    pub fn kind(kind: impl Fn(&TokenKind) -> bool + Send + Sync + 'static) -> Self {
        Self::new(move |token, _source| kind(&token.kind))
    }

    /// Create an Atom that matches a specific word (case-sensitive).
    pub fn exact_word(word: &'static str) -> Self {
        Self::new(move |token, source| token.kind.is_word() && token.get_str(source) == word)
    }

    /// Create an Atom that matches a specific TokenKind.
    pub fn kind_is(kind: TokenKind) -> Self {
        Self::new(move |token, _source| token.kind == kind)
    }

    /// Create an Atom that matches where a TokenKind predicate returns true.
    pub fn kind_where<F>(predicate: F) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        Self::new(move |token, _source| predicate(&token.kind))
    }
}

impl RegexNode for Atom {
    fn exec(&self, tokens: &[Token], source: &[char], start_idx: usize) -> Option<MatchResult> {
        let token = tokens.get(start_idx)?;

        if (self.check)(token, source) {
            Some(MatchResult {
                captures: hashbrown::HashMap::new(),
                tokens_consumed: 1,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    #[test]
    fn test_word_match() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let atom = Atom::word("hello");
        let result = atom.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_word_no_match() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let atom = Atom::word("goodbye");
        let result = atom.exec(tokens, source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_kind_match() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let atom = Atom::kind_where(TokenKind::is_word);
        let result = atom.exec(tokens, source, 0);

        assert!(result.is_some());
    }

    #[test]
    fn test_any_match() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let atom = Atom::any();
        let result = atom.exec(tokens, source, 0);

        assert!(result.is_some());
    }
}
