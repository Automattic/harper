//! Train the English joint char-based tagger+chunker on UD English treebanks
//! and write `harper-brill/finished_joint/{model.mpk,char_vocab.json}`.
//!
//! Training data is fetched on demand over HTTP from the Universal Dependencies
//! GitHub mirrors (individual raw `.conllu` files — no git clone, no manual
//! download) and cached under `target/tmp/ud/`. Delete that directory to force a
//! refresh. The fetch shells out to `curl`.
//!
//!   - UD_English-EWT     en_ewt-ud-{train,dev}   (CC BY-SA 4.0)
//!   - UD_English-GUM     en_gum-ud-train         (CC BY-NC-SA 4.0)
//!   - UD_English-ParTUT  en_partut-ud-train      (CC BY-NC-SA 4.0)
//!   - UD_English-LinES   en_lines-ud-train       (CC BY-NC-SA 4.0)
//!
//! The model trains on all four train splits. EWT dev is the held-out eval
//! split (web prose is closest to what Harper actually lints); the dev/test
//! splits of the other treebanks are left untouched.
//!
//! ```bash
//! cargo run --release -p harper-embedded-dict \
//!   --example regenerate_english_joint --features training
//! ```

use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use std::time::Instant;

use burn::backend::wgpu::RuntimeOptions;
use burn::backend::wgpu::graphics::Metal;
use burn::backend::wgpu::{WgpuDevice, init_setup};
use burn::backend::{Autodiff, wgpu::Wgpu};

use harper_pos_utils::UPOS;
use harper_pos_utils::conllu_utils::extract_records_from_files;
use harper_pos_utils::joint::char_vocab::CharVocab;
use harper_pos_utils::joint::eval::{np_prf1, upos_accuracy};
use harper_pos_utils::joint::inject::build_injection;
use harper_pos_utils::joint::runtime::{
    JointArch, JointRuntime, load_joint_from_bytes, save_joint,
};
use harper_pos_utils::joint::suffix_vocab::SuffixVocab;
use harper_pos_utils::joint::train::{JointTrainConfig, train_joint};

/// Architecture constants — MUST match what harper-brill embeds (`JOINT_ARCH`).
/// Morphology-aware char encoder: multi-width convs + a word-final suffix
/// embedding carry POS morphology, and downstream linters are probability-aware
/// (they read the tagger's top-k), so there is no runtime lexical override.
const ARCH: JointArch = JointArch {
    char_dim: 24,
    conv_channels: 64,
    hidden: 128,
    suffix_k: 3,
    suffix_dim: 32,
    max_word: 20,
};
/// Cap on the suffix vocabulary (most-frequent last-`suffix_k`-char suffixes).
const SUFFIX_VOCAB_CAP: usize = 2_000;

/// GitHub branch the UD treebanks are fetched from.
const UD_BRANCH: &str = "master";

/// A UD English treebank. `dev` is `Some` only for the treebank whose dev split
/// is used as the held-out eval set.
struct Treebank {
    repo: &'static str,
    train: &'static str,
    dev: Option<&'static str>,
}

/// Train on all four train splits; hold out EWT dev for eval.
const TREEBANKS: &[Treebank] = &[
    Treebank {
        repo: "UD_English-EWT",
        train: "en_ewt-ud-train.conllu",
        dev: Some("en_ewt-ud-dev.conllu"),
    },
    Treebank {
        repo: "UD_English-GUM",
        train: "en_gum-ud-train.conllu",
        dev: None,
    },
    Treebank {
        repo: "UD_English-ParTUT",
        train: "en_partut-ud-train.conllu",
        dev: None,
    },
    Treebank {
        repo: "UD_English-LinES",
        train: "en_lines-ud-train.conllu",
        dev: None,
    },
];

