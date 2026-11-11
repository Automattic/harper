//! Reads the Common Pile subsets and builds the parts of a training example that need no parser.

use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use flate2::read::MultiGzDecoder;
use hashbrown::HashMap;
use serde::Deserialize;

use crate::ranker::{CONTEXT_RADIUS, PAD_IDX, SPECIAL_WORDS, UNK_IDX};

/// Subsets that are sampled together, and how many of their words training draws.
pub struct Group {
    pub name: &'static str,
    pub subsets: &'static [&'static str],
    target_words: f64,
    usable_words: f64,
}

pub const GROUPS: [Group; 5] = [
    Group {
        name: "YouTube",
        subsets: &["youtube_filtered"],
        target_words: 128e6,
        usable_words: 3_445e6,
    },
    Group {
        name: "Textbooks",
        subsets: &[
            "pressbooks_filtered",
            "libretexts_filtered",
            "oercommons_filtered",
        ],
        target_words: 64e6,
        usable_words: 125.7e6,
    },
    Group {
        name: "News and blogs",
        subsets: &["news_filtered", "foodista_filtered"],
        target_words: 48e6,
        usable_words: 48.2e6,
    },
    Group {
        name: "US government",
        subsets: &["usgpo_filtered"],
        target_words: 48e6,
        usable_words: 5_003e6,
    },
    Group {
        name: "Hansard from 2000",
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Train,
    Tuning,
    Test,
}

impl Split {
    pub const ALL: [Split; 3] = [Split::Train, Split::Tuning, Split::Test];

    pub fn dir_name(self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::Tuning => "tuning",
            Split::Test => "test",
        }
    }

    fn of(identity: &String, seed: u64) -> Self {
        match stable_hash((seed, "split", identity)) % 100 {
            0 => Split::Tuning,
            1 => Split::Test,
            _ => Split::Train,
        }
    }
}

/// One corpus file, with the share of its documents that training keeps.
pub struct FileTask {
    pub group: u8,
    pub path: PathBuf,
    keep_probability: f64,
    subset: &'static str,
}

/// A document that passed the license, year and sampling filters.
pub struct CorpusDocument<'a> {
    pub identity: String,
    pub split: Split,
    pub text: Cow<'a, str>,
}

pub fn stable_hash(value: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn unit_hash(value: impl Hash) -> f64 {
    stable_hash(value) as f64 / u64::MAX as f64
}

/// A small seeded generator, so one document always yields the same examples.
pub struct SplitMix(u64);

impl SplitMix {
    pub fn new(state: u64) -> Self {
        Self(state)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, bound: usize) -> usize {
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

/// Every `.json.gz` file of every subset, in a stable order.
pub fn file_tasks(corpus: &Path, scale: f64) -> io::Result<Vec<FileTask>> {
    let mut tasks = Vec::new();

    for (group, spec) in GROUPS.iter().enumerate() {
        let keep_probability = (scale * spec.target_words / spec.usable_words).min(1.0);

        for subset in spec.subsets {
            let mut paths: Vec<PathBuf> = fs::read_dir(corpus.join(subset))
                .map_err(|err| io::Error::new(err.kind(), format!("reading {subset}: {err}")))?
                .map(|entry| entry.map(|e| e.path()))
                .collect::<Result<_, _>>()?;
            paths.retain(|path| path.to_string_lossy().ends_with(".json.gz"));
            paths.sort();

            tasks.extend(paths.into_iter().map(|path| FileTask {
                group: group as u8,
                path,
                keep_probability,
                subset,
            }));
        }
    }

    Ok(tasks)
}

fn select<'a>(
    line: &'a str,
    line_no: usize,
    task: &FileTask,
    seed: u64,
) -> io::Result<Option<CorpusDocument<'a>>> {
    let doc: RawDoc = serde_json::from_str(line).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}:{line_no}: {err}", task.path.display()),
        )
    })?;

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

    Ok(Some(CorpusDocument {
        split: Split::of(&identity, seed),
        identity,
        text: doc.text,
    }))
}

