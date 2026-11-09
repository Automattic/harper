//! Turns the Common Pile subsets into context ranker examples.

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use flate2::read::MultiGzDecoder;
use harper_core::language_detection::is_likely_english;
use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::{
    Dictionary, FstDictionary, MAX_RANKED_CANDIDATES, context_word, spelling_candidates,
    surrounding_words,
};
use harper_core::{CharStringExt, Dialect, Token, TokenStringExt};
use harper_pos_utils::{
    CONTEXT_RADIUS, GROUP_NAMES, PAD_IDX, RankerExample, SPECIAL_WORDS, UNK_IDX,
};
use hashbrown::HashMap;
use rayon::prelude::*;
use serde::Deserialize;

struct Group {
    subsets: &'static [&'static str],
    target_words: f64,
    usable_words: f64,
}

const GROUPS: [Group; 5] = [
    Group {
        subsets: &["youtube_filtered"],
        target_words: 128e6,
        usable_words: 3_445e6,
    },
    Group {
        subsets: &[
            "pressbooks_filtered",
            "libretexts_filtered",
            "oercommons_filtered",
        ],
        target_words: 64e6,
        usable_words: 125.7e6,
    },
    Group {
        subsets: &["news_filtered", "foodista_filtered"],
        target_words: 48e6,
        usable_words: 48.2e6,
    },
    Group {
        subsets: &["usgpo_filtered"],
        target_words: 48e6,
        usable_words: 5_003e6,
    },
    Group {
        subsets: &["uk_hansard_filtered"],
        target_words: 32e6,
        usable_words: 866e6,
    },
];

const HANSARD_SUBSET: &str = "uk_hansard_filtered";
const HANSARD_FIRST_YEAR: u32 = 2000;
const FIRST_LETTER_TYPO_RATE: f64 = 0.05;
const TYPO_ATTEMPTS: usize = 8;

#[derive(Deserialize)]
struct RawDoc<'a> {
    #[serde(default)]
    id: serde_json::Value,
    #[serde(borrow)]
    text: Cow<'a, str>,
    metadata: RawMetadata<'a>,
}

#[derive(Deserialize)]
struct RawMetadata<'a> {
    #[serde(borrow, default)]
    license: Cow<'a, str>,
    #[serde(default)]
    year: serde_json::Value,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Split {
    Train,
    Tuning,
    Test,
}

impl Split {
    const ALL: [Split; 3] = [Split::Train, Split::Tuning, Split::Test];

    fn dir_name(self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::Tuning => "tuning",
            Split::Test => "test",
        }
    }
}

struct FileTask {
    group: u8,
    keep_probability: f64,
    subset: &'static str,
    path: PathBuf,
}

/// A document that passed every filter, split into Harper's tokens.
struct Selected {
    identity: String,
    split: Split,
    source: Vec<char>,
    tokens: Vec<Token>,
}

#[derive(Default)]
struct Stats {
    documents: AtomicU64,
    sentences: AtomicU64,
    examples: AtomicU64,
    no_target: AtomicU64,
    no_known_context: AtomicU64,
    no_typo: AtomicU64,
    right_word_missing: AtomicU64,
}

fn stable_hash(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn unit_hash(value: impl Hash) -> f64 {
    stable_hash(value) as f64 / u64::MAX as f64
}

/// A small seeded generator, so one document always yields the same examples.
struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn chance(&mut self, probability: f64) -> bool {
        (self.next() as f64 / u64::MAX as f64) < probability
    }
}

fn license_is_usable(license: &str) -> bool {
    const SHARE_ALIKE: [&str; 5] = ["share-alike", "sharealike", "by-sa", "gnu free", "gfdl"];
    const USABLE: [&str; 5] = [
        "attribution",
        "cc0",
        "zero",
        "public domain",
        "open parliament",
    ];

    let license = license.to_lowercase();
    USABLE.iter().any(|term| license.contains(term))
        && !SHARE_ALIKE.iter().any(|term| license.contains(term))
}

fn year_of(value: &serde_json::Value) -> Option<u32> {
    let text = match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        _ => return None,
    };

    text.get(..4)?.parse().ok()
}

