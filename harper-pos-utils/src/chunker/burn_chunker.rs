use crate::{UPOS, chunker::Chunker};
#[cfg(feature = "training")]
use burn::backend::Autodiff;

#[cfg(feature = "training")]
use burn::nn::loss::{MseLoss, Reduction};
use burn::nn::{Dropout, DropoutConfig};
#[cfg(feature = "training")]
use burn::optim::{GradientsParams, Optimizer};
use burn::record::{FullPrecisionSettings, NamedMpkBytesRecorder, NamedMpkFileRecorder, Recorder};
use burn::tensor::TensorData;
#[cfg(feature = "training")]
use burn::tensor::backend::AutodiffBackend;

use burn::{
    module::Module,
    nn::{BiLstmConfig, EmbeddingConfig, LinearConfig},
    tensor::{Int, Tensor, backend::Backend},
};
use burn_ndarray::{NdArray, NdArrayDevice};
use hashbrown::HashMap;
use std::path::Path;

const UNK_IDX: usize = 1;

#[derive(Module, Debug)]
struct NpModel<B: Backend> {
    embedding_words: burn::nn::Embedding<B>,
    embedding_upos: burn::nn::Embedding<B>,
    lstm: burn::nn::BiLstm<B>,
    linear_out: burn::nn::Linear<B>,
    dropout: Dropout,
}

impl<B: Backend> NpModel<B> {
    fn new(vocab: usize, word_embed_dim: usize, dropout: f32, device: &B::Device) -> Self {
        let upos_embed = 8;
        let total_embed = word_embed_dim + upos_embed;

        Self {
            embedding_words: EmbeddingConfig::new(vocab, word_embed_dim).init(device),
            embedding_upos: EmbeddingConfig::new(20, upos_embed).init(device),
            lstm: BiLstmConfig::new(total_embed, total_embed, false).init(device),
            // Multiply by two because the BiLSTM emits double the hidden parameters
            linear_out: LinearConfig::new(total_embed * 2, 1).init(device),
            dropout: DropoutConfig::new(dropout as f64).init(),
        }
    }

    fn forward(
        &self,
        word_tens: Tensor<B, 2, Int>,
        tag_tens: Tensor<B, 2, Int>,
        use_dropout: bool,
    ) -> Tensor<B, 2> {
        let word_embed = self.embedding_words.forward(word_tens);
        let tag_embed = self.embedding_upos.forward(tag_tens);

        let mut x = Tensor::cat(vec![word_embed, tag_embed], 2);

        if use_dropout {
            x = self.dropout.forward(x);
        }

        let (mut x, _) = self.lstm.forward(x, None);

        if use_dropout {
            x = self.dropout.forward(x);
        }

        let x = self.linear_out.forward(x);
        x.squeeze::<2>(2)
    }
}

pub struct BurnChunker<B: Backend> {
    vocab: HashMap<String, usize>,
    model: NpModel<B>,
    device: B::Device,
}

impl<B: Backend> BurnChunker<B> {
    fn idx(&self, tok: &str) -> usize {
        *self.vocab.get(tok).unwrap_or(&UNK_IDX)
    }

    fn to_tensors(
        &self,
        sent: &[String],
        tags: &[Option<UPOS>],
    ) -> (Tensor<B, 2, Int>, Tensor<B, 2, Int>) {
        // Interleave with UPOS tags
        let idxs: Vec<_> = sent.iter().map(|t| self.idx(t) as i32).collect();

        let upos: Vec<_> = tags
            .iter()
            .map(|t| t.map(|o| o as i32 + 2).unwrap_or(1))
            .collect();

        let word_tensor =
            Tensor::<B, 1, Int>::from_data(TensorData::from(idxs.as_slice()), &self.device)
                .reshape([1, sent.len()]);

        let tag_tensor =
            Tensor::<B, 1, Int>::from_data(TensorData::from(upos.as_slice()), &self.device)
                .reshape([1, sent.len()]);

        (word_tensor, tag_tensor)
    }

    pub fn save_to(&self, dir: impl AsRef<Path>) {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).unwrap();

        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        self.model
            .clone()
            .save_file(dir.join("model.mpk"), &recorder)
            .unwrap();

