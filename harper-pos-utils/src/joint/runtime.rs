//! Runtime inference: loads a trained model record and runs the joint tagger+chunker.

use std::path::Path;

use crate::joint::char_vocab::CharVocab;
use crate::joint::model::JointModel;
use crate::joint::suffix_vocab::SuffixVocab;
use burn::module::Module;
use burn::record::{HalfPrecisionSettings, NamedMpkBytesRecorder, NamedMpkFileRecorder, Recorder};
use burn::tensor::backend::Backend;
use burn_ndarray::{NdArray, NdArrayDevice};

/// Geometry needed to reconstruct a `JointModel` before loading weights.
#[derive(Debug, Clone, Copy)]
pub struct JointArch {
    pub char_dim: usize,
    pub conv_channels: usize,
    pub hidden: usize,
    pub suffix_k: usize,
    pub suffix_dim: usize,
    pub max_word: usize,
}

/// Save the model weights as `model.mpk` (fp16 named-msgpack), the char
/// vocabulary as `char_vocab.json`, and the suffix vocabulary as
/// `suffix_vocab.json` into `dir`.
///
/// # Panics
/// Panics if `dir` cannot be created or any file cannot be written.
pub fn save_joint<B: Backend>(
    model: &JointModel<B>,
    char_vocab: &CharVocab,
    suffix_vocab: &SuffixVocab,
    dir: &Path,
    _cfg: &JointArch,
) {
    std::fs::create_dir_all(dir).expect("create artifact dir");
    let recorder = NamedMpkFileRecorder::<HalfPrecisionSettings>::new();
    // save_file automatically appends the ".mpk" extension, so pass stem only.
    model
        .clone()
        .save_file(dir.join("model"), &recorder)
        .expect("save model.mpk");
    std::fs::write(dir.join("char_vocab.json"), char_vocab.to_json())
        .expect("write char_vocab.json");
    std::fs::write(dir.join("suffix_vocab.json"), suffix_vocab.to_json())
        .expect("write suffix_vocab.json");
}

/// Load the model weights from raw `model.mpk` bytes and reconstruct the
/// `CharVocab` and `SuffixVocab` from their JSON strings.  Uses fp16 precision
/// (must match save).
pub fn load_joint_from_bytes(
    model_bytes: &[u8],
    vocab_json: &str,
    suffix_vocab_json: &str,
    cfg: &JointArch,
) -> (JointModel<NdArray>, CharVocab, SuffixVocab) {
    let device = NdArrayDevice::Cpu;
    let char_vocab = CharVocab::from_json(vocab_json);
    let suffix_vocab = SuffixVocab::from_json(suffix_vocab_json);
    let recorder = NamedMpkBytesRecorder::<HalfPrecisionSettings>::new();
    let record = recorder
        .load(model_bytes.to_vec(), &device)
        .expect("load model record from bytes");
    let model = JointModel::<NdArray>::new(
        char_vocab.len(),
        cfg.char_dim,
        cfg.conv_channels,
        cfg.hidden,
        suffix_vocab.len(),
        cfg.suffix_dim,
        &device,
    )
    .load_record(record);
    (model, char_vocab, suffix_vocab)
}

use crate::annotator::TagSet;
use crate::joint::batch::JointBatch;
use crate::{Annotator, Chunker, Tagger, UPOS};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Map a 0-based class index to UPOS.
/// Classes `0..=15` map to the 16 `UPOS` variants in declaration order
/// (`ADJ`..`VERB`), i.e. the class equals the variant's discriminant.
/// `TAG_PAD_CLASS` (16) and anything else out of range return `None`.
///
/// A direct `match` (jump table) rather than a scan over `UPOS::iter()`. The
/// arms mirror the enum's declaration order; `index_to_upos_maps_classes`
/// checks every arm against the real discriminant, so a reorder or a new
/// variant fails the test loudly instead of silently mis-tagging.
pub fn index_to_upos(class: usize) -> Option<UPOS> {
    use UPOS::*;
    Some(match class {
        0 => ADJ,
        1 => ADP,
        2 => ADV,
        3 => AUX,
        4 => CCONJ,
        5 => DET,
        6 => INTJ,
        7 => NOUN,
        8 => NUM,
        9 => PART,
        10 => PRON,
        11 => PROPN,
        12 => PUNCT,
        13 => SCONJ,
        14 => SYM,
        15 => VERB,
        _ => return None,
    })
}