fn select(
    line: &str,
    line_no: usize,
    task: &FileTask,
    dictionary: &impl Dictionary,
    seed: u64,
) -> Result<Option<Selected>> {
    let doc: RawDoc =
        serde_json::from_str(line).with_context(|| format!("{}:{line_no}", task.path.display()))?;

    if !license_is_usable(&doc.metadata.license) {
        return Ok(None);
    }

    if task.subset == HANSARD_SUBSET
        && year_of(&doc.metadata.year).is_none_or(|year| year < HANSARD_FIRST_YEAR)
    {
        return Ok(None);
    }

    let identity = if doc.id.is_null() {
        format!("{}:{line_no}", task.subset)
    } else {
        format!("{}:{}", task.subset, doc.id)
    };

    if unit_hash((seed, "keep", &identity)) >= task.keep_probability {
        return Ok(None);
    }

    let source: Vec<char> = doc.text.chars().collect();
    let tokens = PlainEnglish.parse(&source);

    if !is_likely_english(&tokens, &source, dictionary) {
        return Ok(None);
    }

    let split = match stable_hash((seed, "split", &identity)) % 100 {
        0 => Split::Tuning,
        1 => Split::Test,
        _ => Split::Train,
    };

    Ok(Some(Selected {
        identity,
        split,
        source,
        tokens,
    }))
}

fn for_each_selected(
    task: &FileTask,
    dictionary: &impl Dictionary,
    seed: u64,
    mut visit: impl FnMut(Selected) -> Result<()>,
) -> Result<()> {
    let file = File::open(&task.path).with_context(|| task.path.display().to_string())?;
    let mut reader = BufReader::with_capacity(1 << 20, MultiGzDecoder::new(file));
    let mut line = String::new();
    let mut line_no = 0;

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        line_no += 1;

        if let Some(selected) = select(&line, line_no, task, dictionary, seed)? {
            visit(selected)?;
        }
    }
}

fn count_words(
    task: &FileTask,
    dictionary: &impl Dictionary,
    seed: u64,
) -> Result<HashMap<String, u64>> {
    let mut counts = HashMap::new();

    for_each_selected(task, dictionary, seed, |doc| {
        if doc.split == Split::Train {
            for token in doc.tokens.iter().filter(|t| t.kind.is_word()) {
                if let Some(word) = context_word(token, &doc.source) {
                    *counts.entry(word).or_insert(0) += 1;
                }
            }
        }
        Ok(())
    })?;

    Ok(counts)
}

fn keyboard_neighbors(letter: char) -> &'static [u8] {
    match letter {
        'q' => b"wa",
        'w' => b"qes",
        'e' => b"wrd",
        'r' => b"etf",
        't' => b"ryg",
        'y' => b"tuh",
        'u' => b"yij",
        'i' => b"uok",
        'o' => b"ipl",
        'p' => b"ol",
        'a' => b"qsz",
        's' => b"awdx",
        'd' => b"sefc",
        'f' => b"drgv",
        'g' => b"fthb",
        'h' => b"gyjn",
        'j' => b"hukm",
        'k' => b"jil",
        'l' => b"kop",
        'z' => b"asx",
        'x' => b"zsc",
        'c' => b"xdv",
        'v' => b"cfb",
        'b' => b"vgn",
        'n' => b"bhm",
        'm' => b"njk",
        _ => b"e",
    }
}

fn apply_one_edit(word: &[char], random: &mut SplitMix) -> Vec<char> {
    let position = if random.chance(FIRST_LETTER_TYPO_RATE) {
        0
    } else {
        1 + random.below(word.len() - 1)
    };

    let neighbors = keyboard_neighbors(word[position]);
    let neighbor = neighbors[random.below(neighbors.len())] as char;
    let mut typo = word.to_vec();

    match random.below(100) {
        0..40 => typo[position] = neighbor,
        40..65 => {
            typo.remove(position);
        }
        65..85 => {
            let first = position.min(word.len() - 2);
            typo.swap(first, first + 1);
        }
        _ => typo.insert(position, neighbor),
    }

    typo
}

fn make_typo(
    word: &[char],
    dictionary: &impl Dictionary,
    random: &mut SplitMix,
) -> Option<Vec<char>> {
    (0..TYPO_ATTEMPTS)
        .map(|_| apply_one_edit(word, random))
        .find(|typo| typo.as_slice() != word && !dictionary.contains_word(typo))
}

fn is_target_shape(word: &[char]) -> bool {
    word.len() >= 3 && word.iter().all(|c| c.is_ascii_lowercase())
}

fn word_id(word: &str, vocab: &HashMap<String, u16>) -> u16 {
    vocab.get(word).copied().unwrap_or(UNK_IDX as u16)
}

fn left_ids(words: &[String], vocab: &HashMap<String, u16>) -> [u16; CONTEXT_RADIUS] {
    let mut ids = [PAD_IDX as u16; CONTEXT_RADIUS];
    let start = CONTEXT_RADIUS - words.len();
    for (slot, word) in ids[start..].iter_mut().zip(words) {
        *slot = word_id(word, vocab);
    }
    ids
}