/// Visit every document of a file that training may use.
pub fn for_each_document(
    task: &FileTask,
    seed: u64,
    mut visit: impl FnMut(CorpusDocument) -> io::Result<()>,
) -> io::Result<()> {
    let file = File::open(&task.path)
        .map_err(|err| io::Error::new(err.kind(), format!("{}: {err}", task.path.display())))?;
    let mut reader = BufReader::with_capacity(1 << 20, MultiGzDecoder::new(file));
    let mut line = String::new();
    let mut line_no = 0;

    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(());
        }
        line_no += 1;

        if let Some(document) = select(&line, line_no, task, seed)? {
            visit(document)?;
        }
    }
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

/// A typo of `word` that `is_word` rejects, or `None` after a few attempts.
pub fn make_typo(
    word: &[char],
    is_word: impl Fn(&[char]) -> bool,
    random: &mut SplitMix,
) -> Option<Vec<char>> {
    (0..TYPO_ATTEMPTS)
        .map(|_| apply_one_edit(word, random))
        .find(|typo| typo.as_slice() != word && !is_word(typo))
}

/// Whether a word can be the hidden word of an example.
pub fn is_target_shape(word: &[char]) -> bool {
    word.len() >= 3 && word.iter().all(|c| c.is_ascii_lowercase())
}

pub fn word_ids(words: &[String]) -> HashMap<String, u16> {
    words
        .iter()
        .enumerate()
        .map(|(id, word)| (word.clone(), id as u16))
        .collect()
}

pub fn word_id(word: &str, vocab: &HashMap<String, u16>) -> u16 {
    vocab.get(word).copied().unwrap_or(UNK_IDX as u16)
}

/// The ids of the nearest words on each side, padded on the side away from the typo.
pub fn context_ids(
    left: &[String],
    right: &[String],
    vocab: &HashMap<String, u16>,
) -> ([u16; CONTEXT_RADIUS], [u16; CONTEXT_RADIUS]) {
    let mut left_ids = [PAD_IDX as u16; CONTEXT_RADIUS];
    let mut right_ids = [PAD_IDX as u16; CONTEXT_RADIUS];

    for (slot, word) in left_ids.iter_mut().rev().zip(left.iter().rev()) {
        *slot = word_id(word, vocab);
    }
    for (slot, word) in right_ids.iter_mut().zip(right) {
        *slot = word_id(word, vocab);
    }

    (left_ids, right_ids)
}

