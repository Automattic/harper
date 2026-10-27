use harper_core::spell::MutableDictionary;
use harper_core::{Mask, Masker, Span};
use harper_tree_sitter::TreeSitterMasker;

pub struct CommentMasker {
    inner: TreeSitterMasker,
    ignore_condition: Box<dyn Fn(&String) -> bool + Send + Sync>,
}

impl CommentMasker {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<MutableDictionary> {
        self.inner.create_ident_dict(source)
    }

    pub fn new(
        language: tree_sitter::Language,
        ts_node_condition: fn(&tree_sitter::Node) -> bool,
    ) -> Self {
        Self::new_with_ignore_condition(
            language,
            ts_node_condition,
            Box::new(|text| {
                text.contains("spellchecker:ignore")
                    || text.contains("spellchecker: ignore")
                    || text.contains("spell-checker:ignore")
                    || text.contains("spell-checker: ignore")
                    || text.contains("spellcheck:ignore")
                    || text.contains("spellcheck: ignore")
                    || text.contains("harper:ignore")
                    || text.contains("harper: ignore")
            }),
        )
    }

    pub fn new_with_ignore_condition(
        language: tree_sitter::Language,
        ts_node_condition: fn(&tree_sitter::Node) -> bool,
        ignore_condition: Box<dyn Fn(&String) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            inner: TreeSitterMasker::new(language, ts_node_condition),
            ignore_condition,
        }
    }
}

impl Masker for CommentMasker {
    fn create_mask(&self, source: &[char]) -> Mask {
        self.inner
            .create_mask(source)
            .iter_allowed(source)
            .filter_map(|(span, chars)| {
                let mut span = span;
                let mut text: String = chars.iter().collect();

                // A real shebang only applies to the first line of the file.
                // If tree-sitter merged the shebang with following comments,
                // keep linting the remainder of the comment block.
                if span.start == 0 && text.starts_with("#!") {
                    span = trim_leading_shebang(span, chars)?;
                    text = span.get_content(source).iter().collect();
                }

                if (self.ignore_condition)(&text) {
                    None
                } else {
                    Some(span)
                }
            })
            .collect()
    }
}

fn trim_leading_shebang(span: Span<char>, chars: &[char]) -> Option<Span<char>> {
    let first_line_end = chars
        .iter()
        .position(|c| *c == '\n')
        .map_or(chars.len(), |index| index + 1);

    let next_content = chars[first_line_end..]
        .iter()
        .position(|c| !c.is_whitespace())?;

    Some(Span::new(
        span.start + first_line_end + next_content,
        span.end,
    ))
}
