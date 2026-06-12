use crate::Span;

use super::{MatchResult, RegexNode};

/// A capture group that wraps another node and records its match span.
///
/// This is analogous to numbered capture groups in regex: `(pattern)`
pub struct CaptureGroup {
    /// The capture group ID (0, 1, 2, ...)
    id: usize,
    /// The inner pattern to capture
    inner: Box<dyn RegexNode>,
}

impl CaptureGroup {
    /// Create a new capture group with the given ID.
    pub fn new(id: usize, inner: Box<dyn RegexNode>) -> Self {
        Self { id, inner }
    }
}

impl RegexNode for CaptureGroup {
    fn exec(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        // Execute the inner pattern
        let inner_result = self.inner.exec(tokens, source, start_idx)?;

        // Calculate the span of this capture
        let capture_span = Span::new(start_idx, start_idx + inner_result.tokens_consumed);

        // Add this capture to the result
        let mut captures = inner_result.captures;
        captures.insert(self.id, capture_span);

        Some(MatchResult {
            captures,
            tokens_consumed: inner_result.tokens_consumed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rig::Atom;

    #[test]
    fn test_capture_group() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let atom = Box::new(Atom::word("hello"));
        let capture = CaptureGroup::new(0, atom);

        let result = capture.exec(tokens, source, 0);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.tokens_consumed, 1);
        assert!(result.captures.contains_key(&0));
        assert_eq!(result.captures[&0].start, 0);
        assert_eq!(result.captures[&0].end, 1);
    }
}
