//! Probe the joint tagger's PROBABILITY distribution (not just argmax) on the
//! key word of each resistant lint case. Tells us whether a failing case is
//! "the right tag is a close runner-up" (a probability-aware linter could fix
//! it without touching the model) or "the right tag has negligible mass" (the
//! model genuinely doesn't know — injection or a different signal is needed).
//!
//! Reads the embedded artifacts from harper-brill/finished_joint at runtime.
//!
//! ```bash
//! cargo run --release -p harper-pos-utils --example probe_pos_probs
//! ```

use std::path::PathBuf;

use harper_pos_utils::joint::TAG_CLASSES;
use harper_pos_utils::joint::batch::JointBatch;
use harper_pos_utils::joint::runtime::{JointArch, index_to_upos, load_joint_from_bytes};
use burn_ndarray::NdArrayDevice;

/// Must match harper-brill's embedded JOINT_ARCH.
const ARCH: JointArch = JointArch {
    char_dim: 24,
    conv_channels: 64,
    hidden: 128,
    suffix_k: 3,
    suffix_dim: 32,
    max_word: 20,
};

/// (label, sentence tokens, focus word, wanted POS) — the resistant cases.
const CASES: &[(&str, &[&str], &str, &str)] = &[
    ("all_ready", &["The", "device", "is", "all", "ready", "available", "in", "Korea", "."], "available", "ADJ"),
    ("there_to_their", &["There", "backpacks", "were", "stacked", "neatly", "by", "the", "door", "."], "backpacks", "NOUN"),
    ("sound_affects", &["Sound", "affects", "were", "added", "in", "post", "."], "affects", "NOUN"),
    ("students_books", &["The", "students", "books", "are", "here", "."], "books", "NOUN"),
    ("birds_song", &["We", "heard", "the", "birds", "song", "today", "."], "song", "NOUN"),
    ("bakers_shop", &["I", "visited", "the", "bakers", "shop", "downtown", "."], "shop", "NOUN"),
    ("you_where_right", &["you", "where", "right", "about", "that", "."], "right", "ADJ"),
    ("missing_determiner", &["Please", "provide", "reproducible", "example", "for", "the", "bug", "."], "example", "NOUN"),
    ("there_ladder", &["We", "borrowed", "there", "ladder", "to", "reach", "the", "light", "."], "ladder", "NOUN"),
    ("there_answer", &["There", "answer", "was", "careful", "and", "precise", "."], "answer", "NOUN"),
    ("there_understanding", &["They", "reconstruct", "there", "understanding", "of", "the", "world", "."], "understanding", "NOUN"),
    ("there_server", &["There", "server", "crashed", "during", "the", "release", "."], "server", "NOUN"),
    ("allcaps_available", &["ALL", "READY", "AVAILABLE", "data", "arrived", "yesterday", "."], "AVAILABLE", "ADJ"),
    ("its_common_adj", &["Its", "common", "for", "users", "to", "get", "frustrated", "."], "common", "ADJ"),
    ("its_common_for", &["Its", "common", "for", "users", "to", "get", "frustrated", "."], "for", "SCONJ"),
    ("imperative_check", &["Check", "were", "the", "error", "occurred", "."], "Check", "VERB"),
    ("existential_there", &["There", "were", "many", "people", "here", "today", "."], "There", "VERB"),
    ("locative_there_brood", &["I", "sat", "there", "brooding", "all", "day", "."], "brooding", "NOUN"),
    ("poss_there_backpacks", &["There", "backpacks", "were", "left", "behind", "."], "backpacks", "NOUN"),
    ("mp_live_hedgehogs", &["the", "balls", "were", "live", "hedgehogs", "today", "."], "live", "ADJ"),
    ("mp_famous_beaches", &["The", "city", "is", "famous", "its", "beaches", "."], "its", "PRON"),
    // passing references (for contrast)
    ("effects_VERB_ok", &["This", "policy", "effects", "employee", "morale", "."], "effects", "VERB"),
    ("each_others_ok", &["They", "reviewed", "each", "others", "code", "before", "release", "."], "code", "NOUN"),
];

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|x| x / sum).collect()
}

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest.join("../harper-brill/finished_joint");
    let model_bytes = std::fs::read(dir.join("model.mpk")).expect("read model.mpk");
    let vocab_json = std::fs::read_to_string(dir.join("char_vocab.json")).expect("char_vocab");
    let suffix_json = std::fs::read_to_string(dir.join("suffix_vocab.json")).expect("suffix_vocab");
    let (model, vocab, suffix_vocab) =
        load_joint_from_bytes(&model_bytes, &vocab_json, &suffix_json, &ARCH);
    let device = NdArrayDevice::Cpu;

    println!("POS probability probe (model: harper-brill/finished_joint)\n");
    for (label, toks, focus, want) in CASES {
        let sent: Vec<String> = toks.iter().map(|s| s.to_string()).collect();
        let dummy_tags = vec![None; sent.len()];
        let dummy_np = vec![false; sent.len()];
        let b = JointBatch::build(
            std::slice::from_ref(&sent),
            std::slice::from_ref(&dummy_tags),
            std::slice::from_ref(&dummy_np),
            &vocab,
            &suffix_vocab,
            ARCH.max_word,
        );
        let (tag_logits, _) =
            model.forward(b.char_ids(&device), b.suffix_ids(&device), 1, b.max_sent);
        let data = tag_logits.into_data();
        let flat: Vec<f32> = data.iter::<f32>().collect(); // [max_sent * TAG_CLASSES]

        let i = toks.iter().position(|w| w == focus).unwrap();
        let row = &flat[i * TAG_CLASSES..i * TAG_CLASSES + TAG_CLASSES];
        let probs = softmax(row);

        // rank classes by prob
        let mut ranked: Vec<(usize, f32)> =
            probs.iter().cloned().enumerate().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let top3: Vec<String> = ranked
            .iter()
            .take(3)
            .map(|(c, p)| {
                let name = index_to_upos(*c).map(|u| format!("{u:?}")).unwrap_or("PAD".into());
                format!("{name}={:.2}", p)
            })
            .collect();

        // probability + rank of the WANTED tag
        let want_prob_rank = ranked.iter().enumerate().find_map(|(rank, (c, p))| {
            index_to_upos(*c)
                .filter(|u| format!("{u:?}") == *want)
                .map(|_| (rank + 1, *p))
        });
        let (wrank, wprob) = want_prob_rank.unwrap_or((99, 0.0));
        let argmax_name = index_to_upos(ranked[0].0).map(|u| format!("{u:?}")).unwrap_or("PAD".into());
        let verdict = if &argmax_name == want {
            "OK"
        } else if wrank <= 2 || wprob >= 0.30 {
            "CLOSE" // right tag is a strong runner-up -> probability-aware rule could fix
        } else {
            "FAR" // model genuinely doesn't know
        };

        println!(
            "  [{verdict:5}] {label:18} {focus:10} got={argmax_name:5} want={want:5} \
             want_p={wprob:.2}(rank {wrank})  top3: {}",
            top3.join(" ")
        );
    }
}
