use crate::Token;

use super::Step;

/// A [`Step`] which will match only if the cursor is over the last non-whitespace character in stream.
/// It will return that token.
///
/// For example, if you built `SequenceExpr::default().t_aco("word").then(AnchorEnd)` and ran it on `This is a word`, the resulting `Span` would only cover the final word token.
pub struct AnchorEnd;

impl Step for AnchorEnd {
    fn step(&self, tokens: &[Token], cursor: usize, _source: &[char]) -> Option<isize> {
        if tokens
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, t)| !t.kind.is_whitespace())
            .map(|(i, _)| i)
            .next()
            == Some(cursor)
        {
            Some(0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::ExprExt;
    use crate::{Document, Span};

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
}
