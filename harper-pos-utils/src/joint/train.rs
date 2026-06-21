//! Training loop: Burn Learner configuration, dataset wiring, and checkpoint saving.

use crate::UPOS;
use crate::joint::TAG_CLASSES;
use crate::joint::batch::JointBatch;
use crate::joint::char_vocab::CharVocab;
use crate::joint::model::JointModel;
use crate::joint::suffix_vocab::SuffixVocab;
use burn::module::AutodiffModule;
use burn::nn::loss::CrossEntropyLossConfig;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;

/// Fixed seed for weight init and the per-epoch shuffle. Training was previously
/// unseeded (random init + `rand::rng()` shuffle), so identical data converged to
/// different weights — the ~5.5% of mistagged tokens landed on different words
/// each run, flipping the brittle one-word lint tests in and out of the failing
/// set. A fixed seed makes training deterministic, so every change to the failing
/// set is attributable to a *data* change, not run-to-run noise.
const SEED: u64 = 0xC0FFEE_1234;

pub struct JointTrainConfig {
    pub char_dim: usize,
    pub conv_channels: usize,
    pub hidden: usize,
    pub suffix_k: usize,
    pub suffix_dim: usize,
    pub max_word: usize,
    pub batch_size: usize,
    pub epochs: usize,
    pub lr: f64,
    pub chunk_loss_weight: f64,
}