/// Fetch `<repo>/<file>` from the UD GitHub mirror into `cache_dir`, unless a
/// non-empty copy is already cached there. Downloads to a `.part` sidecar and
/// renames on success, so an interrupted fetch never leaves a truncated file
/// cached as if it were complete. Returns the local path.
fn ensure_file(cache_dir: &Path, repo: &str, file: &str) -> PathBuf {
    let dest = cache_dir.join(file);
    if dest.metadata().map(|m| m.len() > 0).unwrap_or(false) {
        eprintln!("  cached:      {}", dest.display());
        return dest;
    }
    std::fs::create_dir_all(cache_dir).expect("create UD cache dir");
    let url =
        format!("https://raw.githubusercontent.com/UniversalDependencies/{repo}/{UD_BRANCH}/{file}");
    let part = cache_dir.join(format!("{file}.part"));
    eprintln!("  downloading: {url}");
    let status = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "3",
            "--output",
        ])
        .arg(&part)
        .arg(&url)
        .status()
        .expect("run curl (install curl, or pre-place the files in target/tmp/ud/)");
    assert!(status.success(), "download failed ({status}): {url}");
    std::fs::rename(&part, &dest).expect("rename downloaded UD file into place");
    dest
}

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cache = manifest.join("../target/tmp/ud");
    let out = manifest.join("../harper-brill/finished_joint");

    eprintln!("regenerate_english_joint");
    eprintln!("  ud cache: {}", cache.display());
    eprintln!("  out:      {}", out.display());

    // Fetch (or reuse cached) every treebank's train split, plus the held-out
    // dev split from whichever treebank declares one.
    let mut train_files: Vec<PathBuf> = Vec::new();
    let mut dev_file: Option<PathBuf> = None;
    for tb in TREEBANKS {
        train_files.push(ensure_file(&cache, tb.repo, tb.train));
        if let Some(dev) = tb.dev {
            dev_file = Some(ensure_file(&cache, tb.repo, dev));
        }
    }
    let dev_file = dev_file.expect("a treebank must supply the held-out dev split");

    eprintln!("  train files: {}", train_files.len());
    eprintln!("  dev file:    {}", dev_file.display());

    let (mut sents, mut tags, mut np) = extract_records_from_files(&train_files);

    // Inject many natural, good-length sentences of the error constructions the
    // linters check (noun-noun "Xs Y" head, and an -s verb after a noun subject)
    // — these never appear in the grammatical UD treebanks, so the model mistags
    // them out-of-distribution. Varied vocab teaches the pattern, not the words.
    let (inj_s, inj_t, inj_n) = build_injection();
    let n_inj = inj_s.len();
    sents.extend(inj_s);
    tags.extend(inj_t);
    np.extend(inj_n);

    let (dev_s, dev_t, dev_n) = extract_records_from_files(&[&dev_file]);
    // Build vocabs from the combined corpus so injected words/suffixes are in-vocab.
    let vocab = CharVocab::build(&sents);
    let suffix_vocab = SuffixVocab::build(&sents, ARCH.suffix_k, SUFFIX_VOCAB_CAP);

    eprintln!(
        "  train sents: {} (+{} injected)  dev sents: {}",
        sents.len(),
        n_inj,
        dev_s.len()
    );
    eprintln!(
        "  char vocab:  {}  suffix vocab: {}",
        vocab.len(),
        suffix_vocab.len()
    );

    // Explicit Metal init must precede any tensor operations on macOS.
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
        epochs: 100,
        lr: 0.003,
        chunk_loss_weight: 1.0,
    };

    eprintln!(
        "  arch: char_dim={} conv_channels={} hidden={} suffix_k={} suffix_dim={} max_word={}",
        ARCH.char_dim, ARCH.conv_channels, ARCH.hidden, ARCH.suffix_k, ARCH.suffix_dim, ARCH.max_word
    );
    eprintln!(
        "  training: epochs={} batch={} lr={} chunk_loss_weight={}",
        cfg.epochs, cfg.batch_size, cfg.lr, cfg.chunk_loss_weight
    );

    let t = Instant::now();
    let model = train_joint::<Autodiff<Wgpu>>(
        &sents,
        &tags,
        &np,
        &vocab,
        &suffix_vocab,
        &cfg,
        device,
    );
    eprintln!("trained in {:.1?}", t.elapsed());

    // Persist artifacts (model.mpk + char_vocab.json + suffix_vocab.json).
    save_joint(&model, &vocab, &suffix_vocab, &out, &ARCH);
    eprintln!("wrote artifacts to {}", out.display());

    // Evaluate on dev via the CPU runtime (matches the production inference path).
    let model_bytes = std::fs::read(out.join("model.mpk")).expect("read model.mpk");
    let vocab_json =
        std::fs::read_to_string(out.join("char_vocab.json")).expect("read char_vocab.json");
    let suffix_vocab_json =
        std::fs::read_to_string(out.join("suffix_vocab.json")).expect("read suffix_vocab.json");
    let (cpu_model, cpu_vocab, cpu_suffix_vocab) =
        load_joint_from_bytes(&model_bytes, &vocab_json, &suffix_vocab_json, &ARCH);

    let rt = Rc::new(JointRuntime::new(
        cpu_model,
        cpu_vocab,
        cpu_suffix_vocab,
        ARCH.max_word,
        NonZeroUsize::new(10_000).unwrap(),
    ));

    let pred_tags: Vec<Vec<Option<UPOS>>> = dev_s.iter().map(|s| rt.annotate(s).0).collect();
    let pred_np: Vec<Vec<bool>> = dev_s.iter().map(|s| rt.annotate(s).1).collect();

    let acc = upos_accuracy(&pred_tags, &dev_t);
    let m = np_prf1(&pred_np, &dev_n);
    println!(
        "[dev] UPOS acc = {:.4}  NP prec = {:.4}  NP rec = {:.4}  NP F1 = {:.4}",
        acc, m.precision, m.recall, m.f1
    );
}
