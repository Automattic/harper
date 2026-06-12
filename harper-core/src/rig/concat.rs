use hashbrown::HashMap;

use super::{MatchResult, RegexNode};

/// Concatenates multiple patterns in sequence.
///
/// This is analogous to simply placing patterns next to each other in regex: `abc`
pub struct Concat {
    /// The sequence of patterns to match in order
    nodes: Vec<Box<dyn RegexNode>>,
}

impl Concat {
    /// Create a new Concat from a vector of patterns.
    pub fn new(nodes: Vec<Box<dyn RegexNode>>) -> Self {
        Self { nodes }
    }

    /// Create an empty Concat.
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Append a pattern to the sequence.
    pub fn then(mut self, node: Box<dyn RegexNode>) -> Self {
        self.nodes.push(node);
        self
    }
}

impl RegexNode for Concat {
    fn exec(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        let mut current_idx = start_idx;
        let mut aggregated_captures = HashMap::new();

        for node in &self.nodes {
            let res = node.exec(tokens, source, current_idx)?;

            // Merge captures
            aggregated_captures.extend(res.captures);

            // Advance cursor
            current_idx += res.tokens_consumed;
        }

        Some(MatchResult {
            captures: aggregated_captures,
            tokens_consumed: current_idx - start_idx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rig::Atom;

    #[test]
    fn test_concat_two_atoms() {
        // Use a pattern that matches any token twice
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let concat = Concat::new(vec![Box::new(Atom::any()), Box::new(Atom::any())]);

        let result = concat.exec(tokens, source, 0);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.tokens_consumed, 2);
    }

    #[test]
    fn test_concat_fail_on_second() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let concat = Concat::new(vec![
            Box::new(Atom::word("hello")),
            Box::new(Atom::word("goodbye")),
        ]);

        let result = concat.exec(tokens, source, 0);

        assert!(result.is_none());
    }

    #[test]
    fn test_concat_with_captures() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        use crate::rig::CaptureGroup;

        let concat = Concat::new(vec![
            Box::new(CaptureGroup::new(0, Box::new(Atom::any()))),
            Box::new(CaptureGroup::new(1, Box::new(Atom::any()))),
        ]);

        let result = concat.exec(tokens, source, 0);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.tokens_consumed, 2);
        assert!(result.captures.contains_key(&0));
        assert!(result.captures.contains_key(&1));
    }
}