pub fn train_joint<B: AutodiffBackend>(
    sents: &[Vec<String>],
    tags: &[Vec<Option<UPOS>>],
    np: &[Vec<bool>],
    vocab: &CharVocab,
    suffix_vocab: &SuffixVocab,
    cfg: &JointTrainConfig,
    device: B::Device,
) -> JointModel<B::InnerBackend> {
    // Length-bucketed groups (mirror burn_chunker grouping).
    let mut order: Vec<usize> = (0..sents.len()).collect();
    order.sort_by_key(|&i| sents[i].len());

    let groups: Vec<Vec<usize>> = order
        .chunks(cfg.batch_size)
        .map(<[usize]>::to_vec)
        .collect();
    let batches: Vec<JointBatch> = groups
        .iter()
        .map(|g| {
            let s: Vec<_> = g.iter().map(|&i| sents[i].clone()).collect();
            let t: Vec<_> = g.iter().map(|&i| tags[i].clone()).collect();
            let n: Vec<_> = g.iter().map(|&i| np[i].clone()).collect();
            JointBatch::build(&s, &t, &n, vocab, suffix_vocab, cfg.max_word)
        })
        .collect();

    // Seed the backend RNG so weight initialization is reproducible. Must precede
    // model construction, which draws the initial weights.
    B::seed(&device, SEED);
    let mut model = JointModel::<B>::new(
        vocab.len(),
        cfg.char_dim,
        cfg.conv_channels,
        cfg.hidden,
        suffix_vocab.len(),
        cfg.suffix_dim,
        &device,
    );
    // Label smoothing regularizes the over-confident fit that pure CE reaches
    // once the tagger is actually learning (train loss collapsed to ~0.02 while
    // dev plateaued — classic overfitting). Smoothing improves generalization
    // on the ambiguous content classes (ADJ/NOUN/VERB) that drive lint errors.
    let ce = CrossEntropyLossConfig::new()
        .with_pad_tokens(Some(vec![crate::joint::TAG_PAD_CLASS]))
        .with_smoothing(Some(0.1))
        .init::<B>(&device);
    let mut opt = AdamConfig::new().init();

    use burn::tensor::cast::ToElement;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    // Seeded RNG (not `rand::rng()`) so the per-epoch batch-order shuffle is
    // identical across runs — the other half of deterministic training.
    let mut rng = StdRng::seed_from_u64(SEED);
    // Shuffle batch *order* each epoch (the batches themselves stay fixed, so
    // each keeps its length-bucketed minimal padding). A fixed short->long
    // sweep biases Adam; burn_chunker shuffles for the same reason.
    let mut batch_order: Vec<usize> = (0..batches.len()).collect();

    for epoch in 0..cfg.epochs {
        batch_order.shuffle(&mut rng);
        // Cosine lr decay: a constant 0.003 caused late-training loss spikes
        // (Adam bouncing out of the minimum). Decaying toward ~2% of the base
        // lr lets the model settle, so the final epoch is also the best one.
        let progress = epoch as f64 / cfg.epochs.max(1) as f64;
        let lr = cfg.lr * (0.5 * (1.0 + (std::f64::consts::PI * progress).cos())).max(0.02);
        let mut epoch_loss = 0f64;
        for &bi in &batch_order {
            let b = &batches[bi];
            let char_ids = b.char_ids::<B>(&device);
            let suffix_ids = b.suffix_ids::<B>(&device);
            let (tag_logits, chunk_logits) =
                model.forward(char_ids, suffix_ids, b.batch, b.max_sent);

            // Tag CE over flattened tokens; padded slots zeroed by pad_tokens.
            let flat_logits = tag_logits.reshape([b.batch * b.max_sent, TAG_CLASSES]);
            let flat_tags = b.flat_tags::<B>(&device);
            // `CrossEntropyLoss::forward` ALREADY returns a scalar mean: it sums
            // the (pad-masked) per-token CE and divides by `batch * max_sent`.
            // Dividing that mean a second time by `real_tokens` shrank the
            // tagging loss ~1000x, starving the tagger while the chunk MSE
            // dominated the gradient. Rescale the diluted mean (denominator =
            // all positions) back to a true per-real-token mean so the two
            // heads' losses sit on the same scale.
            let positions = (b.batch * b.max_sent) as f32;
            let ce_mean = ce.forward(flat_logits, flat_tags) * (positions / b.real_tokens as f32);

            // Chunk masked MSE (mirror burn_chunker.rs:502-504).
            let np_t = b.np::<B>(&device);
            let mask = b.mask::<B>(&device);
            let diff = chunk_logits - np_t;
            let masked = diff.clone() * diff * mask.clone();
            let chunk_mse = masked.sum() / mask.sum();

            let loss = ce_mean + chunk_mse * (cfg.chunk_loss_weight as f32);
            epoch_loss += loss.clone().into_scalar().to_f64();
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            model = opt.step(lr, model, grads);
        }
        println!(
            "epoch {:>3}/{}  lr={:.5}  avg_loss={:.4}",
            epoch + 1,
            cfg.epochs,
            lr,
            epoch_loss / batch_order.len().max(1) as f64
        );
    }

    model.valid() // map onto B::InnerBackend for inference/saving
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UPOS;
    use crate::joint::char_vocab::CharVocab;
    use crate::joint::suffix_vocab::SuffixVocab;
    use burn::backend::Autodiff;
    use burn_ndarray::{NdArray, NdArrayDevice};

    #[test]
    fn one_epoch_runs_and_returns_inner_model() {
        let sents = vec![
            vec!["the".to_string(), "dog".to_string()],
            vec!["a".to_string(), "cat".to_string()],
        ];
        let tags = vec![
            vec![Some(UPOS::DET), Some(UPOS::NOUN)],
            vec![Some(UPOS::DET), Some(UPOS::NOUN)],
        ];
        let np = vec![vec![true, true], vec![true, true]];
        let char_vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let cfg = JointTrainConfig {
            char_dim: 8,
            conv_channels: 16,
            hidden: 12,
            suffix_k: 3,
            suffix_dim: 4,
            max_word: 6,
            batch_size: 2,
            epochs: 2,
            lr: 0.01,
            chunk_loss_weight: 1.0,
        };
        let model = train_joint::<Autodiff<NdArray>>(
            &sents,
            &tags,
            &np,
            &char_vocab,
            &suffix_vocab,
            &cfg,
            NdArrayDevice::Cpu,
        );
        // Smoke: model usable for a forward pass on the inner (non-autodiff) backend.
        let b = crate::joint::batch::JointBatch::build(
            &sents,
            &tags,
            &np,
            &char_vocab,
            &suffix_vocab,
            cfg.max_word,
        );
        let (tl, cl) = model.forward(
            b.char_ids(&NdArrayDevice::Cpu),
            b.suffix_ids(&NdArrayDevice::Cpu),
            b.batch,
            b.max_sent,
        );
        assert_eq!(tl.dims()[2], crate::joint::TAG_CLASSES);
        assert_eq!(cl.dims(), [b.batch, b.max_sent]);
    }
}
