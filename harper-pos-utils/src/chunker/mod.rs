use crate::UPOS;

mod brill_chunker;
mod burn_chunker;
mod cached_chunker;
#[cfg(feature = "training")]
mod np_extraction;
mod upos_freq_dict;

pub use brill_chunker::BrillChunker;
pub use burn_chunker::{BurnChunker, BurnChunkerCpu};
pub use cached_chunker::CachedChunker;
pub use upos_freq_dict::UPOSFreqDict;

/// An implementer of this trait is capable of identifying the noun phrases in a provided sentence.
pub trait Chunker {
    /// Iterate over the sentence, identifying the noun phrases contained within.
    /// A token marked `true` is a component of a noun phrase.
    /// A token marked `false` is not.
    fn chunk_sentence(&self, sentence: &[String], tags: &[Option<UPOS>]) -> Vec<bool>;
}
