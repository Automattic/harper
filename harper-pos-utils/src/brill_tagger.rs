use std::path::Path;

#[cfg(feature = "threaded")]
use rayon::slice::ParallelSliceMut;

use rs_conllu::Sentence;
use serde::{Deserialize, Serialize};

use crate::{
    FreqDict, FreqDictBuilder, UPOS,
    conllu_utils::iter_sentences_in_conllu,
    error_counter::{ErrorCounter, ErrorKind},
    patch::Patch,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrillTagger {
    base: FreqDict,
    patches: Vec<Patch>,
}

impl BrillTagger {
    pub fn new(base: FreqDict) -> Self {
        Self {
            base,
            patches: Vec::new(),
        }
    }

    fn tag_sentence_no_patch(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        let mut tags = Vec::new();

        for word in sentence {
            let tag = self.base.get(word);
            tags.push(tag);
        }

        tags
    }

    /// Tag a sentence using the provided frequency dictionary and current patch set.
    /// If the tagger is unable to determine a POS, it returns [`None`] in that position.
    pub fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        let mut tags = self.tag_sentence_no_patch(sentence);
        self.apply_patches(sentence, &mut tags);

        tags
    }

    fn apply_patches(&self, sentence: &[String], tags: &mut [Option<UPOS>]) {
        for patch in &self.patches {
            for i in 0..sentence.len() {
                let Some(i_tag) = tags.get(i).copied().flatten() else {
                    continue;
                };

                if patch.from == i_tag && patch.criteria.fulfils(sentence, tags, i) {
                    tags[i] = Some(patch.to);
                }
            }
        }
    }

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
            if let Some(tag) = tag {
                if let Some(correct_tag) = correct_tag {
                    if tag != correct_tag {
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
            if let Some(tag) = tag {
                if let Some(correct_tag) = correct_tag {
                    if tag != correct_tag {
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
        }

        errors
    }
}

#[cfg(feature = "training")]
impl BrillTagger {
    fn epoch(&mut self, training_file: impl AsRef<Path>) {
        let mut total_tokens = 0;
        let mut error_counter = ErrorCounter::new();

        let sentences: Vec<Sentence> = iter_sentences_in_conllu(training_file).collect();
        let mut sentences_tagged: Vec<(Vec<String>, Vec<Option<UPOS>>)> = Vec::new();

        for sent in &sentences {
            let mut toks: Vec<String> = Vec::new();
            let mut tags = Vec::new();

            for token in &sent.tokens {
                let form = token.form.clone();
                if let Some(last) = toks.last_mut() {
                    match form.as_str() {
                        "'snt" | "'ll" | "'ve" | "'re" | "'d" | "'m" => {
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
        println!("Total tokens in training set: {}", total_tokens);
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
            base_tags.push(self.tag_sentence_no_patch(toks));
        }

        let mut candidates = Patch::generate_candidate_patches(&error_counter);

        #[cfg(feature = "threaded")]
        candidates.par_sort_by_cached_key(|candidate| {
            self.score_candidate(candidate.clone(), &sentences_tagged, &base_tags)
        });

        #[cfg(not(feature = "threaded"))]
        candidates.sort_by_cached_key(|candidate| {
            self.score_candidate(candidate.clone(), &sentences_tagged, &base_tags)
        });

        if let Some(best) = candidates.first() {
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
        tagger.patches = self.patches.clone();
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
    pub fn train(training_file: impl AsRef<Path>, epochs: usize) -> Self {
        let mut freq_dict_builder = FreqDictBuilder::new();
        freq_dict_builder.inc_from_conllu_file(&training_file);

        let freq_dict = freq_dict_builder.build();

        let mut tagger = Self::new(freq_dict);

        for _ in 0..epochs {
            tagger.epoch(&training_file);
        }

        tagger
    }
}
