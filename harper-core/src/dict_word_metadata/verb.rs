use serde::{Deserialize, Serialize};

// These verb forms are morphological variations, distinct from TAM (Tense-Aspect-Mood)
// Each form can be used in various TAM combinations:
// - Lemma form (infinitive, citation form, dictionary form)
//   Used in infinitives (e.g., "to sleep"), imperatives (e.g., "sleep!"), and with modals (e.g., "will sleep")
// - Past form (past participle and simple past)
//   Used as verbs (e.g., "slept") or adjectives (e.g., "closed door")
// - Progressive form (present participle and gerund)
//   Used as verbs (e.g., "sleeping"), nouns (e.g., "sleeping is important"), or adjectives (e.g., "sleeping dog")
// - Third person singular present (-s/-es)
//   Used for third person singular subjects (e.g., "he sleeps", "she reads")
//
// Important notes:
// 1. English expresses time through auxiliary verbs, not verb form alone
// 2. Irregular verbs can have different forms for past participle and simple past
// 3. Future is always expressed through auxiliary verbs (e.g., "will sleep", "going to sleep")
#[repr(u32)]
pub enum VerbForm {
    /// The uninflected verb form: "walk", "eat"
    LemmaForm = 1 << 0,
    /// The past form for regular verbs: "walked"
    PastForm = 1 << 1,
    /// The simple past/preterite form for irregular verbs: "ate"
    SimplePastForm = 1 << 2,
    /// The past participle form for irregular verbs: "eaten"
    PastParticipleForm = 1 << 3,
    /// The progressive/continuous/gerund/present participle form: "walking", "eating"
    ProgressiveForm = 1 << 4,
    /// The third person singular present form: "walks", "eats"
    ThirdPersonSingularPresentForm = 1 << 5,
}

/// The underlying type used for verb form flags.
pub type VerbFormFlagsUnderlyingType = u32;

bitflags::bitflags! {
    /// A collection of bit flags used to represent verb forms.
    ///
    /// This allows a word to be tagged with multiple verb forms when applicable.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
    #[serde(transparent)]
    pub struct VerbFormFlags: VerbFormFlagsUnderlyingType {
        const LEMMA = VerbForm::LemmaForm as VerbFormFlagsUnderlyingType;
        const PAST = VerbForm::PastForm as VerbFormFlagsUnderlyingType;
        const PRETERITE = VerbForm::SimplePastForm as VerbFormFlagsUnderlyingType;
        const PAST_PARTICIPLE = VerbForm::PastParticipleForm as VerbFormFlagsUnderlyingType;
        const PROGRESSIVE = VerbForm::ProgressiveForm as VerbFormFlagsUnderlyingType;
        const THIRD_PERSON_SINGULAR = VerbForm::ThirdPersonSingularPresentForm as VerbFormFlagsUnderlyingType;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
pub struct VerbData {
    pub is_linking: Option<bool>,
    pub is_auxiliary: Option<bool>,
    #[serde(rename = "verb_form", default)]
    pub verb_forms: Option<VerbFormFlags>,
}

impl VerbData {
    /// Produce a copy of `self` with the known properties of `other` set.
    pub fn or(&self, other: &Self) -> Self {
        let verb_forms = match (self.verb_forms, other.verb_forms) {
            (Some(self_verb_forms), Some(other_verb_forms)) => {
                Some(self_verb_forms | other_verb_forms)
            }
            (Some(self_verb_forms), None) => Some(self_verb_forms),
            (None, Some(other_verb_forms)) => Some(other_verb_forms),
            (None, None) => None,
        };

        Self {
            is_linking: self.is_linking.or(other.is_linking),
            is_auxiliary: self.is_auxiliary.or(other.is_auxiliary),
            verb_forms,
        }
    }
}
