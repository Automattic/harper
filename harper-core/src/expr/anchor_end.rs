use crate::Token;

use super::Step;

/// A [`Step`] which will match only if the cursor is over the last non-whitespace character in stream.
/// It will return that token.
///
/// For example, if you built `SequenceExpr::default().t_aco("word").then(AnchorEnd)` and ran it on `This is a word`, the resulting `Span` would only cover the final word token.
pub struct AnchorEnd;

impl Step for AnchorEnd {
    fn step(&self, tokens: &[Token], cursor: usize, _source: &[char]) -> Option<isize> {
        let last_non_ws = tokens
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, t)| !t.kind.is_whitespace())
            .map(|(i, _)| i)
            .next();

        // Match if cursor is at or past the last non-whitespace token
        // This allows AnchorEnd to work in sequences where the cursor has advanced
        // past the matched content, including when cursor is past the end of the token stream
        if let Some(last) = last_non_ws {
            if cursor >= last {
                return Some(0);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::{AnchorStart, ExprExt, SequenceExpr};
    use crate::{Document, Span, TokenStringExt};

    use super::AnchorEnd;

    #[test]
    fn matches_period() {
        let document = Document::new_markdown_default_curated("This is a test.");
        let matches: Vec<_> = AnchorEnd.iter_matches_in_doc(&document).collect();

        assert_eq!(matches, vec![Span::new(7, 7)])
    }

    #[test]
    fn does_not_match_empty() {
        let document = Document::new_markdown_default_curated("");
        let matches: Vec<_> = AnchorEnd.iter_matches_in_doc(&document).collect();

        assert_eq!(matches, vec![])
    }

    #[test]
    fn test_word_at_end_of_document() {
        // Test matching a specific word at the end of a document
        let document = Document::new_plain_english_curated("This is the end");
        let expr = SequenceExpr::default()
            .then_any_capitalization_of("end")
            .then(AnchorEnd);

        let matches: Vec<_> = expr.iter_matches_in_doc(&document).collect();
        // Should match "end" at position 6 (accounting for whitespace tokens)
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 6);
    }

    #[test]
    fn test_word_not_at_end_of_doc() {
        // Test that it doesn't match when word has trailing content
        let document = Document::new_plain_english_curated("This is the end, really");
        let expr = SequenceExpr::default()
            .then_any_capitalization_of("end")
            .then(AnchorEnd);

        let matches: Vec<_> = expr.iter_matches_in_doc(&document).collect();
        // Should NOT match because "end" is not at chunk end
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_word_at_end_of_chunk() {
        // Chunks are split by commas, so "hello, world" becomes two chunks
        // Test that AnchorEnd works at the end of a chunk
        let document = Document::new_plain_english_curated("hello, world");
        let expr = SequenceExpr::default()
            .then_any_capitalization_of("hello")
            .then(AnchorEnd);

        // Test on the first chunk which contains just "hello"
        let first_chunk = document.iter_chunks().next().unwrap();
        let matches: Vec<_> = expr
            .iter_matches(first_chunk, document.get_source())
            .collect();
        // Should match because "hello" is at the end of its chunk
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_compare_with_anchor_start() {
        // AnchorStart works as expected
        let document = Document::new_plain_english_curated("Start here");
        let expr = SequenceExpr::default()
            .then(AnchorStart)
            .then_any_capitalization_of("start");

        let matches: Vec<_> = expr.iter_matches_in_doc(&document).collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 0);
    }

    #[test]
    fn test_word_with_trailing_whitespace_at_end_of_doc() {
        // Test AnchorEnd after matching whitespace
        let document = Document::new_plain_english_curated("foo ");
        let expr = SequenceExpr::default()
            .then_any_capitalization_of("foo")
            .then_whitespace()
            .then(AnchorEnd);

        let matches: Vec<_> = expr.iter_matches_in_doc(&document).collect();
        // Should match "foo" at the end (cursor is past the token stream)
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].end, 2);
    }
}
