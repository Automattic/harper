use harper_comments::CommentParser;
use harper_core::{
    Lrc, Masker, Token,
    parsers::{Markdown, MarkdownOptions, Mask, Parser},
};

mod masker;
use harper_core::spell::MutableDictionary;
use itertools::Itertools;
use masker::LiterateHaskellMasker;

/// Parses a Literate Haskell document by masking out the code and considering text as Markdown.
pub struct LiterateHaskellParser {
    inner: Lrc<dyn Parser>,
}

impl LiterateHaskellParser {
    pub fn new(inner: Lrc<dyn Parser>) -> Self {
        Self { inner }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: Lrc::new(Markdown::new(markdown_options)),
        }
    }

    pub fn create_ident_dict(
        &self,
        source: &[char],
        markdown_options: MarkdownOptions,
    ) -> Option<MutableDictionary> {
        let parser = CommentParser::new_from_language_id("haskell", markdown_options).unwrap();
        let mask = LiterateHaskellMasker::code_only().create_mask(source);

        let code = mask
            .iter_allowed(source)
            .flat_map(|(_, src)| src.to_owned())
            .collect_vec();
        parser.create_ident_dict(&code)
    }
}

impl Parser for LiterateHaskellParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        Mask::new(LiterateHaskellMasker::text_only(), self.inner.clone()).parse(source)
    }
}
