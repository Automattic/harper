use harper_comments::CommentParser;
use harper_core::{
    parsers::{Markdown, Mask, Parser},
    FullDictionary, Masker, Token,
};

mod masker;
use itertools::Itertools;
use masker::LiterateHaskellMasker;

pub struct LiterateHaskellParser;

impl LiterateHaskellParser {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<FullDictionary> {
        let parser = CommentParser::new_from_language_id("haskell").unwrap();
        let mask = LiterateHaskellMasker::code_only().create_mask(source);

        let code = mask
            .iter_allowed(source)
            .flat_map(|(_, src)| src.to_owned())
            .collect_vec();
        parser.create_ident_dict(&code)
    }
}

impl Parser for LiterateHaskellParser {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        Mask::new(LiterateHaskellMasker::text_only(), Markdown).parse(source)
    }
}