/// The special words, then the `size` most common words that `is_known` accepts.
pub fn vocabulary(
    counts: impl IntoIterator<Item = HashMap<String, u64>>,
    is_known: impl Fn(&str) -> bool,
    size: usize,
) -> Vec<String> {
    let mut totals: HashMap<String, u64> = HashMap::new();
    for (word, count) in counts.into_iter().flatten() {
        *totals.entry(word).or_insert(0) += count;
    }

    let mut known: Vec<(String, u64)> = totals
        .into_iter()
        .filter(|(word, _)| is_known(word))
        .collect();
    known.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    known.truncate(size);

    SPECIAL_WORDS
        .iter()
        .map(|word| word.to_string())
        .chain(known.into_iter().map(|(word, _)| word))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    use flate2::{Compression, write::GzEncoder};
    use hashbrown::HashMap;

    use super::{
        CONTEXT_RADIUS, FileTask, GROUPS, HANSARD_SUBSET, PAD_IDX, SPECIAL_WORDS, Split, SplitMix,
        UNK_IDX, apply_one_edit, context_ids, file_tasks, for_each_document, is_target_shape,
        license_is_usable, make_typo, vocabulary, word_ids, year_of,
    };

    fn chars(word: &str) -> Vec<char> {
        word.chars().collect()
    }

    fn strings(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    fn write_gz(path: &Path, lines: &[serde_json::Value]) {
        let mut gz = GzEncoder::new(File::create(path).unwrap(), Compression::default());
        for line in lines {
            writeln!(gz, "{line}").unwrap();
        }
        gz.finish().unwrap();
    }

    fn doc(id: &str, license: &str, year: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "text": "A short text.",
            "metadata": { "license": license, "year": year },
        })
    }

    fn kept_identities(path: &Path, subset: &'static str) -> Vec<String> {
        let task = FileTask {
            group: 0,
            path: path.to_path_buf(),
            keep_probability: 1.0,
            subset,
        };
        let mut identities = Vec::new();
        for_each_document(&task, 1, |document| {
            identities.push(document.identity);
            Ok(())
        })
        .unwrap();
        identities
    }

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
    fn documents_with_other_licenses_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("part.json.gz");
        write_gz(
            &path,
            &[
                doc("by", "Creative Commons - Attribution", "2010"),
                doc(
                    "by-sa",
                    "Creative Commons - Attribution Share-Alike",
                    "2010",
                ),
                doc("pd", "Public Domain", "2010"),
                doc("gfdl", "GNU Free Documentation License", "2010"),
            ],
        );

        assert_eq!(
            kept_identities(&path, "news_filtered"),
            ["news_filtered:\"by\"", "news_filtered:\"pd\""]
        );
    }

    #[test]
    fn hansard_before_2000_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("part.json.gz");
        write_gz(
            &path,
            &[
                doc("old", "Open Parliament Licence", "1999-12-31"),
                doc("new", "Open Parliament Licence", "2000-01-01"),
                serde_json::json!({
                    "id": "no-year",
                    "text": "A short text.",
                    "metadata": { "license": "Open Parliament Licence" },
                }),
            ],
        );

        assert_eq!(
            kept_identities(&path, HANSARD_SUBSET),
            ["uk_hansard_filtered:\"new\""]
        );
    }

    #[test]
    fn a_broken_line_names_the_file_and_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("part.json.gz");
        write_gz(
            &path,
            &[
                doc("ok", "Public Domain", "2010"),
                serde_json::json!("oops"),
            ],
        );
        let task = FileTask {
            group: 0,
            path: path.clone(),
            keep_probability: 1.0,
            subset: "news_filtered",
        };

        let error = for_each_document(&task, 1, |_| Ok(())).unwrap_err();

        assert!(error.to_string().contains("part.json.gz:2"));
    }

    #[test]
    fn file_tasks_list_every_subset_in_order() {
        let corpus = tempfile::tempdir().unwrap();
        for subset in GROUPS.iter().flat_map(|group| group.subsets) {
            let dir = corpus.path().join(subset);
            fs::create_dir_all(&dir).unwrap();
            write_gz(&dir.join("b.json.gz"), &[]);
            write_gz(&dir.join("a.json.gz"), &[]);
            fs::write(dir.join("notes.txt"), "skip me").unwrap();
        }

        let tasks = file_tasks(corpus.path(), 1.0).unwrap();
        let subsets: usize = GROUPS.iter().map(|group| group.subsets.len()).sum();

        assert_eq!(tasks.len(), subsets * 2);
        assert!(tasks[0].path.ends_with("youtube_filtered/a.json.gz"));
        assert!(tasks[1].path.ends_with("youtube_filtered/b.json.gz"));
        assert!(tasks.windows(2).all(|pair| pair[0].group <= pair[1].group));
        assert!(
            tasks
                .iter()
                .all(|task| task.keep_probability > 0.0 && task.keep_probability <= 1.0)
        );
    }

    #[test]
    fn a_missing_subset_is_an_error() {
        let corpus = tempfile::tempdir().unwrap();

        let error = file_tasks(corpus.path(), 1.0).err().unwrap();

        assert!(error.to_string().contains("youtube_filtered"));
    }

    #[test]
    fn the_split_depends_only_on_the_document() {
        let identities: Vec<String> = (0..10_000).map(|i| format!("news:{i}")).collect();
        let splits: Vec<Split> = identities.iter().map(|id| Split::of(id, 1)).collect();
        let train = splits
            .iter()
            .filter(|split| **split == Split::Train)
            .count();

        assert_eq!(
            splits,
            identities
                .iter()
                .map(|id| Split::of(id, 1))
                .collect::<Vec<_>>()
        );
        assert!((9_700..9_900).contains(&train));
        assert!(splits.contains(&Split::Tuning));
        assert!(splits.contains(&Split::Test));
    }

    #[test]
    fn one_edit_changes_the_word() {
        let word = chars("contract");
        let mut random = SplitMix::new(7);

        let changed = (0..500)
            .map(|_| apply_one_edit(&word, &mut random))
            .filter(|typo| *typo != word)
            .count();

        assert!(changed > 450);
    }

    #[test]
    fn a_typo_is_never_a_known_word() {
        let word = chars("contract");
        let known = [chars("contract"), chars("contracts")];
        let mut random = SplitMix::new(3);

        for _ in 0..200 {
            if let Some(typo) = make_typo(&word, |w| known.iter().any(|k| k == w), &mut random) {
                assert!(!known.contains(&typo));
            }
        }
    }

    #[test]
    fn no_typo_when_every_edit_is_a_word() {
        let mut random = SplitMix::new(3);

        assert_eq!(make_typo(&chars("contract"), |_| true, &mut random), None);
    }

    #[test]
    fn the_same_seed_gives_the_same_typos() {
        let word = chars("contract");
        let typos = |seed| {
            let mut random = SplitMix::new(seed);
            (0..20)
                .map(|_| make_typo(&word, |_| false, &mut random))
                .collect::<Vec<_>>()
        };

        assert_eq!(typos(11), typos(11));
        assert_ne!(typos(11), typos(12));
    }

    #[test]
    fn target_words_are_lowercase_and_long_enough() {
        assert!(is_target_shape(&chars("five")));
        assert!(!is_target_shape(&chars("to")));
        assert!(!is_target_shape(&chars("Five")));
        assert!(!is_target_shape(&chars("café")));
    }

    #[test]
    fn context_ids_pad_away_from_the_typo() {
        let vocab = word_ids(&strings("<PAD> <UNK> <TARGET> <NUM> one two four"));

        let (left, right) = context_ids(&strings("one two"), &strings("four qzx"), &vocab);

        assert_eq!(left[CONTEXT_RADIUS - 2..], [4, 5]);
        assert!(
            left[..CONTEXT_RADIUS - 2]
                .iter()
                .all(|id| *id == PAD_IDX as u16)
        );
        assert_eq!(right[..2], [6, UNK_IDX as u16]);
        assert!(right[2..].iter().all(|id| *id == PAD_IDX as u16));
    }

    #[test]
    fn context_ids_keep_the_nearest_words() {
        let vocab = word_ids(&strings("<PAD> <UNK> <TARGET> <NUM> one two"));
        let left = strings(&format!("{}two", "one ".repeat(CONTEXT_RADIUS)));

        let (left, right) = context_ids(&left, &left, &vocab);

        assert_eq!(left[CONTEXT_RADIUS - 1], 5);
        assert_eq!(right[0], 4);
    }

    #[test]
    fn the_vocabulary_keeps_the_most_common_known_words() {
        let first = HashMap::from([("the".to_string(), 10), ("cat".to_string(), 3)]);
        let second = HashMap::from([
            ("cat".to_string(), 4),
            ("dog".to_string(), 7),
            ("qzx".to_string(), 99),
            ("ant".to_string(), 7),
        ]);

        let words = vocabulary([first, second], |word| word != "qzx", 3);

        assert_eq!(words[..SPECIAL_WORDS.len()], SPECIAL_WORDS);
        assert_eq!(words[SPECIAL_WORDS.len()..], ["the", "ant", "cat"]);
    }
}
