use std::collections::HashSet;

use harper_core::spell::MutableDictionary;
use harper_core::{DictWordMetadata, Mask, Masker, Span};
use tree_sitter::{Language, Node, Tree, TreeCursor};

/// A Harper [`Masker`] that wraps a given tree-sitter language and a condition,
/// allowing you to selectively parse only specific tree-sitter nodes.
pub struct TreeSitterMasker {
    language: Language,
    node_condition: fn(&Node) -> bool,
}

impl TreeSitterMasker {
    pub fn new(language: Language, node_condition: fn(&Node) -> bool) -> Self {
        Self {
            language,
            node_condition,
        }
    }

    fn parse_root(&self, text: &str) -> Option<Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.language).unwrap();

        // TODO: Use incremental parsing
        parser.parse(text, None)
    }

    pub fn create_ident_dict(&self, source: &[char]) -> Option<MutableDictionary> {
        let text: String = source.iter().collect();

        // Byte-indexed
        let mut ident_spans = Vec::new();

        let tree = self.parse_root(&text)?;
        Self::visit_nodes(&mut tree.walk(), &mut |node: &Node| {
            if node.child_count() == 0 && node.kind().contains("ident") {
                ident_spans.push(node.byte_range().into())
            }
        });

        let ident_spans = byte_spans_to_char_spans(ident_spans, &text);

        let mut idents = HashSet::new();

        for span in ident_spans {
            idents.insert(span.get_content(source));
        }

        let idents: Vec<_> = idents
            .into_iter()
            .map(|ident| (ident, DictWordMetadata::default()))
            .collect();

        let mut dictionary = MutableDictionary::new();
        dictionary.extend_words(idents);

        Some(dictionary)
    }

    /// Visits the children of a TreeSitter node, searching for comments.
    ///
    /// Returns the BYTE spans of the comment position.
    fn extract_comments(&self, cursor: &mut TreeCursor, comments: &mut Vec<Span<u8>>) {
        Self::visit_nodes(cursor, &mut |node: &Node| {
            if (self.node_condition)(node) {
                comments.push(node.byte_range().into());
            }
        });
    }

    fn visit_nodes(cursor: &mut TreeCursor, visit: &mut impl FnMut(&Node)) {
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            let node = cursor.node();

            visit(&node);

            Self::visit_nodes(cursor, visit);

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        cursor.goto_parent();
    }
}

impl Masker for TreeSitterMasker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let text: String = source.iter().collect();

        let Some(root) = self.parse_root(&text) else {
            return Mask::new_blank();
        };

        let mut comments_spans = Vec::new();

        self.extract_comments(&mut root.walk(), &mut comments_spans);
        let comments_spans = byte_spans_to_char_spans(comments_spans, &text);

        let mut mask = Mask::new_blank();

        for span in comments_spans {
            mask.push_allowed(span);
        }

        mask.merge_whitespace_sep(source);

        mask
    }
}

/// Converts a set of byte-indexed [`Span`]s to char-index Spans and returns them.
/// NOTE: Will sort the given slice by their [`Span::start`].
///
/// If any spans overlap, it will merge them.
fn byte_spans_to_char_spans(mut byte_spans: Vec<Span<u8>>, source: &str) -> Vec<Span<char>> {
    byte_spans.sort_unstable_by_key(|s| s.start);

    // merge overlapping spans
    let mut spans = Vec::with_capacity(byte_spans.len());
    for span in byte_spans {
        match spans.last_mut() {
            Some(last) if !span.overlaps_with(*last) => spans.push(span),
            Some(last) => {
                // ranges overlap, we can merge them
                last.end = span.end;
            }
            None => spans.push(span),
        }
    }

    // Convert byte spans to char spans.
    spans
        .iter()
        .scan((0, 0), |(last_byte_pos, last_char_pos), span| {
            let byte_span = *span;

            *last_char_pos += source[*last_byte_pos..byte_span.start].chars().count();
            let start = *last_char_pos;

            *last_char_pos += source[byte_span.start..byte_span.end].chars().count();
            let end = *last_char_pos;

            *last_byte_pos = byte_span.end;
            Some(Span::new(start, end))
        })
        .collect()
}
