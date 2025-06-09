use serde::{Deserialize, Serialize};

use crate::patch_criteria::PatchCriteria;
#[cfg(feature = "training")]
use crate::word_counter::WordCounter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub from: bool,
    pub criteria: PatchCriteria,
}

#[cfg(feature = "training")]
impl Patch {
    pub fn generate_candidate_patches(relevant_words: &WordCounter) -> Vec<Self> {
        use crate::UPOS;
        use strum::IntoEnumIterator;

        const TOP_N_WORDS: usize = 50;
        const REL_POS: [isize; 7] = [-3, -2, -1, 0, 1, 2, 3];

        let mut atoms: Vec<(bool, PatchCriteria)> = Vec::new();

        for from in [false, true] {
            for rel in REL_POS {
                for tag in UPOS::iter() {
                    atoms.push((
                        from,
                        PatchCriteria::WordIsTaggedWith {
                            relative: rel,
                            is_tagged: tag,
                        },
                    ));
                }
            }
            for max_rel in 1..=5 {
                for tag in UPOS::iter() {
                    atoms.push((
                        from,
                        PatchCriteria::AnyWordIsTaggedWith {
                            max_relative: max_rel,
                            is_tagged: tag,
                        },
                    ));
                }
            }
            for prev in UPOS::iter() {
                for post in UPOS::iter() {
                    atoms.push((
                        from,
                        PatchCriteria::SandwichTaggedWith {
                            prev_word_tagged: prev,
                            post_word_tagged: post,
                        },
                    ));
                }
            }
            for rel in REL_POS {
                for is_np in [false, true] {
                    atoms.push((
                        from,
                        PatchCriteria::NounPhraseAt {
                            is_np,
                            relative: rel,
                        },
                    ));
                }
            }
        }

        let tag_atom_count = atoms.len();

        let mut word_atoms: Vec<(bool, PatchCriteria)> = Vec::new();
        for from in [false, true] {
            for rel in REL_POS {
                for w in relevant_words.iter_top_n_words(TOP_N_WORDS) {
                    word_atoms.push((
                        from,
                        PatchCriteria::WordIs {
                            relative: rel,
                            word: w.clone(),
                        },
                    ));
                }
            }
        }

        atoms.extend(word_atoms);

        let total_atoms = atoms.len();
        let word_start = tag_atom_count;
        let word_atoms_ct = total_atoms - word_start;
        let combos_ct = word_atoms_ct * total_atoms - word_atoms_ct;
        let mut patches = Vec::with_capacity(total_atoms + combos_ct);

        for (from, crit) in &atoms {
            patches.push(Self {
                from: *from,
                criteria: crit.clone(),
            });
        }

        for i in word_start..total_atoms {
            let (from_i, ref crit_i) = atoms[i];
            for (j, (_from_j, crit_j)) in atoms.iter().enumerate() {
                if i == j {
                    continue;
                }
                patches.push(Self {
                    from: from_i,
                    criteria: PatchCriteria::Combined {
                        a: Box::new(crit_i.clone()),
                        b: Box::new(crit_j.clone()),
                    },
                });
            }
        }

        patches
    }
}
