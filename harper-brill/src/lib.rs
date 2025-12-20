use lazy_static::lazy_static;
use std::num::NonZero;
use std::rc::Rc;
use std::sync::Arc;

pub use harper_pos_utils::{
    BrillChunker, BrillTagger, BurnChunkerCpu, CachedChunker, Chunker, FreqDict, Tagger, UPOS,
};

const BRILL_TAGGER_SOURCE: &str = include_str!("../trained_tagger_model.json");

lazy_static! {
    static ref BRILL_TAGGER: Arc<BrillTagger<FreqDict>> = Arc::new(uncached_brill_tagger());
}

fn uncached_brill_tagger() -> BrillTagger<FreqDict> {
    serde_json::from_str(BRILL_TAGGER_SOURCE).unwrap()
}

/// Get a copy of a shared, lazily-initialized [`BrillTagger`]. There will be only one instance
/// per-process.
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

/// Get a copy of a shared, lazily-initialized [`BrillChunker`]. There will be only one instance
/// per-process.
pub fn brill_chunker() -> Arc<BrillChunker> {
    (*BRILL_CHUNKER).clone()
}

const BURN_CHUNKER_VOCAB: &[u8; 627993] = include_bytes!("../finished_chunker/vocab.json");
const BURN_CHUNKER_BIN: &[u8; 806312] = include_bytes!("../finished_chunker/model.mpk");

thread_local! {
    static BURN_CHUNKER: Rc<CachedChunker<BurnChunkerCpu>> =  Rc::new(uncached_burn_chunker());
}

fn uncached_burn_chunker() -> CachedChunker<BurnChunkerCpu> {
    CachedChunker::new(
        BurnChunkerCpu::load_from_bytes_cpu(BURN_CHUNKER_BIN, BURN_CHUNKER_VOCAB, 6, 0.3),
        NonZero::new(10000).unwrap(),
    )
}

/// Get a copy of a shared, lazily-initialized [`BurnChunkerCpu`]. There will be only one instance
/// per-process. Since neural net inference is extremely expensive, this chunker is memoized as
/// well.
pub fn burn_chunker() -> Rc<CachedChunker<BurnChunkerCpu>> {
    (BURN_CHUNKER).with(|c| c.clone())
}
