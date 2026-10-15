#[cfg(feature = "training")]
use std::path::Path;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::UPOS;

use super::Chunker;

/// Tracks the number of times any given UPOS is associated with a noun phrase.
/// Used as the baseline for the chunker.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UPOSFreqDict {
    /// The # of times each [`UPOS`] was not part of an NP subtracted from the number of times it
    /// was.
    pub counts: HashMap<UPOS, isize>,
}

impl UPOSFreqDict {
    pub fn is_likely_np_component(&self, upos: &UPOS) -> bool {
        self.counts.get(upos).cloned().unwrap_or_default() > 0
    }
}

impl Chunker for UPOSFreqDict {
    fn chunk_sentence(&self, _sentence: &[String], tags: &[Option<UPOS>]) -> Vec<bool> {
        tags.iter()
            .map(|t| {
                t.as_ref()
                    .map(|t| self.is_likely_np_component(t))
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(feature = "training")]
impl UPOSFreqDict {
    /// Increment the count for a particular lint kind.
    pub fn inc_is_np(&mut self, upos: UPOS, is_np: bool) {
        self.counts
            .entry(upos)
            .and_modify(|counter| *counter += if is_np { 1 } else { -1 })
            .or_insert(1);
    }

    /// Parse a `.conllu` file and use it to train a frequency dictionary.
    /// For error-handling purposes, this function should not be made accessible outside of training.
    pub fn inc_from_conllu_file(&mut self, path: impl AsRef<Path>) {
        use crate::conllu_utils::{
            TrainingRecord, iter_sentences_in_conllu, sentence_to_training_record,
        };

        for sent in iter_sentences_in_conllu(path) {
            // Merged surface tokens' tags + NP mask — multiword-token and
            // contraction aware (see `sentence_to_training_record`).
            let TrainingRecord { tags, np_mask, .. } = sentence_to_training_record(&sent);
            for (tag, is_np) in tags.into_iter().zip(np_mask) {
                if let Some(upos) = tag {
                    self.inc_is_np(upos, is_np);
                }
            }
        }
    }
}
