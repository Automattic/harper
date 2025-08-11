mod patch;

#[cfg(feature = "training")]
use std::path::Path;

use patch::Patch;
use serde::{Deserialize, Serialize};

#[cfg(feature = "training")]
use super::FreqDict;
#[cfg(feature = "training")]
use super::error_counter::{ErrorCounter, ErrorKind};

use crate::{Tagger, UPOS};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrillTagger<B>
where
    B: Tagger,
{
    base: B,
    patches: Vec<Patch>,
}

impl<B> BrillTagger<B>
where
    B: Tagger,
{
    pub fn new(base: B) -> Self {
        Self {
            base,
            patches: Vec::new(),
        }
    }

    fn apply_patches(&self, sentence: &[String], tags: &mut [Option<UPOS>]) {
        for patch in &self.patches {
            for i in 0..sentence.len() {
                let Some(i_tag) = tags.get(i).copied().flatten() else {
                    continue;
                };

                if patch.from == i_tag && patch.criteria.fulfils(sentence, tags, &[], i) {
                    tags[i] = Some(patch.to);
                }
            }
        }
    }
}

impl<B> Tagger for BrillTagger<B>
where
    B: Tagger,
{
    /// Tag a sentence using the provided frequency dictionary and current patch set.
    /// If the tagger is unable to determine a POS, it returns [`None`] in that position.
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        let mut tags = self.base.tag_sentence(sentence);
        self.apply_patches(sentence, &mut tags);

        tags
    }
}

#[cfg(feature = "training")]
impl BrillTagger<FreqDict> {
    /// Tag a provided sentence with patches, providing the "correct" tags (from a dataset or
    /// other source), returning the number of errors.
    pub fn locate_patch_errors(
        &self,
        sentence: &[String],
        correct_tags: &[Option<UPOS>],
        base_tags: &[Option<UPOS>],
        errors: &mut ErrorCounter,
    ) {
        let mut base_tags = base_tags.to_vec();
        self.apply_patches(sentence, &mut base_tags);

        for ((tag, correct_tag), word) in base_tags.iter().zip(correct_tags.iter()).zip(sentence) {
            if let Some(tag) = tag
                && let Some(correct_tag) = correct_tag
                && tag != correct_tag
            {
                errors.inc(
                    ErrorKind {
                        was_tagged: *tag,
                        correct_tag: *correct_tag,
                    },
                    word.as_str(),
                )
            }
        }
    }

    /// Tag a provided sentence with the tagger, providing the "correct" tags (from a dataset or
    /// other source), returning the number of errors.
    pub fn locate_tag_errors(
        &self,
        sentence: &[String],
        correct_tags: &[Option<UPOS>],
    ) -> ErrorCounter {
        let tags = self.tag_sentence(sentence);

        let mut errors = ErrorCounter::new();

        for ((tag, correct_tag), word) in tags.iter().zip(correct_tags.iter()).zip(sentence) {
            if let Some(tag) = tag
                && let Some(correct_tag) = correct_tag
                && tag != correct_tag
            {
                errors.inc(
                    ErrorKind {
                        was_tagged: *tag,
                        correct_tag: *correct_tag,
                    },
                    word.as_str(),
                )
            }
        }

        errors
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
        let mut error_counter = ErrorCounter::new();

        let sentences: Vec<Sentence> = training_files
            .iter()
            .flat_map(iter_sentences_in_conllu)
            .collect();
        let mut sentences_tagged: Vec<(Vec<String>, Vec<Option<UPOS>>)> = Vec::new();

        for sent in &sentences {
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

            sentences_tagged.push((toks, tags));
        }

        for (tok_buf, tag_buf) in &sentences_tagged {
            total_tokens += tok_buf.len();
            error_counter
                .merge_from(self.locate_tag_errors(tok_buf.as_slice(), tag_buf.as_slice()));
        }

        println!("=============");
        println!("Total tokens in training set: {total_tokens}");
        println!(
            "Tokens incorrectly tagged: {}",
            error_counter.total_errors()
        );
        println!(
            "Error rate: {}%",
            error_counter.total_errors() as f32 / total_tokens as f32 * 100.
        );

        // Before adding any patches, let's get a good base.
        let mut base_tags = Vec::new();
        for (toks, _) in &sentences_tagged {
            base_tags.push(self.tag_sentence(toks));
        }

        let all_candidates = Patch::generate_candidate_patches(&error_counter);
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
                self.score_candidate(candidate.clone(), &sentences_tagged, &base_tags)
            },
        );

        #[cfg(not(feature = "threaded"))]
        pruned_candidates.sort_by_cached_key(|candidate| {
            self.score_candidate(candidate.clone(), &sentences_tagged, &base_tags)
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
        sentences_tagged: &[(Vec<String>, Vec<Option<UPOS>>)],
        base_tags: &[Vec<Option<UPOS>>],
    ) -> usize {
        let mut tagger = BrillTagger::new(FreqDict::default());
        tagger.patches.push(candidate);

        let mut candidate_errors = ErrorCounter::new();

        for ((toks, tags), base) in sentences_tagged.iter().zip(base_tags.iter()) {
            tagger.locate_patch_errors(
                toks.as_slice(),
                tags.as_slice(),
                base,
                &mut candidate_errors,
            );
        }

        candidate_errors.total_errors()
    }

    /// Train a brand-new tagger on a `.conllu` dataset, provided via a path.
    /// This does not do _any_ error handling, and should not run in production.
    /// It should be used for training a model that _will_ be used in production.
    pub fn train(
        training_files: &[impl AsRef<Path>],
        epochs: usize,
        candidate_selection_chance: f32,
    ) -> Self {
        use crate::FreqDictBuilder;

        let mut freq_dict_builder = FreqDictBuilder::new();

        for file in training_files {
            freq_dict_builder.inc_from_conllu_file(file);
        }

        let freq_dict = freq_dict_builder.build();

        let mut tagger = Self::new(freq_dict);

        for _ in 0..epochs {
            tagger.epoch(training_files, candidate_selection_chance);
        }

        tagger
    }
}