        let vocab_bytes = serde_json::to_vec(&self.vocab).unwrap();
        std::fs::write(dir.join("vocab.json"), vocab_bytes).unwrap();
    }

    pub fn load_from_bytes(
        model_bytes: impl AsRef<[u8]>,
        vocab_bytes: impl AsRef<[u8]>,
        embed_dim: usize,
        dropout: f32,
        device: B::Device,
    ) -> Self {
        let vocab: HashMap<String, usize> = serde_json::from_slice(vocab_bytes.as_ref()).unwrap();

        let recorder = NamedMpkBytesRecorder::<FullPrecisionSettings>::new();

        let owned_data = model_bytes.as_ref().to_vec();
        let record = recorder.load(owned_data, &device).unwrap();

        let model = NpModel::new(vocab.len(), embed_dim, dropout, &device);
        let model = model.load_record(record);

        Self {
            vocab,
            model,
            device,
        }
    }
}

#[cfg(feature = "training")]
struct ExtractedSentences(
    Vec<Vec<String>>,
    Vec<Vec<Option<UPOS>>>,
    Vec<Vec<bool>>,
    HashMap<String, usize>,
);

#[cfg(feature = "training")]
impl<B: Backend + AutodiffBackend> BurnChunker<B> {
    fn to_label(&self, labels: &[bool]) -> Tensor<B, 2> {
        let ys: Vec<_> = labels.iter().map(|b| if *b { 1. } else { 0. }).collect();

        Tensor::<B, 1, _>::from_data(TensorData::from(ys.as_slice()), &self.device)
            .reshape([1, labels.len()])
    }

    pub fn train(
        training_files: &[impl AsRef<Path>],
        test_file: &impl AsRef<Path>,
        word_embed_dim: usize,
        dropout: f32,
        epochs: usize,
        lr: f64,
        device: B::Device,
    ) -> Self {
        use burn::tensor::cast::ToElement;

        println!("Preparing datasets...");
        let ExtractedSentences(sents, tags, labs, vocab) =
            Self::extract_sents_from_files(training_files);

        println!("Preparing model and training config...");

        let mut model = NpModel::<B>::new(vocab.len(), word_embed_dim, dropout, &device);
        let opt_config = burn::optim::AdamConfig::new();
        let mut opt = opt_config.init();

        let util = BurnChunker {
            vocab: vocab.clone(),
            model: model.clone(),
            device: device.clone(),
        };

        let loss_fn = MseLoss::new();
        let mut last_score = 0.;

        println!("Training...");

        for _ in 0..epochs {
            let mut total_loss = 0.;
            let mut total_tokens = 0;
            let mut total_correct: usize = 0;

            for (i, ((x, w), y)) in sents.iter().zip(tags.iter()).zip(labs.iter()).enumerate() {
                let (word_tens, tag_tens) = util.to_tensors(x, w);
                let y_tensor = util.to_label(y);

                let logits = model.forward(word_tens, tag_tens, true);
                total_correct += logits
                    .to_data()
                    .iter()
                    .map(|p: f32| p > 0.5)
                    .zip(y)
                    .map(|(a, b)| if a == *b { 1 } else { 0 })
                    .sum::<usize>();

                let loss = loss_fn.forward(logits, y_tensor, Reduction::Mean);

                let grads = loss.backward();
                let grads = GradientsParams::from_grads(grads, &model);

                model = opt.step(lr, model, grads);

                total_loss += loss.into_scalar().to_f64();
                total_tokens += x.len();

                if i % 1000 == 0 {
                    println!("{i}/{}", sents.len());
                }
            }

            println!(
                "Average loss for epoch: {}",
                total_loss / sents.len() as f64 * 100.
            );

            println!(
                "{}% correct in training dataset",
                total_correct as f32 / total_tokens as f32 * 100.
            );

            let score = util.score_model(&model, test_file);
            println!("{}% correct in test dataset", score * 100.);

            if score < last_score {
                println!("Overfitting detected. Stopping...");
                break;
            }

            last_score = score;
        }

        Self {
            vocab,
            model,
            device,
        }
    }

