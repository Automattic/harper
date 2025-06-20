use lazy_static::lazy_static;
use std::sync::Arc;

pub use harper_pos_utils::{BrillChunker, BrillTagger, Chunker, FreqDict, Tagger, UPOS};

const BRILL_TAGGER_SOURCE: &str = include_str!("../trained_tagger_model.json");

lazy_static! {
    static ref BRILL_TAGGER: Arc<BrillTagger<FreqDict>> = Arc::new(uncached_brill_tagger());
}

fn uncached_brill_tagger() -> BrillTagger<FreqDict> {
    serde_json::from_str(BRILL_TAGGER_SOURCE).unwrap()
}

pub fn brill_tagger() -> Arc<BrillTagger<FreqDict>> {
    (*BRILL_TAGGER).clone()
}

const BRILL_CHUNKER_SOURCE: &str = include_str!("../trained_chunker_model.json");

lazy_static! {
    static ref BRILL_CHUNKER: Arc<BrillChunker> = Arc::new(uncached_brill_chunker());
}

fn uncached_brill_chunker() -> BrillChunker {
    serde_json::from_str(BRILL_CHUNKER_SOURCE).unwrap()
}

pub fn brill_chunker() -> Arc<BrillChunker> {
    (*BRILL_CHUNKER).clone()
}
