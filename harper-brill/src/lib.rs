use std::num::NonZero;
use std::rc::Rc;

use harper_pos_utils::joint::runtime::{JointArch, JointRuntime, load_joint_from_bytes};
pub use harper_pos_utils::{
    Annotator, BrillChunker, BrillTagger, BurnChunkerCpu, CachedChunker, Chunker, FreqDict, TagSet,
    Tagger, UPOS,
};

// The English part-of-speech tagger and noun-phrase chunker are now a single
// joint neural model (see `harper-pos-utils/src/joint`). One char-CNN + BiLSTM
// encoder feeds two heads — a UPOS tag head and an NP-membership head — so both
// `brill_tagger()` and `burn_chunker()` are served by the same memoized runtime.
const JOINT_MODEL_BIN: &[u8] = include_bytes!("../finished_joint/model.mpk");
const JOINT_CHAR_VOCAB: &str = include_str!("../finished_joint/char_vocab.json");
const JOINT_SUFFIX_VOCAB: &str = include_str!("../finished_joint/suffix_vocab.json");

// Architecture hyper-parameters. MUST match the values the model was trained
// with — see the `regenerate_english_joint` example's `ARCH` constant.
const JOINT_ARCH: JointArch = JointArch {
    char_dim: 24,
    conv_channels: 64,
    hidden: 128,
    suffix_k: 3,
    suffix_dim: 32,
    max_word: 20,
};

thread_local! {
    // burn's NdArray model is `!Sync`, so the runtime is a per-thread `Rc`
    // rather than a process-wide `Arc`. Inference is memoized inside the runtime.
    static JOINT: Rc<JointRuntime> = Rc::new(build_joint());
}

fn build_joint() -> JointRuntime {
    let (model, vocab, suffix_vocab) =
        load_joint_from_bytes(JOINT_MODEL_BIN, JOINT_CHAR_VOCAB, JOINT_SUFFIX_VOCAB, &JOINT_ARCH);
    JointRuntime::new(
        model,
        vocab,
        suffix_vocab,
        JOINT_ARCH.max_word,
        NonZero::new(10_000).unwrap(),
    )
}

/// Get the shared, lazily-initialized English part-of-speech tagger. There is
/// one instance per thread; inference is memoized.
pub fn brill_tagger() -> Rc<dyn Tagger> {
    JOINT.with(|j| j.clone() as Rc<dyn Tagger>)
}

/// Get the shared, lazily-initialized English noun-phrase chunker. Backed by the
/// same joint model as [`brill_tagger`]; one instance per thread, memoized.
pub fn burn_chunker() -> Rc<dyn Chunker> {
    JOINT.with(|j| j.clone() as Rc<dyn Chunker>)
}

/// Get the shared English [`Annotator`] — the joint model exposed as a single
/// tagging-plus-chunking lookup. Prefer this over pairing [`brill_tagger`] with
/// [`burn_chunker`] when you need both: it returns the argmax tags, the
/// plausible-tag (top-k) sets, and NP flags from one cached forward pass. One
/// instance per thread, memoized.
pub fn annotator() -> Rc<dyn Annotator> {
    JOINT.with(|j| j.clone() as Rc<dyn Annotator>)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test: the joint model loads from the embedded artifacts and
    /// produces tag + NP vectors of the right shape on a small English sentence.
    #[test]
    fn joint_tagger_and_chunker_load_and_run() {
        let sent: Vec<String> = ["The", "dog", "ran", "fast", "."]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let tags = brill_tagger().tag_sentence(&sent);
        assert_eq!(tags.len(), sent.len());
        let np = burn_chunker().chunk_sentence(&sent, &tags);
        assert_eq!(np.len(), sent.len());
    }
}
