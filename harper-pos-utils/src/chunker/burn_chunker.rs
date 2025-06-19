use crate::chunker::np_extraction::locate_noun_phrases_in_sent;
use crate::{UPOS, chunker::Chunker, conllu_utils::iter_sentences_in_conllu};
use burn::backend::Autodiff;
use burn::module::extract_type_name;
use burn::nn::loss::{BinaryCrossEntropyLossConfig, MseLoss, Reduction};
use burn::optim::{GradientsParams, Optimizer};
use burn::record::{FullPrecisionSettings, NamedMpkFileRecorder};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::cast::ToElement;
use burn::tensor::{Float, TensorData};
use burn::{
    module::Module,
    nn::{BiLstmConfig, EmbeddingConfig, LinearConfig},
    optim::AdamConfig,
    tensor::{Int, Tensor, backend::Backend},
};
use burn_ndarray::{NdArray, NdArrayDevice};
use hashbrown::HashMap;
use rand::seq::SliceRandom;
use rs_conllu::Sentence;
use serde::de;
use std::path::Path;

const PAD_IDX: usize = 0;
const UNK_IDX: usize = 1;

#[derive(Module, Debug)]
struct NpModel<B: Backend> {
    embedding: burn::nn::Embedding<B>,
    lstm: burn::nn::BiLstm<B>,
    linear: burn::nn::Linear<B>,
}

impl<B: Backend> NpModel<B> {
    fn new(vocab: usize, embed_dim: usize, hidden: usize, device: &B::Device) -> Self {
        Self {
            embedding: EmbeddingConfig::new(vocab, embed_dim).init(device),
            lstm: BiLstmConfig::new(embed_dim, hidden, false).init(device),
            linear: LinearConfig::new(hidden * 2, 1).init(device),
        }
    }

    fn forward(&self, input: Tensor<B, 2, Int>) -> Tensor<B, 2> {
        let x = self.embedding.forward(input);
        let (x, _) = self.lstm.forward(x, None);
        let x = self.linear.forward(x);
        x.squeeze::<2>(2)
    }
}

pub struct BurnChunker<B: Backend> {
    vocab: HashMap<String, usize>,
    model: NpModel<B>,
    device: B::Device,
}

impl<B: Backend + AutodiffBackend> BurnChunker<B> {
    fn idx(&self, tok: &str) -> usize {
        *self.vocab.get(tok).unwrap_or(&UNK_IDX)
    }

    fn to_tensor(&self, sent: &[String]) -> Tensor<B, 2, Int> {
        let idxs: Vec<_> = sent.iter().map(|t| self.idx(t) as i32).collect();

        Tensor::<B, 1, Int>::from_data(TensorData::from(idxs.as_slice()), &self.device)
            .reshape([1, sent.len()])
    }

    fn to_label(&self, labels: &[bool]) -> Tensor<B, 2> {
        let ys: Vec<_> = labels.iter().map(|b| if *b { 1. } else { 0. }).collect();

        Tensor::<B, 1, _>::from_data(TensorData::from(ys.as_slice()), &self.device)
            .reshape([1, labels.len()])
    }

    pub fn save_to(&self, path: impl AsRef<Path>) {
        let recorder = NamedMpkFileRecorder::<FullPrecisionSettings>::new();
        self.model
            .clone()
            .save_file(path.as_ref(), &recorder)
            .expect("Should be able to save the model");
    }

    pub fn train(
        training_files: &[impl AsRef<Path>],
        test_file: &impl AsRef<Path>,
        embed_dim: usize,
        epochs: usize,
        lr: f64,
        device: B::Device,
    ) -> Self {
        println!("Preparing datasets...");
        let (sents, labs, vocab) = Self::extract_sents_from_files(training_files);

        println!("Preparing model and training config...");

        let hidden = embed_dim;
        let mut model = NpModel::<B>::new(vocab.len(), embed_dim, hidden, &device);
        let opt_config = AdamConfig::new();
        let mut opt = opt_config.init();

        let util = BurnChunker {
            vocab: vocab.clone(),
            model: model.clone(),
            device: device.clone(),
        };

        let loss_fn = MseLoss::new();

        println!("Training...");

        for _ in 0..epochs {
            let mut total_loss = 0.;
            let mut total_tokens = 0;
            let mut total_correct: usize = 0;

            for (i, (x, y)) in sents.iter().zip(labs.iter()).enumerate() {
                let x_tensor = util.to_tensor(x);
                let y_tensor = util.to_label(y);

                let logits = model.forward(x_tensor);
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
        }

        Self {
            vocab,
            model,
            device,
        }
    }

    fn score_model(&self, model: &NpModel<B>, dataset: &impl AsRef<Path>) -> f32 {
        let (sents, labs, _) = Self::extract_sents_from_files(&[dataset]);

        let mut total_tokens = 0;
        let mut total_correct: usize = 0;

        for (x, y) in sents.iter().zip(labs.iter()) {
            let x_tensor = self.to_tensor(x);

            let logits = model.forward(x_tensor);
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

    fn extract_sents_from_files(
        files: &[impl AsRef<Path>],
    ) -> (Vec<Vec<String>>, Vec<Vec<bool>>, HashMap<String, usize>) {
        let mut vocab: HashMap<String, usize> = HashMap::new();
        vocab.insert("<PAD>".into(), PAD_IDX);
        vocab.insert("<UNK>".into(), UNK_IDX);

        let mut sents: Vec<Vec<String>> = Vec::new();
        let mut labs: Vec<Vec<bool>> = Vec::new();

        for file in files {
            for sent in iter_sentences_in_conllu(file) {
                let mut toks: Vec<String> = Vec::new();
                let mut tags = Vec::new();
                for tok in &sent.tokens {
                    toks.push(tok.form.clone());
                    tags.push(tok.upos.and_then(UPOS::from_conllu));
                }
                for t in &toks {
                    if !vocab.contains_key(t) {
                        let next = vocab.len();
                        vocab.insert(t.clone(), next);
                    }
                }
                let spans = locate_noun_phrases_in_sent(&sent);
                let mut mask = vec![false; toks.len()];
                for span in spans {
                    for i in span {
                        mask[i] = true;
                    }
                }
                sents.push(toks);
                labs.push(mask);
            }
        }

        (sents, labs, vocab)
    }
}

impl BurnChunker<Autodiff<NdArray>> {
    pub fn train_cpu(
        training_files: &[impl AsRef<Path>],
        test_file: &impl AsRef<Path>,
        embed_dim: usize,
        epochs: usize,
        lr: f64,
    ) -> Self {
        BurnChunker::<Autodiff<NdArray>>::train(
            training_files,
            test_file,
            embed_dim,
            epochs,
            lr,
            NdArrayDevice::Cpu,
        )
    }
}

impl<B: Backend + AutodiffBackend> Chunker for BurnChunker<B> {
    fn chunk_sentence(&self, sentence: &[String], _tags: &[Option<UPOS>]) -> Vec<bool> {
        let tensor = self.to_tensor(sentence);
        let prob = self.model.forward(tensor);
        prob.to_data().iter().map(|p: f32| p > 0.5).collect()
    }
}
