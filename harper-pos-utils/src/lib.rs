mod chunker;
#[cfg(feature = "training")]
mod conllu_utils;
mod patch_criteria;
mod tagger;
mod upos;

pub use tagger::{BrillTagger, FreqDict, FreqDictBuilder, Tagger};
pub use upos::{UPOS, UPOSIter};