fn right_ids(words: &[String], vocab: &HashMap<String, u16>) -> [u16; CONTEXT_RADIUS] {
    let mut ids = [PAD_IDX as u16; CONTEXT_RADIUS];
    for (slot, word) in ids.iter_mut().zip(words) {
        *slot = word_id(word, vocab);
    }
    ids
}

fn example_from_sentence(
    sentence: &[Token],
    sentence_no: usize,
    doc: &Selected,
    group: u8,
    dictionary: &impl Dictionary,
    vocab: &HashMap<String, u16>,
    seed: u64,
    stats: &Stats,
) -> Option<RankerExample> {
    let eligible: Vec<usize> = sentence
        .iter()
        .enumerate()
        .filter(|(_, token)| token.kind.is_word())
        .filter(|(_, token)| {
            let chars = token.get_ch(&doc.source);
            is_target_shape(chars) && vocab.contains_key(&chars.iter().collect::<String>())
        })
        .map(|(position, _)| position)
        .collect();

    if eligible.is_empty() {
        stats.no_target.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    let mut random = SplitMix(stable_hash((seed, &doc.identity, sentence_no)));
    let position = eligible[random.below(eligible.len())];
    let right_word = sentence[position].get_ch(&doc.source);
    let right_text: String = right_word.iter().collect();

    let (left, right) = surrounding_words(sentence, &doc.source, position, CONTEXT_RADIUS);
    if !left
        .iter()
        .chain(&right)
        .any(|word| vocab.contains_key(word))
    {
        stats.no_known_context.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    let Some(typo) = make_typo(right_word, dictionary, &mut random) else {
        stats.no_typo.fetch_add(1, Ordering::Relaxed);
        return None;
    };

    let candidates: Vec<(String, i32)> =
        spelling_candidates(&typo, dictionary, Dialect::American, MAX_RANKED_CANDIDATES)
            .into_iter()
            .map(|(word, score)| (word.to_lower().iter().collect(), score))
            .collect();

    let Some(target) = candidates.iter().position(|(word, _)| *word == right_text) else {
        stats.right_word_missing.fetch_add(1, Ordering::Relaxed);
        return None;
    };

    let words: Vec<String> = sentence
        .iter()
        .filter_map(|token| context_word(token, &doc.source))
        .collect();

    Some(RankerExample {
        group,
        sentence_hash: stable_hash(&words),
        left: left_ids(&left, vocab),
        right: right_ids(&right, vocab),
        target: target as u8,
        candidates: candidates
            .iter()
            .map(|(word, score)| (word_id(word, vocab), *score as i16))
            .collect(),
    })
}

fn write_examples(
    task: &FileTask,
    output: &Path,
    dictionary: &impl Dictionary,
    vocab: &HashMap<String, u16>,
    seed: u64,
    stats: &Stats,
) -> Result<()> {
    let name = format!(
        "{}.bin",
        task.path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace('.', "-")
    );

    let mut writers = Vec::new();
    for split in Split::ALL {
        let dir = output.join(split.dir_name());
        writers.push((
            split,
            BufWriter::with_capacity(1 << 20, File::create(dir.join(&name))?),
        ));
    }

    for_each_selected(task, dictionary, seed, |doc| {
        stats.documents.fetch_add(1, Ordering::Relaxed);

        let writer = &mut writers
            .iter_mut()
            .find(|(split, _)| *split == doc.split)
            .unwrap()
            .1;

        for (sentence_no, sentence) in doc.tokens.iter_sentences().enumerate() {
            stats.sentences.fetch_add(1, Ordering::Relaxed);

            if let Some(example) = example_from_sentence(
                sentence,
                sentence_no,
                &doc,
                task.group,
                dictionary,
                vocab,
                seed,
                stats,
            ) {
                stats.examples.fetch_add(1, Ordering::Relaxed);
                example.write_to(writer)?;
            }
        }

        Ok(())
    })?;

    for (_, mut writer) in writers {
        writer.flush()?;
    }

    Ok(())
}

fn file_tasks(corpus: &Path, scale: f64) -> Result<Vec<FileTask>> {
    let mut tasks = Vec::new();

    for (group, spec) in GROUPS.iter().enumerate() {
        let keep_probability = (scale * spec.target_words / spec.usable_words).min(1.0);

        for subset in spec.subsets {
            let mut paths: Vec<PathBuf> = fs::read_dir(corpus.join(subset))
                .with_context(|| format!("reading {subset}"))?
                .map(|entry| entry.map(|e| e.path()))
                .collect::<Result<_, _>>()?;
            paths.retain(|path| path.to_string_lossy().ends_with(".json.gz"));
            paths.sort();

            tasks.extend(paths.into_iter().map(|path| FileTask {
                group: group as u8,
                keep_probability,
                subset,
                path,
            }));
        }
    }

    Ok(tasks)
}

fn build_vocabulary(
    tasks: &[FileTask],
    dictionary: &impl Dictionary,
    vocab_size: usize,
    seed: u64,
) -> Result<Vec<String>> {
    let per_file: Vec<HashMap<String, u64>> = tasks
        .par_iter()
        .map(|task| {
            let counts = count_words(task, dictionary, seed)?;
            eprintln!("counted {}", task.path.display());
            Ok(counts)
        })
        .collect::<Result<_>>()?;

    let mut totals: HashMap<String, u64> = HashMap::new();
    for counts in per_file {
        for (word, count) in counts {
            *totals.entry(word).or_insert(0) += count;
        }
    }

    let mut known: Vec<(String, u64)> = totals
        .into_iter()
        .filter(|(word, _)| dictionary.contains_word_str(word))
        .collect();
    known.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    known.truncate(vocab_size);

    Ok(SPECIAL_WORDS
        .iter()
        .map(|word| word.to_string())
        .chain(known.into_iter().map(|(word, _)| word))
        .collect())
}

pub fn prepare(
    corpus: &Path,
    output: &Path,
    scale: f64,
    vocab_size: usize,
    seed: u64,
) -> Result<()> {
    assert_eq!(GROUPS.len(), GROUP_NAMES.len());

    let dictionary = FstDictionary::curated();
    let tasks = file_tasks(corpus, scale)?;

    for split in Split::ALL {
        fs::create_dir_all(output.join(split.dir_name()))?;
    }

    let words = build_vocabulary(&tasks, dictionary.as_ref(), vocab_size, seed)?;
    fs::write(output.join("vocab.json"), serde_json::to_vec(&words)?)?;
    eprintln!("vocabulary has {} words", words.len());

    let vocab: HashMap<String, u16> = words
        .iter()
        .enumerate()
        .map(|(id, word)| (word.clone(), id as u16))
        .collect();

    let stats = Stats::default();
    tasks.par_iter().try_for_each(|task| {
        write_examples(task, output, dictionary.as_ref(), &vocab, seed, &stats)?;
        eprintln!("wrote {}", task.path.display());
        Ok::<(), anyhow::Error>(())
    })?;

    let summary = serde_json::json!({
        "scale": scale,
        "seed": seed,
        "documents": stats.documents.load(Ordering::Relaxed),
        "sentences": stats.sentences.load(Ordering::Relaxed),
        "examples": stats.examples.load(Ordering::Relaxed),
        "no_target": stats.no_target.load(Ordering::Relaxed),
        "no_known_context": stats.no_known_context.load(Ordering::Relaxed),
        "no_typo": stats.no_typo.load(Ordering::Relaxed),
        "right_word_missing": stats.right_word_missing.load(Ordering::Relaxed),
    });
    fs::write(
        output.join("prepare-stats.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{summary}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SplitMix, apply_one_edit, license_is_usable, year_of};

    #[test]
    fn license_filter_keeps_only_usable_licenses() {
        assert!(license_is_usable(
            "Creative Commons - Attribution - https://creativecommons.org/licenses/by/4.0/"
        ));
        assert!(license_is_usable("Public Domain"));
        assert!(license_is_usable("Open Parliament Licence - https://x"));
        assert!(!license_is_usable(
            "Creative Commons - Attribution Share-Alike - https://creativecommons.org/licenses/by-sa/4.0/"
        ));
        assert!(!license_is_usable("GNU Free Documentation License"));
        assert!(!license_is_usable(""));
    }

    #[test]
    fn reads_years_from_strings_and_numbers() {
        assert_eq!(year_of(&serde_json::json!("2016")), Some(2016));
        assert_eq!(year_of(&serde_json::json!("2016-06-15")), Some(2016));
        assert_eq!(year_of(&serde_json::json!(1987)), Some(1987));
        assert_eq!(year_of(&serde_json::Value::Null), None);
    }

    #[test]
    fn one_edit_changes_the_word() {
        let word: Vec<char> = "contract".chars().collect();
        let mut random = SplitMix(7);

        let changed = (0..500)
            .map(|_| apply_one_edit(&word, &mut random))
            .filter(|typo| *typo != word)
            .count();

        assert!(changed > 450);
    }
}
