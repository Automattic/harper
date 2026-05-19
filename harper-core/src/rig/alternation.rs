use super::{MatchResult, RegexNode};

/// Matches the first pattern that succeeds from a list of alternatives.
///
/// This is analogous to the `|` operator in regex: `a|b|c`
pub struct Alternation {
    /// The alternative patterns to try in order
    nodes: Vec<Box<dyn RegexNode>>,
}

impl Alternation {
    /// Create a new Alternation from a vector of patterns.
    pub fn new(nodes: Vec<Box<dyn RegexNode>>) -> Self {
        Self { nodes }
    }

    /// Create an empty Alternation.
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add an alternative pattern.
    pub fn or(mut self, node: Box<dyn RegexNode>) -> Self {
        self.nodes.push(node);
        self
    }
}

impl RegexNode for Alternation {
    fn exec(
        &self,
        tokens: &[crate::Token],
        source: &[char],
        start_idx: usize,
    ) -> Option<MatchResult> {
        // Try each alternative in order, return the first that matches
        for node in &self.nodes {
            if let Some(result) = node.exec(tokens, source, start_idx) {
                return Some(result);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::rig::Atom;

    #[test]
    fn test_alternation_first_matches() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let alt = Alternation::new(vec![
            Box::new(Atom::word("hello")),
            Box::new(Atom::word("goodbye")),
        ]);

        let result = alt.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_alternation_second_matches() {
        let doc = Document::new_plain_english_curated("goodbye world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let alt = Alternation::new(vec![
            Box::new(Atom::word("hello")),
            Box::new(Atom::word("goodbye")),
        ]);

        let result = alt.exec(tokens, source, 0);

        assert!(result.is_some());
        assert_eq!(result.unwrap().tokens_consumed, 1);
    }

    #[test]
    fn test_alternation_none_match() {
        let doc = Document::new_plain_english_curated("hello world");
        let tokens = doc.get_tokens();
        let source = doc.get_source();

        let alt = Alternation::new(vec![
            Box::new(Atom::word("foo")),
            Box::new(Atom::word("bar")),
        ]);

        let result = alt.exec(tokens, source, 0);

        assert!(result.is_none());
    }
}
