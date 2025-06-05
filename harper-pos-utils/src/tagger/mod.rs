mod brill_tagger;
#[cfg(feature = "training")]
mod error_counter;
mod freq_dict;
mod freq_dict_builder;

use crate::UPOS;

pub use brill_tagger::BrillTagger;
pub use freq_dict::FreqDict;
pub use freq_dict_builder::FreqDictBuilder;

/// An implementer of this trait is capable of assigned Part-of-Speech tags to a provided sentence.
pub trait Tagger {
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>>;
}
