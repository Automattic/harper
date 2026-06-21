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
/// This is widely useful for various applications. [See here.](https://en.wikipedia.org/wiki/Part-of-speech_tagging)
pub trait Tagger {
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>>;

    /// Like [`Tagger::tag_sentence`], but returns the set of *plausible* POS tags
    /// per token — the argmax plus any close runner-up above a probability floor.
    /// Enables probability-aware linting: a genuine homograph ("books" = NOUN or
    /// VERB) carries both readings so a rule can match the one it needs instead
    /// of being defeated by a hair-thin argmax. The default just wraps the single
    /// argmax tag, so taggers without a probability distribution still work.
    fn tag_sentence_topk(&self, sentence: &[String]) -> Vec<Vec<UPOS>> {
        self.tag_sentence(sentence)
            .into_iter()
            .map(|t| t.into_iter().collect())
            .collect()
    }
}