/// Index of the maximum value in `row`, first occurrence on ties. Mirrors
/// burn's ndarray `argmax` (a strictly-greater fold from index 0), so the
/// chosen tag is identical to the previous tensor-`argmax(2)` path including
/// tie-breaking. `row` must be non-empty.
fn argmax_first(row: &[f32]) -> usize {
    let mut am = 0;
    let mut max = row[0];
    for (i, &v) in row.iter().enumerate() {
        if v > max {
            max = v;
            am = i;
        }
    }
    am
}

/// Sentence (token list) -> cached annotation: per-token plausible-tag sets
/// (argmax first) and NP-membership flags — both derived from one forward pass.
type AnnotateCache = LruCache<Vec<String>, (Vec<TagSet>, Vec<bool>)>;

pub struct JointRuntime {
    model: JointModel<NdArray>,
    char_vocab: CharVocab,
    suffix_vocab: SuffixVocab,
    max_word: usize,
    cache: Mutex<AnnotateCache>,
}

impl JointRuntime {
    pub fn new(
        model: JointModel<NdArray>,
        char_vocab: CharVocab,
        suffix_vocab: SuffixVocab,
        max_word: usize,
        capacity: NonZeroUsize,
    ) -> Self {
        Self {
            model,
            char_vocab,
            suffix_vocab,
            max_word,
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    /// One forward pass producing both per-token outputs together — the
    /// plausible-tag (top-k) sets (argmax first) and NP-membership flags —
    /// cached per sentence. Tagging, probability-aware top-k, and chunking the
    /// same sentence therefore cost a single inference, not three (the tag head
    /// feeds both the argmax and the softmax top-k; the most-likely tag is the
    /// first element of each set).
    fn infer(&self, sentence: &[String]) -> (Vec<TagSet>, Vec<bool>) {
        // Top-k floor, tuned against the lint suite: low enough to admit the
        // correct rank-2 reading of an ambiguous homograph (observed as low as
        // ~0.07), high enough that a confidently-tagged token stays a singleton.
        const FLOOR: f32 = 0.05;
        if sentence.is_empty() {
            return (Vec::new(), Vec::new());
        }
        // Look up by borrowed slice — `Vec<String>: Borrow<[String]>` and both
        // hash identically, so a cache hit costs only the hash, no key clone.
        // The owned key is allocated only on a miss (the `put` below).
        if let Ok(mut c) = self.cache.try_lock()
            && let Some(hit) = c.get(sentence)
        {
            return hit.clone();
        }

        // Cache miss: one owned copy serves both the forward pass and the
        // cache insert below.
        let owned: Vec<String> = sentence.to_vec();

        // A single forward pass (batch = 1) yields both heads. Inference reads
        // only the char/suffix inputs, so build the lean inference batch — no
        // gold-label buffers, no dummy tag/np vectors.
        let device = NdArrayDevice::Cpu;
        let b = JointBatch::build_inference(
            std::slice::from_ref(&owned),
            &self.char_vocab,
            &self.suffix_vocab,
            self.max_word,
        );
        let (tag_logits, chunk_logits) =
            self.model
                .forward(b.char_ids(&device), b.suffix_ids(&device), 1, b.max_sent);

        // tag_logits: [1, max_sent, TAG_CLASSES] -> read ONCE into a flat buffer.
        // The chosen tag (argmax) and the softmax top-k are both derived per row
        // from `flat` below via `argmax_first`, so there is no separate argmax
        // tensor op and no clone of the logits tensor.
        let flat: Vec<f32> = tag_logits.into_data().iter::<f32>().collect();
        // chunk_logits: [1, max_sent]
        let np_vec: Vec<f32> = chunk_logits.into_data().iter::<f32>().collect();

        let n = sentence.len();
        let tc = crate::joint::TAG_CLASSES;
        let mut topk: Vec<TagSet> = Vec::with_capacity(n);
        let mut np = Vec::with_capacity(n);
        for i in 0..n {
            let row = &flat[i * tc..(i + 1) * tc];
            // Chosen tag = argmax of this row; its value doubles as the softmax shift.
            let am = argmax_first(row);
            let max = row[am];
            // Softmax over the row into a fixed stack buffer (no per-token heap
            // alloc) — `row.len() == TAG_CLASSES`, so the array fits exactly.
            let mut exps = [0f32; crate::joint::TAG_CLASSES];
            let mut sum = 0f32;
            for (c, &x) in row.iter().enumerate() {
                let e = (x - max).exp();
                exps[c] = e;
                sum += e;
            }
            // Plausible-tag set with the argmax FIRST, then any runner-up that
            // clears the floor. Argmax-first lets a caller read the most-likely
            // tag as `set.first()`. A `SmallVec` keeps the common 1-3 tag set
            // inline (no heap allocation) and matches harper-core's
            // `pos_tag_topk` type, so it flows in without re-collecting.
            let mut set = TagSet::new();
            if let Some(u) = index_to_upos(am) {
                set.push(u);
            }
            for (c, e) in exps.iter().enumerate() {
                if c != am
                    && e / sum >= FLOOR
                    && let Some(u) = index_to_upos(c)
                {
                    set.push(u);
                }
            }
            topk.push(set);

            np.push(np_vec[i] > 0.5);
        }

        if let Ok(mut c) = self.cache.try_lock() {
            c.put(owned, (topk.clone(), np.clone()));
        }
        (topk, np)
    }
}

/// The joint model is a unified annotator: the plausible-tag sets and NP flags
/// both come from one cached forward pass ([`Self::infer`]).
impl Annotator for JointRuntime {
    fn annotate(&self, sentence: &[String]) -> (Vec<TagSet>, Vec<bool>) {
        self.infer(sentence)
    }
}

// `Tagger` and `Chunker` remain implemented so the legacy `brill_tagger()` /
// `burn_chunker()` accessors keep working; both just read the shared `infer`.
impl Chunker for JointRuntime {
    fn chunk_sentence(&self, sentence: &[String], _tags: &[Option<UPOS>]) -> Vec<bool> {
        self.infer(sentence).1
    }
}

impl Tagger for JointRuntime {
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        // The most-likely tag is the first of each plausible-tag set.
        self.infer(sentence)
            .0
            .iter()
            .map(|set| set.first().copied())
            .collect()
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use crate::joint::char_vocab::CharVocab;
    use crate::joint::model::JointModel;
    use crate::joint::suffix_vocab::SuffixVocab;
    use crate::{Annotator, Chunker, Tagger};
    use burn_ndarray::NdArrayDevice;
    use std::num::NonZeroUsize;
    use std::rc::Rc;

    #[test]
    fn tagger_and_chunker_share_one_annotate() {
        let device = NdArrayDevice::Cpu;
        let sents = vec![vec!["the".to_string(), "dog".to_string()]];
        let char_vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let model = JointModel::new(char_vocab.len(), 8, 16, 12, suffix_vocab.len(), 4, &device);
        let rt = Rc::new(JointRuntime::new(
            model,
            char_vocab,
            suffix_vocab,
            6,
            NonZeroUsize::new(8).unwrap(),
        ));

        let sentence = vec!["the".to_string(), "dog".to_string()];
        // Tagger, Chunker, and the unified Annotator all read the same cached
        // forward pass, so their views agree.
        let tags = Tagger::tag_sentence(&*rt, &sentence);
        let np = Chunker::chunk_sentence(&*rt, &sentence, &tags);

        assert_eq!(tags.len(), 2);
        assert_eq!(np.len(), 2);
        // The unified Annotator returns the same NP flags as the Chunker view,
        // and its tag sets lead with the Tagger's argmax (all read one cached
        // pass).
        let (a_sets, a_np) = Annotator::annotate(&*rt, &sentence);
        assert_eq!(np, a_np);
        assert_eq!(a_sets.len(), 2);
        let argmax: Vec<_> = a_sets.iter().map(|s| s.first().copied()).collect();
        assert_eq!(tags, argmax);
    }

    #[test]
    fn index_to_upos_maps_classes() {
        use strum::IntoEnumIterator;
        // Drift guard: every UPOS variant must round-trip through its own
        // discriminant, keeping index_to_upos's match arms in lock-step with the
        // enum's declaration order. A reorder or a new variant fails here.
        let mut count = 0;
        for u in crate::UPOS::iter() {
            assert_eq!(index_to_upos(u as usize), Some(u));
            count += 1;
        }
        assert_eq!(
            count, 16,
            "UPOS variant count changed; update index_to_upos"
        );
        // TAG_PAD_CLASS (16), the class-count sentinel (17), and anything else -> None.
        assert_eq!(index_to_upos(crate::joint::TAG_PAD_CLASS), None);
        assert_eq!(index_to_upos(crate::joint::TAG_CLASSES), None);
        assert_eq!(index_to_upos(999), None);
    }

    #[test]
    fn argmax_first_matches_burn_tiebreak() {
        // Strictly-greater fold => first occurrence of the max wins on ties,
        // exactly like burn's ndarray argmax (base.rs `e > acc.0`).
        assert_eq!(argmax_first(&[1.0, 3.0, 2.0]), 1);
        assert_eq!(argmax_first(&[3.0, 3.0, 1.0]), 0); // tie at 0,1 -> first (0)
        assert_eq!(argmax_first(&[1.0, 3.0, 3.0]), 1); // tie at 1,2 -> first (1)
        assert_eq!(argmax_first(&[-1.0, -3.0, -1.0]), 0); // negatives, tie -> first (0)
        assert_eq!(argmax_first(&[5.0]), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint::char_vocab::CharVocab;
    use crate::joint::model::JointModel;
    use crate::joint::suffix_vocab::SuffixVocab;
    use burn_ndarray::{NdArray, NdArrayDevice};

    fn arch() -> JointArch {
        JointArch {
            char_dim: 8,
            conv_channels: 16,
            hidden: 12,
            suffix_k: 3,
            suffix_dim: 4,
            max_word: 6,
        }
    }

    #[test]
    fn save_then_load_preserves_forward() {
        let device = NdArrayDevice::Cpu;
        let sents = vec![vec!["hi".to_string(), "yo".to_string()]];
        let char_vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let a = arch();
        let model = JointModel::<NdArray>::new(
            char_vocab.len(),
            a.char_dim,
            a.conv_channels,
            a.hidden,
            suffix_vocab.len(),
            a.suffix_dim,
            &device,
        );

        let dir = std::env::temp_dir().join("harper_joint_roundtrip");
        save_joint(&model, &char_vocab, &suffix_vocab, &dir, &a);

        let model_bytes = std::fs::read(dir.join("model.mpk")).unwrap();
        let vocab_json = std::fs::read_to_string(dir.join("char_vocab.json")).unwrap();
        let suffix_vocab_json = std::fs::read_to_string(dir.join("suffix_vocab.json")).unwrap();
        let (loaded, _v, _sv) =
            load_joint_from_bytes(&model_bytes, &vocab_json, &suffix_vocab_json, &a);

        // Same input -> (near-)equal output. fp16 round-trip introduces tiny error.
        let char_ids_data: Vec<i32> = vec![2, 3, 0, 0, 0, 0, 4, 5, 0, 0, 0, 0]; // [2 words, 6 chars]
        let char_ids = burn::tensor::Tensor::<NdArray, 1, burn::tensor::Int>::from_data(
            burn::tensor::TensorData::from(char_ids_data.as_slice()),
            &device,
        )
        .reshape([2, a.max_word]);
        let suffix_ids = burn::tensor::Tensor::<NdArray, 1, burn::tensor::Int>::from_data(
            burn::tensor::TensorData::from([0i32, 0].as_slice()),
            &device,
        )
        .reshape([1, 2]);
        let (o1, _) = model.forward(char_ids.clone(), suffix_ids.clone(), 1, 2);
        let (o2, _) = loaded.forward(char_ids, suffix_ids, 1, 2);
        let diff = (o1 - o2).abs().max().into_scalar();
        assert!(
            burn::tensor::cast::ToElement::to_f32(&diff) < 1e-2,
            "fp16 round-trip drift too large"
        );
    }
}
