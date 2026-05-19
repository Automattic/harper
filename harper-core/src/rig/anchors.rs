use crate::Token;

use super::{MatchResult, RegexNode};

/// Matches at the start of the token stream (chunk/sentence).
///
/// This is a zero-width assertion analogous to regex `^`.
pub struct AnchorStart;

impl RegexNode for AnchorStart {
    fn exec(&self, _tokens: &[Token], _source: &[char], start_idx: usize) -> Option<MatchResult> {
        if start_idx == 0 {
            Some(MatchResult {
                captures: hashbrown::HashMap::new(),
                tokens_consumed: 0,
            })
        } else {
            None
        }
    }
}

/// Matches at the end of the token stream (chunk/sentence).
///
/// This is a zero-width assertion analogous to regex `$`.
/// Unlike Expr's AnchorEnd, this works correctly in sequences because
/// it checks the position after matching, not the cursor position.
pub struct AnchorEnd;

impl RegexNode for AnchorEnd {
    fn exec(&self, tokens: &[Token], _source: &[char], start_idx: usize) -> Option<MatchResult> {
        // Check if we're at the last non-whitespace token
        let last_non_ws = tokens
            .iter()
            .enumerate()
            .rev()
            .find(|(_, t)| !t.kind.is_whitespace())
            .map(|(i, _)| i);

        if last_non_ws == Some(start_idx) {
            Some(MatchResult {
                captures: hashbrown::HashMap::new(),
                tokens_consumed: 0,
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
    use crate::rig::{Atom, Concat};

    #[test]
    fn test_anchor_start() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let anchor = AnchorStart;
        let result = anchor.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 0);
    }

    #[test]
    fn test_anchor_start_not_at_start() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let anchor = AnchorStart;
        let result = anchor.exec(tokens, source, 1);

        assert!(result.is_none());
    }

    #[test]
    fn test_anchor_end() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let anchor = AnchorEnd;
        let result = anchor.exec(tokens, source, tokens.len() - 1);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 0);
    }

    #[test]
    fn test_anchor_end_not_at_end() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let anchor = AnchorEnd;
        let result = anchor.exec(tokens, source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_anchor_in_sequence() {
        // This is the key test: AnchorEnd should work in sequences
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let pattern = Concat::new(vec![
            Box::new(AnchorStart),
            Box::new(Atom::any()),
            Box::new(Atom::any()),
            Box::new(AnchorEnd),
        ]);

        let result = pattern.exec(tokens, source, 0);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.tokens_consumed, 2); // Two tokens
    }

    #[test]
    fn test_anchor_end_sequence_fail() {
        // Should not match if not at end
        let doc = Document::new_plain_english_curated("hello world, more");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let pattern = Concat::new(vec![Box::new(Atom::word("hello")), Box::new(AnchorEnd)]);

        let result = pattern.exec(tokens, source, 0);

        assert!(result.is_none());
    }
}
