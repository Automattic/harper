mod general;

use super::effect_affect::EffectAffect;
use crate::linting::merge_linters::merge_linters;
use general::GeneralNounInsteadOfVerb;

merge_linters! {
    NounInsteadOfVerb =>
        GeneralNounInsteadOfVerb,
        EffectAffect
    => "Corrects noun/verb confusions such as `advice/advise` and handles the common `effect/affect` mix-up."
}
