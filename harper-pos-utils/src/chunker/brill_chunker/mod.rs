mod patch;

#[cfg(feature = "training")]
use std::path::Path;

#[cfg(feature = "training")]
use crate::word_counter::WordCounter;
use crate::{
    UPOS,
    chunker::{Chunker, upos_freq_dict::UPOSFreqDict},
};

use patch::Patch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrillChunker {
    base: UPOSFreqDict,
    patches: Vec<Patch>,
}

impl BrillChunker {
    pub fn new(base: UPOSFreqDict) -> Self {
        Self {
            base,
            patches: Vec::new(),
        }
    }

    fn apply_patches(&self, sentence: &[String], tags: &[Option<UPOS>], np_states: &mut [bool]) {
        for patch in &self.patches {
            for i in 0..sentence.len() {
                if patch.from == np_states[i]
                    && patch.criteria.fulfils(sentence, tags, np_states, i)
                {
                    np_states[i] = !np_states[i];
                }
            }
        }
    }
}

impl Chunker for BrillChunker {
    fn chunk_sentence(&self, sentence: &[String], tags: &[Option<UPOS>]) -> Vec<bool> {
        let mut initial_pass = self.base.chunk_sentence(sentence, tags);

        self.apply_patches(sentence, tags, &mut initial_pass);

        initial_pass
    }
}

#[cfg(feature = "training")]
type CandidateArgs = (Vec<String>, Vec<Option<UPOS>>, Vec<bool>);

#[cfg(feature = "training")]
impl BrillChunker {
    /// Tag a provided sentence with the tagger, providing the "correct" tags (from a dataset or
    /// other source), returning the number of errors.
    pub fn count_patch_errors(
        &self,
        sentence: &[String],
        tags: &[Option<UPOS>],
        base_flags: &[bool],
        correct_np_flags: &[bool],
    ) -> usize {
        let mut flags = base_flags.to_vec();
        self.apply_patches(sentence, tags, &mut flags);

        let mut loss = 0;
        for (a, b) in flags.into_iter().zip(correct_np_flags) {
            if a != *b {
                loss += 1;
            }
        }

        loss
    }

    /// Tag a provided sentence with the tagger, providing the "correct" tags (from a dataset or
    /// other source), returning the number of errors.
    pub fn count_chunk_errors(
        &self,
        sentence: &[String],
        tags: &[Option<UPOS>],
        correct_np_flags: &[bool],
        relevant_words: &mut WordCounter,
    ) -> usize {
        let flags = self.chunk_sentence(sentence, tags);

        let mut loss = 0;
        for ((a, b), word) in flags.into_iter().zip(correct_np_flags).zip(sentence) {
            if a != *b {
                loss += 1;
                relevant_words.inc(word);
            }
        }

        loss
    }