    fn score_model(&self, model: &NpModel<B>, dataset: &impl AsRef<Path>) -> f32 {
        let ExtractedSentences(sents, tags, labs, _) = Self::extract_sents_from_files(&[dataset]);

        let mut total_tokens = 0;
        let mut total_correct: usize = 0;

        for ((x, w), y) in sents.iter().zip(tags.iter()).zip(labs.iter()) {
            let (word_tens, tag_tens) = self.to_tensors(x, w);

            let logits = model.forward(word_tens, tag_tens, false);
            total_correct += logits
                .to_data()
                .iter()
                .map(|p: f32| p > 0.5)
                .zip(y)
                .map(|(a, b)| if a == *b { 1 } else { 0 })
                .sum::<usize>();

            total_tokens += x.len();
        }

        total_correct as f32 / total_tokens as f32
    }

    fn extract_sents_from_files(files: &[impl AsRef<Path>]) -> ExtractedSentences {
        use super::np_extraction::locate_noun_phrases_in_sent;
        use crate::conllu_utils::iter_sentences_in_conllu;

        let mut vocab: HashMap<String, usize> = HashMap::new();
        vocab.insert("<UNK>".into(), UNK_IDX);

        let mut sents: Vec<Vec<String>> = Vec::new();
        let mut sent_tags: Vec<Vec<Option<UPOS>>> = Vec::new();
        let mut labs: Vec<Vec<bool>> = Vec::new();

        const CONTRACTIONS: &[&str] = &["sn't", "n't", "'ll", "'ve", "'re", "'d", "'m", "'s"];

        for file in files {
            for sent in iter_sentences_in_conllu(file) {
                let spans = locate_noun_phrases_in_sent(&sent);

                let mut original_mask = vec![false; sent.tokens.len()];
                for span in spans {
                    for i in span {
                        original_mask[i] = true;
                    }
                }

                let mut toks: Vec<String> = Vec::new();
                let mut tags: Vec<Option<UPOS>> = Vec::new();
                let mut mask: Vec<bool> = Vec::new();

                for (idx, tok) in sent.tokens.iter().enumerate() {
                    let is_contraction = CONTRACTIONS.contains(&&tok.form[..]);
                    if is_contraction && !toks.is_empty() {
                        let prev_tok = toks.pop().unwrap();
                        let prev_mask = mask.pop().unwrap();
                        toks.push(format!("{prev_tok}{}", tok.form));
                        mask.push(prev_mask || original_mask[idx]);
                    } else {
                        toks.push(tok.form.clone());
                        tags.push(tok.upos.and_then(UPOS::from_conllu));
                        mask.push(original_mask[idx]);
                    }
                }

                for t in &toks {
                    if !vocab.contains_key(t) {
                        let next = vocab.len();
                        vocab.insert(t.clone(), next);
                    }
                }

                sents.push(toks);
                sent_tags.push(tags);
                labs.push(mask);
            }
        }

        ExtractedSentences(sents, sent_tags, labs, vocab)
    }
}

#[cfg(feature = "training")]
pub type BurnChunkerCpu = BurnChunker<burn::backend::Autodiff<NdArray>>;

#[cfg(not(feature = "training"))]
pub type BurnChunkerCpu = BurnChunker<NdArray>;

impl BurnChunkerCpu {
    pub fn load_from_bytes_cpu(
        model_bytes: impl AsRef<[u8]>,
        vocab_bytes: impl AsRef<[u8]>,
        embed_dim: usize,
        dropout: f32,
    ) -> Self {
        Self::load_from_bytes(
            model_bytes,
            vocab_bytes,
            embed_dim,
            dropout,
            NdArrayDevice::Cpu,
        )
    }
}

#[cfg(feature = "training")]
impl BurnChunkerCpu {
    pub fn train_cpu(
        training_files: &[impl AsRef<Path>],
        test_file: &impl AsRef<Path>,
        embed_dim: usize,
        dropout: f32,
        epochs: usize,
        lr: f64,
    ) -> Self {
        BurnChunker::<Autodiff<NdArray>>::train(
            training_files,
            test_file,
            embed_dim,
            dropout,
            epochs,
            lr,
            NdArrayDevice::Cpu,
        )
    }
}

impl<B: Backend> Chunker for BurnChunker<B> {
    fn chunk_sentence(&self, sentence: &[String], tags: &[Option<UPOS>]) -> Vec<bool> {
        // Solves a divide-by-zero error in the linear layer.
        if sentence.is_empty() {
            return Vec::new();
        }

        let (word_tens, tag_tens) = self.to_tensors(sentence, tags);
        let prob = self.model.forward(word_tens, tag_tens, false);
        prob.to_data().iter().map(|p: f32| p > 0.5).collect()
    }
}
