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
    vocab: &CharVocab,
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
    std::fs::write(dir.join("char_vocab.json"), vocab.to_json()).expect("write char_vocab.json");
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
    let vocab = CharVocab::from_json(vocab_json);
    let suffix_vocab = SuffixVocab::from_json(suffix_vocab_json);
    let recorder = NamedMpkBytesRecorder::<HalfPrecisionSettings>::new();
    let record = recorder
        .load(model_bytes.to_vec(), &device)
        .expect("load model record from bytes");
    let model = JointModel::<NdArray>::new(
        vocab.len(),
        cfg.char_dim,
        cfg.conv_channels,
        cfg.hidden,
        suffix_vocab.len(),
        cfg.suffix_dim,
        &device,
    )
    .load_record(record);
    (model, vocab, suffix_vocab)
}

use crate::joint::batch::JointBatch;
use crate::{Chunker, Tagger, UPOS};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use strum::IntoEnumIterator;

/// Map a 0-based class index to UPOS.
/// Classes `0..=15` map to the 16 `UPOS` variants in declaration order
/// (`ADJ`..`VERB`), i.e. the class equals the variant's discriminant.
/// `TAG_PAD_CLASS` (16) and anything else out of range return `None`.
pub fn index_to_upos(class: usize) -> Option<UPOS> {
    UPOS::iter().find(|u| *u as usize == class)
}

/// Sentence (token list) -> cached `(tags, np)` annotation.
type AnnotateCache = LruCache<Vec<String>, (Vec<Option<UPOS>>, Vec<bool>)>;

pub struct JointRuntime {
    model: JointModel<NdArray>,
    vocab: CharVocab,
    suffix_vocab: SuffixVocab,
    max_word: usize,
    cache: Mutex<AnnotateCache>,
}

impl JointRuntime {
    pub fn new(
        model: JointModel<NdArray>,
        vocab: CharVocab,
        suffix_vocab: SuffixVocab,
        max_word: usize,
        capacity: NonZeroUsize,
    ) -> Self {
        Self {
            model,
            vocab,
            suffix_vocab,
            max_word,
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    pub fn annotate(&self, sentence: &[String]) -> (Vec<Option<UPOS>>, Vec<bool>) {
        if sentence.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let key: Vec<String> = sentence.to_vec();
        if let Ok(mut c) = self.cache.try_lock()
            && let Some(hit) = c.get(&key)
        {
            return hit.clone();
        }

        // One forward pass; batch = 1.
        let device = NdArrayDevice::Cpu;
        let dummy_tags = vec![None; sentence.len()];
        let dummy_np = vec![false; sentence.len()];
        let b = JointBatch::build(
            std::slice::from_ref(&key),
            std::slice::from_ref(&dummy_tags),
            std::slice::from_ref(&dummy_np),
            &self.vocab,
            &self.suffix_vocab,
            self.max_word,
        );
        let (tag_logits, chunk_logits) =
            self.model
                .forward(b.char_ids(&device), b.suffix_ids(&device), 1, b.max_sent);

        // tag_logits: [1, max_sent, TAG_CLASSES]
        // argmax(2) keeps the dim: [1, max_sent, 1] with dtype Int (i64).
        let classes = tag_logits.argmax(2).into_data();
        let class_vec: Vec<i64> = classes.iter::<i64>().collect();
        // chunk_logits: [1, max_sent]
        let np_data = chunk_logits.into_data();
        let np_vec: Vec<f32> = np_data.iter::<f32>().collect();

        let n = sentence.len();
        let mut tags = Vec::with_capacity(n);
        let mut np = Vec::with_capacity(n);
        for i in 0..n {
            tags.push(index_to_upos(class_vec[i] as usize));
            np.push(np_vec[i] > 0.5);
        }

        if let Ok(mut c) = self.cache.try_lock() {
            c.put(key, (tags.clone(), np.clone()));
        }
        (tags, np)
    }

    /// Per-token set of *plausible* POS tags: the argmax plus every tag whose
    /// softmax probability clears `FLOOR`. Powers probability-aware linting — a
    /// genuine homograph ("books" = NOUN or VERB; "right" = ADV or ADJ) keeps
    /// both readings so a rule can match the one it needs rather than being
    /// defeated by a hair-thin argmax.
    pub fn tag_topk(&self, sentence: &[String]) -> Vec<Vec<UPOS>> {
        // Tuned against the lint suite: low enough to admit the correct rank-2
        // reading of an ambiguous homograph (observed as low as ~0.07), high
        // enough that a confidently-tagged token stays a singleton set.
        const FLOOR: f32 = 0.05;
        if sentence.is_empty() {
            return Vec::new();
        }
        let device = NdArrayDevice::Cpu;
        let key: Vec<String> = sentence.to_vec();
        let dummy_tags = vec![None; sentence.len()];
        let dummy_np = vec![false; sentence.len()];
        let b = JointBatch::build(
            std::slice::from_ref(&key),
            std::slice::from_ref(&dummy_tags),
            std::slice::from_ref(&dummy_np),
            &self.vocab,
            &self.suffix_vocab,
            self.max_word,
        );
        let (tag_logits, _) =
            self.model
                .forward(b.char_ids(&device), b.suffix_ids(&device), 1, b.max_sent);
        let flat: Vec<f32> = tag_logits.into_data().iter::<f32>().collect(); // [max_sent * TAG_CLASSES]

        let n = sentence.len();
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let row = &flat[i * crate::joint::TAG_CLASSES..(i + 1) * crate::joint::TAG_CLASSES];
            let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let exps: Vec<f32> = row.iter().map(|x| (x - max).exp()).collect();
            let sum: f32 = exps.iter().sum();
            let argmax = exps
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(c, _)| c)
                .unwrap_or(0);
            let mut tags: Vec<UPOS> = Vec::new();
            for (c, e) in exps.iter().enumerate() {
                if (c == argmax || e / sum >= FLOOR)
                    && let Some(u) = index_to_upos(c)
                {
                    tags.push(u);
                }
            }
            out.push(tags);
        }
        out
    }
}

