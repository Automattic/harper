//! CPU-only eval of the committed joint model on the EWT dev set.
//! Reports overall UPOS accuracy and a per-gold-tag accuracy breakdown.
//! Parses CoNLL-U inline (no training feature).

use std::num::NonZeroUsize;
use std::path::PathBuf;

use harper_pos_utils::Annotator;
use harper_pos_utils::joint::runtime::{JointArch, JointRuntime, load_joint_from_bytes};

const ARCH: JointArch = JointArch {
    char_dim: 24,
    conv_channels: 64,
    hidden: 128,
    suffix_k: 3,
    suffix_dim: 32,
    max_word: 20,
};

/// Parse CoNLL-U into sentences of (form, upos_string). Skips comments,
/// multiword-token ranges (`1-2`), and empty nodes (`1.1`).
fn parse_conllu(text: &str) -> Vec<Vec<(String, String)>> {
    let mut sents = Vec::new();
    let mut cur: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            if !cur.is_empty() {
                sents.push(std::mem::take(&mut cur));
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        let id = cols[0];
        if id.contains('-') || id.contains('.') {
            continue;
        }
        cur.push((cols[1].to_string(), cols[3].to_string()));
    }
    if !cur.is_empty() {
        sents.push(cur);
    }
    sents
}

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fin = manifest.join("finished_joint");
    // UD treebanks are fetched by `regenerate_english_joint` into target/tmp/ud/.
    let ud = manifest.join("../target/tmp/ud");
    let dev = if std::env::var_os("EVAL_TRAIN").is_some() {
        ud.join("en_ewt-ud-train.conllu")
    } else {
        ud.join("en_ewt-ud-dev.conllu")
    };

    let model_bytes = std::fs::read(fin.join("model.mpk")).expect("model.mpk");
    let vocab_json = std::fs::read_to_string(fin.join("char_vocab.json")).expect("char_vocab");
    let suffix_vocab_json =
        std::fs::read_to_string(fin.join("suffix_vocab.json")).expect("suffix_vocab");
    let (model, cvocab, svocab) =
        load_joint_from_bytes(&model_bytes, &vocab_json, &suffix_vocab_json, &ARCH);

    let text = std::fs::read_to_string(&dev).expect("dev conllu");
    let sents = parse_conllu(&text);
    eprintln!("dev sents: {}", sents.len());

    let rt = JointRuntime::new(
        model,
        cvocab,
        svocab,
        ARCH.max_word,
        NonZeroUsize::new(30_000).unwrap(),
    );

    let mut total = 0usize;
    let mut correct = 0usize;
    let mut per_tag: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();

    for sent in &sents {
        let toks: Vec<String> = sent.iter().map(|(f, _)| f.clone()).collect();
        // Most-likely tag per token = first of each plausible-tag set.
        let (sets, _np) = rt.annotate(&toks);
        let pred: Vec<Option<_>> = sets.iter().map(|s| s.first().copied()).collect();
        for wi in 0..toks.len() {
            let gold = &sent[wi].1;
            // skip tags our 16-variant enum can't represent (e.g. "X")
            if gold == "X" || gold == "_" {
                continue;
            }
            let p = pred[wi]
                .map(|u| format!("{u:?}"))
                .unwrap_or_else(|| "None".into());
            total += 1;
            let e = per_tag.entry(gold.clone()).or_insert((0, 0));
            e.1 += 1;
            let hit = &p == gold;
            if hit {
                correct += 1;
                e.0 += 1;
            }
        }
    }

    println!("==================================================");
    println!(
        "[dev] UPOS acc    = {:.4}  ({}/{})",
        correct as f64 / total as f64,
        correct,
        total
    );
    println!("--- per-gold-tag accuracy (worst first) ---");
    let mut rows: Vec<_> = per_tag
        .iter()
        .map(|(u, &(c, t))| (u.clone(), c, t))
        .collect();
    rows.sort_by(|a, b| {
        (a.1 as f64 / a.2 as f64)
            .partial_cmp(&(b.1 as f64 / b.2 as f64))
            .unwrap()
    });
    for (u, c, t) in rows {
        println!("  {:<6} {:.3}  ({}/{})", u, c as f64 / t as f64, c, t);
    }
}
