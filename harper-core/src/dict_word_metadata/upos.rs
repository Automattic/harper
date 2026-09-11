use harper_brill::UPOS;
use itertools::Itertools;
use smallvec::SmallVec;

use crate::{
    DictWordMetadata,
    dict_word_metadata::{
        AdverbData, ConjunctionData, DeterminerData,
        adjective::AdjectiveData,
        nominal::{NounData, PronounData},
        verb::VerbData,
    },
};

impl DictWordMetadata {
    /// If there is only one possible interpretation of the metadata, infer its UPOS tag.
    pub fn infer_pos_tag(&self) -> Option<UPOS> {
        // If an explicit POS tag exists, return it immediately.
        if let Some(pos) = self.pos_tag {
            return Some(pos);
        }

        // Collect all possible POS tags from metadata
        let mut candidates = SmallVec::<[UPOS; 14]>::with_capacity(14);

        if self.is_proper_noun() {
            candidates.push(UPOS::PROPN);
        }

        if self.is_pronoun() {
            candidates.push(UPOS::PRON);
        }
        if self.is_noun() {
            candidates.push(UPOS::NOUN);
        }
        if self.is_verb() {
            // Distinguish auxiliary verbs
            if let Some(data) = &self.verb {
                if data.is_auxiliary == Some(true) {
                    candidates.push(UPOS::AUX);
                } else {
                    candidates.push(UPOS::VERB);
                }
            } else {
                candidates.push(UPOS::VERB);
            }
        }
        if self.is_adjective() {
            candidates.push(UPOS::ADJ);
        }
        if self.is_adverb() {
            candidates.push(UPOS::ADV);
        }
        if self.is_conjunction() {
            candidates.push(UPOS::CCONJ);
        }
        if self.is_determiner() {
            candidates.push(UPOS::DET);
        }
        if self.preposition {
            candidates.push(UPOS::ADP);
        }

        // Remove duplicates
        candidates.sort();
        candidates.dedup();

        candidates.into_iter().exactly_one().ok()
    }

    /// Given a UPOS tag, discard any metadata that would disagree with the given POS tag.
    /// For example, if the metadata suggests a word could either be a noun or an adjective, and we
    /// provide a [`UPOS::NOUN`], this function will remove the adjective data.
    ///
    /// Additionally, if the metadata does not currently declare the potential of the word to be
    /// the specific POS, it becomes so. That means if we provide a [`UPOS::ADJ`] to the function
    /// for a metadata whose `Self::adjective = None`, it will become `Some`.
    pub fn enforce_pos_exclusivity(&mut self, pos: &UPOS) {
        use UPOS::*;
        match pos {
            NOUN => {
                if let Some(noun) = self.noun {
                    self.noun = Some(NounData {
                        is_proper: Some(false),
                        ..noun
                    })
                } else {
                    self.noun = Some(NounData {
                        is_proper: Some(false),
                        is_singular: None,
                        is_plural: None,
                        is_countable: None,
                        is_mass: None,
                        is_possessive: None,
                    })
                }

                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            PROPN => {
                if let Some(noun) = self.noun {
                    self.noun = Some(NounData {
                        is_proper: Some(true),
                        ..noun
                    })
                } else {
                    self.noun = Some(NounData {
                        is_proper: Some(true),
                        is_singular: None,
                        is_plural: None,
                        is_countable: None,
                        is_mass: None,
                        is_possessive: None,
                    })
                }

                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            PRON => {
                if self.pronoun.is_none() {
                    self.pronoun = Some(PronounData::default())
                }

                self.noun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            VERB => {
                if let Some(verb) = self.verb {
                    self.verb = Some(VerbData {
                        is_auxiliary: Some(false),
                        ..verb
                    })
                } else {
                    self.verb = Some(VerbData {
                        is_auxiliary: Some(false),
                        ..Default::default()
                    })
                }

                self.noun = None;
                self.pronoun = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            AUX => {
                if let Some(verb) = self.verb {
                    self.verb = Some(VerbData {
                        is_auxiliary: Some(true),
                        ..verb
                    })
                } else {
                    self.verb = Some(VerbData {
                        is_auxiliary: Some(true),
                        ..Default::default()
                    })
                }

                self.noun = None;
                self.pronoun = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            ADJ => {
                if self.adjective.is_none() {
                    self.adjective = Some(AdjectiveData::default())
                }

                self.noun = None;
                self.pronoun = None;
                self.verb = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            ADV => {
                if self.adverb.is_none() {
                    self.adverb = Some(AdverbData::default())
                }

                self.noun = None;
                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            ADP => {
                self.noun = None;
                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = true;
            }
            DET => {
                self.noun = None;
                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.conjunction = None;
                self.affix = None;
                self.preposition = false;
                self.determiner = Some(DeterminerData::default());
            }
            CCONJ | SCONJ => {
                if self.conjunction.is_none() {
                    self.conjunction = Some(ConjunctionData::default())
                }

                self.noun = None;
                self.pronoun = None;
                self.verb = None;
                self.adjective = None;
                self.adverb = None;
                self.determiner = None;
                self.affix = None;
                self.preposition = false;
            }
            _ => {}
        }
    }
}
