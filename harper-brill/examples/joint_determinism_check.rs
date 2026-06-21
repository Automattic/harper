//! Cheap determinism probe for the seeded joint trainer.
//!
//! Builds the training corpus + vocabs ONCE, then runs a short (few-epoch)
//! training TWICE on the real `Autodiff<Wgpu>`/Metal path with identical inputs.
//! Because `train_joint` reseeds the backend (`B::seed`) and uses a seeded
//! shuffle RNG, the two runs should print identical per-epoch `avg_loss` lines.
//! Any drift means GPU float-reduction order is introducing residual
//! non-determinism that seeding alone cannot pin.
//!
//! Reuses the UD treebanks already cached under `target/tmp/ud/` (run
//! `regenerate_english_joint` once first if the cache is empty).
//!
//! ```bash
//! cargo run --release -p harper-embedded-dict \
//!   --example joint_determinism_check --features training
//! ```

use std::path::PathBuf;
use std::time::Instant;

use burn::backend::wgpu::RuntimeOptions;
use burn::backend::wgpu::graphics::Metal;
use burn::backend::wgpu::{WgpuDevice, init_setup};
use burn::backend::{Autodiff, wgpu::Wgpu};

use harper_pos_utils::conllu_utils::extract_records_from_files;
use harper_pos_utils::joint::char_vocab::CharVocab;
use harper_pos_utils::joint::inject::build_injection;
use harper_pos_utils::joint::runtime::JointArch;
use harper_pos_utils::joint::suffix_vocab::SuffixVocab;
use harper_pos_utils::joint::train::{JointTrainConfig, train_joint};

/// Must match `regenerate_english_joint`'s ARCH so the probe reflects production.
const ARCH: JointArch = JointArch {
    char_dim: 24,
    conv_channels: 64,
    hidden: 128,
    suffix_k: 3,
    suffix_dim: 32,
    max_word: 20,
};
const SUFFIX_VOCAB_CAP: usize = 2_000;

/// Short run — just enough epochs to see whether the loss trajectories diverge.
const PROBE_EPOCHS: usize = 3;

const TRAIN_FILES: &[&str] = &[
    "en_ewt-ud-train.conllu",
    "en_gum-ud-train.conllu",
    "en_partut-ud-train.conllu",
    "en_lines-ud-train.conllu",
];

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = manifest.join("../target/tmp/ud");

    let train_files: Vec<PathBuf> = TRAIN_FILES.iter().map(|f| cache.join(f)).collect();
    for f in &train_files {
        assert!(
            f.metadata().map(|m| m.len() > 0).unwrap_or(false),
            "missing cached corpus {} — run regenerate_english_joint once first",
            f.display()
        );
    }

    // Build the corpus + vocabs ONCE so both runs see byte-identical inputs;
    // this isolates the probe to the training loop itself.
    let (mut sents, mut tags, mut np) = extract_records_from_files(&train_files);
    let (inj_s, inj_t, inj_n) = build_injection();
    sents.extend(inj_s);
    tags.extend(inj_t);
    np.extend(inj_n);
    let vocab = CharVocab::build(&sents);
    let suffix_vocab = SuffixVocab::build(&sents, ARCH.suffix_k, SUFFIX_VOCAB_CAP);

    eprintln!(
        "determinism check: {} sents, char vocab {}, suffix vocab {}, {} epochs x 2 runs",
        sents.len(),
        vocab.len(),
        suffix_vocab.len(),
        PROBE_EPOCHS
    );

    let device = WgpuDevice::DefaultDevice;
    init_setup::<Metal>(&device, RuntimeOptions::default());

    let cfg = JointTrainConfig {
        char_dim: ARCH.char_dim,
        conv_channels: ARCH.conv_channels,
        hidden: ARCH.hidden,
        suffix_k: ARCH.suffix_k,
        suffix_dim: ARCH.suffix_dim,
        max_word: ARCH.max_word,
        batch_size: 256,
        epochs: PROBE_EPOCHS,
        lr: 0.003,
        chunk_loss_weight: 1.0,
    };

    for run in 1..=2 {
        println!("=== DETERMINISM CHECK: RUN {run} ===");
        let t = Instant::now();
        let _ = train_joint::<Autodiff<Wgpu>>(
            &sents,
            &tags,
            &np,
            &vocab,
            &suffix_vocab,
            &cfg,
            device.clone(),
        );
        println!("=== RUN {run} done in {:.1?} ===", t.elapsed());
    }
}
