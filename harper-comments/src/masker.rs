use harper_core::{Masker, MutableDictionary};
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
                    || text.starts_with("#!")
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
    fn create_mask(&self, source: &[char]) -> harper_core::Mask {
        self.inner
            .create_mask(source)
            .iter_allowed(source)
            .map(|(span, chars)| (span, chars.iter().collect::<String>()))
            .filter(|(_, text)| !(self.ignore_condition)(text))
            .map(|(span, _)| span)
            .collect()
    }
}
