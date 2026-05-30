mod affect_to_effect;
mod effect_to_affect;

use affect_to_effect::AffectToEffect;
use effect_to_affect::EffectToAffect;

use crate::linting::merge_linters::merge_linters;

merge_linters!(
    EffectAffect =>
        EffectToAffect,
        AffectToEffect
    => "Guides writers toward the right choice between `effect` and `affect`, correcting each term when it shows up in the other one's role."
);
