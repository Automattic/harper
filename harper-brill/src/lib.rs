use lazy_static::lazy_static;
use std::sync::Arc;

pub use harper_pos_utils::{BrillTagger, FreqDict, Tagger, UPOS};

const BRILL_TAGGER_SOURCE: &str = include_str!("../trained_brill_model.json");

lazy_static! {
    static ref BRILL_TAGGER: Arc<BrillTagger<FreqDict>> = Arc::new(uncached_brill_tagger());
}

fn uncached_brill_tagger() -> BrillTagger<FreqDict> {
    serde_json::from_str(BRILL_TAGGER_SOURCE).unwrap()
}

pub fn brill_tagger() -> Arc<BrillTagger<FreqDict>> {
    (*BRILL_TAGGER).clone()
}
