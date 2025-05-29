use serde::{Deserialize, Serialize};

use crate::{UPOS, error_counter::ErrorCounter, patch_criteria::PatchCriteria};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub from: UPOS,
    pub to: UPOS,
    pub criteria: PatchCriteria,
}

impl Patch {
    /// Given a list of tagging errors, generate a collection of candidate patches that _might_ fix
    /// them. Training involves determining which candidates actually work.
    pub fn generate_candidate_patches(error_counter: &ErrorCounter) -> Vec<Patch> {
        let mut candidates = Vec::new();

        for key in error_counter.error_counts.keys() {
            candidates.extend(PatchCriteria::gen_candidates().into_iter().map(|c| Patch {
                from: key.was_tagged,
                to: key.correct_tag,
                criteria: c,
            }));
        }

        candidates
    }
}
