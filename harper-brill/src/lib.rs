use lazy_static::lazy_static;
use std::sync::Arc;

pub use harper_pos_utils::{BrillTagger, Tagger, UPOS};

const BRILL_TAGGER_SOURCE: &str = include_str!("../trained_brill_model.json");

lazy_static! {
    static ref BRILL_TAGGER: Arc<BrillTagger> = Arc::new(uncached_brill_tagger());
}

fn uncached_brill_tagger() -> BrillTagger {
    serde_json::from_str(BRILL_TAGGER_SOURCE).unwrap()
}

pub fn brill_tagger() -> Arc<BrillTagger> {
    (*BRILL_TAGGER).clone()
}