impl Chunker for JointRuntime {
    fn chunk_sentence(&self, sentence: &[String], _tags: &[Option<UPOS>]) -> Vec<bool> {
        self.annotate(sentence).1
    }
}

impl Tagger for JointRuntime {
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        self.annotate(sentence).0
    }
    fn tag_sentence_topk(&self, sentence: &[String]) -> Vec<Vec<UPOS>> {
        self.tag_topk(sentence)
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use crate::joint::char_vocab::CharVocab;
    use crate::joint::model::JointModel;
    use crate::joint::suffix_vocab::SuffixVocab;
    use crate::{Chunker, Tagger};
    use burn_ndarray::NdArrayDevice;
    use std::num::NonZeroUsize;
    use std::rc::Rc;

    #[test]
    fn tagger_and_chunker_share_one_annotate() {
        let device = NdArrayDevice::Cpu;
        let sents = vec![vec!["the".to_string(), "dog".to_string()]];
        let vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let model = JointModel::new(vocab.len(), 8, 16, 12, suffix_vocab.len(), 4, &device);
        let rt = Rc::new(JointRuntime::new(
            model,
            vocab,
            suffix_vocab,
            6,
            NonZeroUsize::new(8).unwrap(),
        ));

        let sentence = vec!["the".to_string(), "dog".to_string()];
        // JointRuntime implements both traits directly.
        let tags = Tagger::tag_sentence(&*rt, &sentence);
        let np = Chunker::chunk_sentence(&*rt, &sentence, &tags);

        assert_eq!(tags.len(), 2);
        assert_eq!(np.len(), 2);
        // annotate() is deterministic; calling again returns identical vectors.
        let (t2, n2) = rt.annotate(&sentence);
        assert_eq!(tags, t2);
        assert_eq!(np, n2);
    }

    #[test]
    fn index_to_upos_maps_classes() {
        // class 0 -> first variant -> UPOS::ADJ
        assert_eq!(index_to_upos(0), Some(crate::UPOS::ADJ));
        // class 15 -> last variant -> UPOS::VERB
        assert_eq!(index_to_upos(15), Some(crate::UPOS::VERB));
        // TAG_PAD_CLASS = 17 -> out of range -> None
        assert_eq!(index_to_upos(crate::joint::TAG_PAD_CLASS), None);
        // arbitrary large index -> None
        assert_eq!(index_to_upos(999), None);
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
        let vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let a = arch();
        let model = JointModel::<NdArray>::new(
            vocab.len(),
            a.char_dim,
            a.conv_channels,
            a.hidden,
            suffix_vocab.len(),
            a.suffix_dim,
            &device,
        );

        let dir = std::env::temp_dir().join("harper_joint_roundtrip");
        save_joint(&model, &vocab, &suffix_vocab, &dir, &a);

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
