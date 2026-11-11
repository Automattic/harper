//! Turns the Common Pile subsets into context ranker examples.

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Result, ensure};
use harper_context_ranker::{
    CONTEXT_RADIUS, CorpusDocument, FileTask, RankerExample, Split, SplitMix, context_ids,
    file_tasks, for_each_document, is_target_shape, make_typo, stable_hash, vocabulary, word_id,
    word_ids,
};
use harper_core::language_detection::is_likely_english;
use harper_core::parsers::{Parser, PlainEnglish};
use harper_core::spell::{
    Dictionary, FstDictionary, MAX_RANKED_CANDIDATES, context_word, spelling_candidates,
    surrounding_words,
};
use harper_core::{CharStringExt, Dialect, Token, TokenStringExt};
use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;

/// An English corpus document, split into Harper's tokens.
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

/// What every example of one run shares.
struct Run<'a, D: Dictionary> {
    dictionary: &'a D,
    vocab: &'a HashMap<String, u16>,
    seed: u64,
    stats: &'a Stats,
}

fn parse(document: CorpusDocument, dictionary: &impl Dictionary) -> Option<Selected> {
    let source: Vec<char> = document.text.chars().collect();
    let tokens = PlainEnglish.parse(&source);

    is_likely_english(&tokens, &source, dictionary).then_some(Selected {
        identity: document.identity,
        split: document.split,
        source,
        tokens,
    })
}

fn for_each_selected(
    task: &FileTask,
    dictionary: &impl Dictionary,
    seed: u64,
    mut visit: impl FnMut(Selected) -> io::Result<()>,
) -> io::Result<()> {
    for_each_document(task, seed, |document| {
        parse(document, dictionary).map_or(Ok(()), &mut visit)
    })
}

