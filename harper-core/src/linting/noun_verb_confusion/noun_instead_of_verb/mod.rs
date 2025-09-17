mod effect_affect;
mod general;

use crate::linting::merge_linters::merge_linters;
use effect_affect::EffectAffect;
use general::GeneralNounInsteadOfVerb;

merge_linters! {
    NounInsteadOfVerb =>
        GeneralNounInsteadOfVerb,
        EffectAffect
    => "Corrects noun/verb confusions such as `advice/advise` and handles the common `effect/affect` mixup."
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NounInsteadOfVerb;

    #[test]
    fn combines_general_and_effect_affect() {
        assert_suggestion_result(
            "System outages effect our customers.",
            NounInsteadOfVerb::default(),
            "System outages affect our customers.",
        );
    }
}