    /// To speed up training, only try a subset of all possible candidates.
    /// How many to select is given by the `candidate_selection_chance`. A higher chance means a
    /// longer training time.
    fn epoch(&mut self, training_files: &[impl AsRef<Path>], candidate_selection_chance: f32) {
        use crate::conllu_utils::iter_sentences_in_conllu;
        use rs_conllu::Sentence;
        use std::time::Instant;

        assert!((0.0..=1.0).contains(&candidate_selection_chance));

        let mut total_tokens = 0;
        let mut error_counter = 0;

        let sentences: Vec<Sentence> = training_files
            .iter()
            .flat_map(iter_sentences_in_conllu)
            .collect();
        let mut sentences_flagged: Vec<CandidateArgs> = Vec::new();

        for sent in &sentences {
            use hashbrown::HashSet;

            use crate::chunker::np_extraction::locate_noun_phrases_in_sent;

            let mut toks: Vec<String> = Vec::new();
            let mut tags = Vec::new();

            for token in &sent.tokens {
                let form = token.form.clone();
                if let Some(last) = toks.last_mut() {
                    match form.as_str() {
                        "sn't" | "n't" | "'ll" | "'ve" | "'re" | "'d" | "'m" | "'s" => {
                            last.push_str(&form);
                            continue;
                        }
                        _ => {}
                    }
                }
                toks.push(form);
                tags.push(token.upos.and_then(UPOS::from_conllu));
            }

            let actual = locate_noun_phrases_in_sent(sent);
            let actual_flat = actual.into_iter().fold(HashSet::new(), |mut a, b| {
                a.extend(b.into_iter());
                a
            });

            let mut actual_seq = Vec::new();

            for el in actual_flat {
                if el >= actual_seq.len() {
                    actual_seq.resize(el + 1, false);
                }
                actual_seq[el] = true;
            }

            sentences_flagged.push((toks, tags, actual_seq));
        }

        let mut relevant_words = WordCounter::default();

        for (tok_buf, tag_buf, flag_buf) in &sentences_flagged {
            total_tokens += tok_buf.len();
            error_counter += self.count_chunk_errors(
                tok_buf.as_slice(),
                tag_buf,
                flag_buf.as_slice(),
                &mut relevant_words,
            );
        }

        println!("=============");
        println!("Total tokens in training set: {total_tokens}");
        println!("Tokens incorrectly flagged: {error_counter}");
        println!(
            "Error rate: {}%",
            error_counter as f32 / total_tokens as f32 * 100.
        );

        // Before adding any patches, let's get a good base.
        let mut base_flags = Vec::new();
        for (toks, tags, _) in &sentences_flagged {
            base_flags.push(self.chunk_sentence(toks, tags));
        }

        let all_candidates = Patch::generate_candidate_patches(&relevant_words);
        let mut pruned_candidates: Vec<Patch> = rand::seq::IndexedRandom::choose_multiple(
            all_candidates.as_slice(),
            &mut rand::rng(),
            (all_candidates.len() as f32 * candidate_selection_chance) as usize,
        )
        .cloned()
        .collect();

        let start = Instant::now();

        #[cfg(feature = "threaded")]
        rayon::slice::ParallelSliceMut::par_sort_by_cached_key(
            pruned_candidates.as_mut_slice(),
            |candidate: &Patch| {
                self.score_candidate(candidate.clone(), &sentences_flagged, &base_flags)
            },
        );

        #[cfg(not(feature = "threaded"))]
        pruned_candidates.sort_by_cached_key(|candidate| {
            self.score_candidate(candidate.clone(), &sentences_flagged, &base_flags)
        });

        let duration = start.elapsed();
        let seconds = duration.as_secs();
        let millis = duration.subsec_millis();

        println!(
            "It took {} seconds and {} milliseconds to search through {} candidates at {} c/sec.",
            seconds,
            millis,
            pruned_candidates.len(),
            pruned_candidates.len() as f32 / seconds as f32
        );

        if let Some(best) = pruned_candidates.first() {
            self.patches.push(best.clone());
        }
    }

    /// Lower is better
    fn score_candidate(
        &self,
        candidate: Patch,
        sentences_flagged: &[CandidateArgs],
        base_flags: &[Vec<bool>],
    ) -> usize {
        let mut tagger = BrillChunker::new(UPOSFreqDict::default());
        tagger.patches.push(candidate);

        let mut errors = 0;

        for ((toks, tags, flags), base) in sentences_flagged.iter().zip(base_flags.iter()) {
            errors += tagger.count_patch_errors(toks.as_slice(), tags.as_slice(), base, flags);
        }

        errors
    }

    /// Train a brand-new tagger on a `.conllu` dataset, provided via a path.
    /// This does not do _any_ error handling, and should not run in production.
    /// It should be used for training a model that _will_ be used in production.
    pub fn train(
        training_files: &[impl AsRef<Path>],
        epochs: usize,
        candidate_selection_chance: f32,
    ) -> Self {
        let mut freq_dict = UPOSFreqDict::default();

        for file in training_files {
            freq_dict.inc_from_conllu_file(file);
        }

        let mut chunker = Self::new(freq_dict);

        for _ in 0..epochs {
            chunker.epoch(training_files, candidate_selection_chance);
        }

        chunker
    }
}