fn count_words(
    task: &FileTask,
    dictionary: &impl Dictionary,
    seed: u64,
) -> io::Result<HashMap<String, u64>> {
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

fn example_from_sentence(
    sentence: &[Token],
    sentence_no: usize,
    doc: &Selected,
    group: u8,
    run: &Run<impl Dictionary>,
) -> Option<RankerExample> {
    let eligible: Vec<usize> = sentence
        .iter()
        .enumerate()
        .filter(|(_, token)| token.kind.is_word())
        .filter(|(_, token)| {
            let chars = token.get_ch(&doc.source);
            is_target_shape(chars) && run.vocab.contains_key(&chars.iter().collect::<String>())
        })
        .map(|(position, _)| position)
        .collect();

    if eligible.is_empty() {
        run.stats.no_target.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    let mut random = SplitMix::new(stable_hash((run.seed, &doc.identity, sentence_no)));
    let position = eligible[random.below(eligible.len())];
    let right_word = sentence[position].get_ch(&doc.source);
    let right_text: String = right_word.iter().collect();

    let (left, right) = surrounding_words(sentence, &doc.source, position, CONTEXT_RADIUS);
    if !left
        .iter()
        .chain(&right)
        .any(|word| run.vocab.contains_key(word))
    {
        run.stats.no_known_context.fetch_add(1, Ordering::Relaxed);
        return None;
    }

    let Some(typo) = make_typo(
        right_word,
        |typo| run.dictionary.contains_word(typo),
        &mut random,
    ) else {
        run.stats.no_typo.fetch_add(1, Ordering::Relaxed);
        return None;
    };

    let candidates: Vec<(String, i32)> = spelling_candidates(
        &typo,
        run.dictionary,
        Dialect::American,
        MAX_RANKED_CANDIDATES,
    )
    .into_iter()
    .map(|(word, score)| (word.to_lower().iter().collect(), score))
    .collect();

    let Some(target) = candidates.iter().position(|(word, _)| *word == right_text) else {
        run.stats.right_word_missing.fetch_add(1, Ordering::Relaxed);
        return None;
    };

    let words: Vec<String> = sentence
        .iter()
        .filter_map(|token| context_word(token, &doc.source))
        .collect();
    let (left, right) = context_ids(&left, &right, run.vocab);

    Some(RankerExample {
        group,
        sentence_hash: stable_hash(&words),
        left,
        right,
        target: target as u8,
        candidates: candidates
            .iter()
            .map(|(word, score)| (word_id(word, run.vocab), *score as i16))
            .collect(),
    })
}

fn shard_name(task: &FileTask) -> String {
    format!(
        "{}.bin",
        task.path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .replace('.', "-")
    )
}

fn write_examples(task: &FileTask, output: &Path, run: &Run<impl Dictionary>) -> io::Result<()> {
    let name = shard_name(task);

    let mut writers = Vec::new();
    for split in Split::ALL {
        let dir = output.join(split.dir_name());
        writers.push((
            split,
            BufWriter::with_capacity(1 << 20, File::create(dir.join(&name))?),
        ));
    }

    for_each_selected(task, run.dictionary, run.seed, |doc| {
        run.stats.documents.fetch_add(1, Ordering::Relaxed);

        let writer = &mut writers
            .iter_mut()
            .find(|(split, _)| *split == doc.split)
            .unwrap()
            .1;

        for (sentence_no, sentence) in doc.tokens.iter_sentences().enumerate() {
            run.stats.sentences.fetch_add(1, Ordering::Relaxed);

            if let Some(example) =
                example_from_sentence(sentence, sentence_no, &doc, task.group, run)
            {
                run.stats.examples.fetch_add(1, Ordering::Relaxed);
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

fn build_vocabulary(
    tasks: &[FileTask],
    dictionary: &impl Dictionary,
    vocab_size: usize,
    seed: u64,
) -> io::Result<Vec<String>> {
    let per_file: Vec<HashMap<String, u64>> = tasks
        .par_iter()
        .map(|task| {
            let counts = count_words(task, dictionary, seed)?;
            eprintln!("counted {}", task.path.display());
            Ok(counts)
        })
        .collect::<io::Result<_>>()?;

    Ok(vocabulary(
        per_file,
        |word| dictionary.contains_word_str(word),
        vocab_size,
    ))
}

pub fn prepare(
    corpus: &Path,
    output: &Path,
    scale: f64,
    vocab_size: usize,
    seed: u64,
) -> Result<()> {
    let dictionary = FstDictionary::curated();
    let tasks = file_tasks(corpus, scale)?;

    let shards: HashSet<String> = tasks.iter().map(shard_name).collect();
    ensure!(
        shards.len() == tasks.len(),
        "two corpus files would write the same shard"
    );

    for split in Split::ALL {
        fs::create_dir_all(output.join(split.dir_name()))?;
    }

    let words = build_vocabulary(&tasks, dictionary.as_ref(), vocab_size, seed)?;
    fs::write(output.join("vocab.json"), serde_json::to_vec(&words)?)?;
    eprintln!("vocabulary has {} words", words.len());

    let vocab = word_ids(&words);
    let stats = Stats::default();
    let run = Run {
        dictionary: dictionary.as_ref(),
        vocab: &vocab,
        seed,
        stats: &stats,
    };

    tasks.par_iter().try_for_each(|task| {
        write_examples(task, output, &run)?;
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
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    use flate2::{Compression, write::GzEncoder};
    use harper_context_ranker::{GROUPS, SPECIAL_WORDS, Split, read_examples, read_vocabulary};

    use super::prepare;

    const TEXTS: [&str; 3] = [
        "The committee will review the contract before the meeting on Friday. \
         We ate lunch at noon and then walked back to the office together. \
         The teacher wrote three questions on the board for the class.",
        "People all over the world watched the game on television last night. \
         Please save the file to your desktop before you close the window. \
         The car lost a front wheel on the highway near the bridge.",
        "Add salt and pepper to taste, then cook the soup for five minutes. \
         It was raining outside, so the children played games in the house. \
         He told me a funny story about his first job at the bakery.",
    ];

    fn write_corpus(dir: &Path, file_name: impl Fn(&str) -> String) {
        for subset in GROUPS.iter().flat_map(|group| group.subsets) {
            let subset_dir = dir.join(subset);
            fs::create_dir_all(&subset_dir).unwrap();

            let file = File::create(subset_dir.join(file_name(subset))).unwrap();
            let mut gz = GzEncoder::new(file, Compression::default());
            for (i, text) in TEXTS.iter().enumerate() {
                let line = serde_json::json!({
                    "id": format!("{subset}-{i}"),
                    "text": text,
                    "metadata": { "license": "Public Domain", "year": "2010" },
                });
                writeln!(gz, "{line}").unwrap();
            }
            gz.finish().unwrap();
        }
    }

    fn files_in(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file())
            .map(|path| {
                (
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    fs::read(&path).unwrap(),
                )
            })
            .collect();
        files.sort();
        files
    }

    fn all_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        Split::ALL
            .iter()
            .flat_map(|split| files_in(&dir.join(split.dir_name())))
            .chain(files_in(dir))
            .collect()
    }

    fn unique_name(subset: &str) -> String {
        format!("{subset}-0000.json.gz")
    }

    #[test]
    fn prepare_writes_the_same_examples_every_time() {
        let corpus = tempfile::tempdir().unwrap();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        write_corpus(corpus.path(), unique_name);

        prepare(corpus.path(), first.path(), 1e9, 500, 1).unwrap();
        prepare(corpus.path(), second.path(), 1e9, 500, 1).unwrap();

        assert_eq!(all_files(first.path()), all_files(second.path()));
    }

    #[test]
    fn prepared_examples_are_valid() {
        let corpus = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        write_corpus(corpus.path(), unique_name);

        prepare(corpus.path(), output.path(), 1e9, 500, 1).unwrap();

        let words = read_vocabulary(output.path().join("vocab.json"));
        let examples: Vec<_> = Split::ALL
            .iter()
            .flat_map(|split| read_examples(output.path().join(split.dir_name())).unwrap())
            .collect();
        let stats: serde_json::Value =
            serde_json::from_slice(&fs::read(output.path().join("prepare-stats.json")).unwrap())
                .unwrap();

        assert_eq!(words[..SPECIAL_WORDS.len()], SPECIAL_WORDS);
        assert!(words.len() > SPECIAL_WORDS.len());
        assert!(examples.len() > 10);
        assert_eq!(Some(examples.len() as u64), stats["examples"].as_u64());
        for example in &examples {
            let (right_word, _) = example.candidates[example.target as usize];
            assert!((example.group as usize) < GROUPS.len());
            assert!((right_word as usize) < words.len());
            assert!(
                example
                    .left
                    .iter()
                    .chain(&example.right)
                    .all(|id| (*id as usize) < words.len())
            );
        }
    }

    #[test]
    fn files_that_would_share_a_shard_are_an_error() {
        let corpus = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        write_corpus(corpus.path(), |_| "part-0.json.gz".to_string());

        assert!(prepare(corpus.path(), output.path(), 1e9, 500, 1).is_err());
    }

    #[test]
    fn a_missing_corpus_is_an_error() {
        let corpus = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();

        assert!(prepare(corpus.path(), output.path(), 1.0, 500, 1).is_err());
    }
}
