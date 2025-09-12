use typst_syntax::Source;

/// Encapsulation of the translation between byte-based spans and char-based spans. This is used to
/// avoid recomputing the number of characters between the beginning of the file and the current
/// byte since `typst_syntax` uses byte spans while we use char spans.
#[derive(Debug, Clone, Copy)]
pub struct OffsetCursor<'a> {
    doc: &'a Source,
    pub char: usize,
    pub byte: usize,
}

impl<'a> OffsetCursor<'a> {
    pub fn new(doc: &'a Source) -> Self {
        Self {
            doc,
            char: 0,
            byte: 0,
        }
    }

    /// Returns a new [`OffsetCursor`] at the given byte based on the current cursor.
    pub fn push_to(self, new_byte: usize) -> Option<Self> {
        assert!(new_byte >= self.byte);

        if new_byte == self.byte {
            return Some(self);
        }

        Some(Self {
            char: self.char + self.doc.get(self.byte..new_byte)?.chars().count(),
            byte: new_byte,
            ..self
        })
    }

    /// Returns a new [`OffsetCursor`] at the beginning of the given [`typst_syntax::Span`] based
    /// on the current cursor.
    pub fn push_to_span(self, span: typst_syntax::Span) -> Option<Self> {
        let new_byte = self.doc.range(span)?.start;

        self.push_to(new_byte)
    }
}
