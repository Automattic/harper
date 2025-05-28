use serde::{Deserialize, Serialize};

use crate::{
    FreqDict, UPOS,
    error_counter::{ErrorCounter, ErrorKind},
    patch_criteria::PatchCriteria,
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
    ) -> ErrorCounter {
        let mut base_tags = base_tags.to_vec();
        self.apply_patches(sentence, &mut base_tags);

        let mut errors = ErrorCounter::new();

        for (tag, correct_tag) in base_tags.iter().zip(correct_tags.iter()) {
            if let Some(tag) = tag {
                if let Some(correct_tag) = correct_tag {
                    if tag != correct_tag {
                        errors.inc(ErrorKind {
                            was_tagged: *tag,
                            correct_tag: *correct_tag,
                        })
                    }
                }
            }
        }

        errors
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

        for (tag, correct_tag) in tags.iter().zip(correct_tags.iter()) {
            if let Some(tag) = tag {
                if let Some(correct_tag) = correct_tag {
                    if tag != correct_tag {
                        errors.inc(ErrorKind {
                            was_tagged: *tag,
                            correct_tag: *correct_tag,
                        })
                    }
                }
            }
        }

        errors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Patch {
    from: UPOS,
    to: UPOS,
    criteria: PatchCriteria,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rayon::slice::ParallelSliceMut;
    use rs_conllu::Sentence;
    use serde_json::to_string_pretty;
    use strum::IntoEnumIterator;

    use crate::{
        FreqDictBuilder, UPOS, brill_tagger::BrillTagger, conllu_utils::iter_sentences_in_conllu,
        error_counter::ErrorCounter, patch_criteria::PatchCriteria,
    };

    use super::Patch;

    /// Lower is better
    fn score_candidate(
        tagger: &BrillTagger,
        candidate: Patch,
        sentences_tagged: &[(Vec<String>, Vec<Option<UPOS>>)],
        base_tags: &[Vec<Option<UPOS>>],
    ) -> usize {
        let mut tagger = tagger.clone();
        tagger.patches.push(candidate);

        let mut candidate_errors = ErrorCounter::new();

        for ((toks, tags), base) in sentences_tagged.iter().zip(base_tags.iter()) {
            candidate_errors.merge_from(tagger.locate_patch_errors(
                toks.as_slice(),
                tags.as_slice(),
                base,
            ));
        }

        candidate_errors.total_errors()
    }

    fn epoch(tagger: &mut BrillTagger, training_file: &str) {
        let mut total_tokens = 0;
        let mut error_counter = ErrorCounter::new();

        let sentences: Vec<Sentence> = iter_sentences_in_conllu(training_file).collect();
        let mut sentences_tagged: Vec<(Vec<String>, Vec<Option<UPOS>>)> = Vec::new();

        for sent in &sentences {
            let mut toks = Vec::new();
            let mut tags = Vec::new();

            for token in &sent.tokens {
                toks.push(token.form.clone());
                tags.push(token.upos.and_then(UPOS::from_conllu));
            }

            sentences_tagged.push((toks, tags));
        }

        for (tok_buf, tag_buf) in &sentences_tagged {
            total_tokens += tok_buf.len();
            error_counter
                .merge_from(tagger.locate_tag_errors(tok_buf.as_slice(), tag_buf.as_slice()));
        }

        dbg!(total_tokens);
        dbg!(error_counter.total_errors());
        dbg!(error_counter.total_errors() as f32 / total_tokens as f32 * 100.);

        // Before adding any patches, let's get a good base.
        let mut base_tags = Vec::new();
        for (toks, _) in &sentences_tagged {
            base_tags.push(tagger.tag_sentence_no_patch(toks));
        }

        let mut candidates = generate_candidate_patches(&error_counter);

        candidates.par_sort_by_cached_key(|candidate| {
            score_candidate(tagger, candidate.clone(), &sentences_tagged, &base_tags)
        });

        if let Some(best) = candidates.first() {
            tagger.patches.push(best.clone());
        }
    }

    fn candidate_criterion() -> Vec<PatchCriteria> {
        let mut criteria = Vec::new();
        for upos in UPOS::iter() {
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -1,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -2,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::WordIsTaggedWith {
                relative: -3,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -2,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -3,
                is_tagged: upos,
            });
            criteria.push(PatchCriteria::AnyWordIsTaggedWith {
                max_relative: -4,
                is_tagged: upos,
            });

            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 0,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 0,
                is_capitalized: false,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: -1,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: -1,
                is_capitalized: false,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 1,
                is_capitalized: true,
            });
            criteria.push(PatchCriteria::RelativeWordCaps {
                relative: 1,
                is_capitalized: false,
            });

            for upos_b in UPOS::iter() {
                criteria.push(PatchCriteria::SandwichTaggedWith {
                    prev_word_tagged: upos,
                    post_word_tagged: upos_b,
                });

                criteria.push(PatchCriteria::Combined {
                    a: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: 1,
                        is_tagged: upos,
                    }),
                    b: Box::new(PatchCriteria::WordIsTaggedWith {
                        relative: -2,
                        is_tagged: upos_b,
                    }),
                })
            }
        }

        criteria
    }

    fn generate_candidate_patches(error_counter: &ErrorCounter) -> Vec<Patch> {
        let mut candidates = Vec::new();

        for key in error_counter.error_counts.keys() {
            candidates.extend(candidate_criterion().into_iter().map(|c| Patch {
                from: key.was_tagged,
                to: key.correct_tag,
                criteria: c,
            }));
        }

        candidates
    }

    #[test]
    fn training() {
        let training_file = "./en_gum-ud-train.conllu";

        let mut freq_dict_builder = FreqDictBuilder::new();
        freq_dict_builder.inc_from_conllu_file(training_file);

        let freq_dict = freq_dict_builder.build();

        let mut tagger = BrillTagger::new(freq_dict);

        for i in 0..1000 {
            epoch(&mut tagger, training_file);
            let dest_json = format!("epoch{}.json", i);
            fs::write(dest_json, to_string_pretty(&tagger).unwrap()).unwrap();
        }

        panic!();
    }
}
