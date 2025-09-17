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

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::EffectAffect;

    // `effect` mistakenly used as the verb `affect`.
    #[test]
    fn corrects_noun_subject_effects_object() {
        assert_suggestion_result(
            "System outages effect our customers.",
            EffectAffect::default(),
            "System outages affect our customers.",
        );
    }

    #[test]
    fn corrects_effects_variant() {
        assert_suggestion_result(
            "This policy effects employee morale.",
            EffectAffect::default(),
            "This policy affects employee morale.",
        );
    }

    #[test]
    fn ignores_effect_change_idiom() {
        assert_lint_count(
            "Leaders work to effect change in their communities.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_noun_phrase() {
        assert_lint_count(
            "The effect your plan had was dramatic.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_as_result_noun() {
        assert_lint_count(
            "The effect was immediate and obvious.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_to_effect_substitutions() {
        assert_lint_count(
            "or it may be desired to effect substitutions",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_followed_by_of_phrase() {
        assert_lint_count(
            "We measured the effect of caffeine on sleep.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_side_effects_usage() {
        assert_lint_count(
            "Side effects may include mild nausea.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_special_effects_phrase() {
        assert_lint_count(
            "She admired the special effects in the film.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_in_cause_and_effect() {
        assert_lint_count(
            "The diagram explains cause and effect relationships.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effects_with_pronoun_subject() {
        assert_lint_count(
            "Those effects were less severe than expected.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn corrects_tariff_effect_import_prices() {
        assert_suggestion_result(
            "The new tariff will effect import prices next quarter.",
            EffectAffect::default(),
            "The new tariff will affect import prices next quarter.",
        );
    }

    #[test]
    fn corrects_droughts_effect_crop_yields() {
        assert_suggestion_result(
            "Prolonged droughts severely effect crop yields across the valley.",
            EffectAffect::default(),
            "Prolonged droughts severely affect crop yields across the valley.",
        );
    }

    #[test]
    fn corrects_caffeine_effect_sleep() {
        assert_suggestion_result(
            "Caffeine can effect your sleep architecture.",
            EffectAffect::default(),
            "Caffeine can affect your sleep architecture.",
        );
    }

    #[test]
    fn corrects_bug_effect_devices() {
        assert_suggestion_result(
            "The firmware bug doesn't effect older devices.",
            EffectAffect::default(),
            "The firmware bug doesn't affect older devices.",
        );
    }

    #[test]
    fn corrects_sarcasm_effect_morale() {
        assert_suggestion_result(
            "Her sarcasm seemed to effect the team's morale.",
            EffectAffect::default(),
            "Her sarcasm seemed to affect the team's morale.",
        );
    }

    #[test]
    fn corrects_outage_effect_timeline() {
        assert_suggestion_result(
            "How will this outage effect our deployment timeline?",
            EffectAffect::default(),
            "How will this outage affect our deployment timeline?",
        );
    }

    #[test]
    fn corrects_temperatures_effect_battery() {
        assert_suggestion_result(
            "Cold temperatures drastically effect lithium-ion battery performance.",
            EffectAffect::default(),
            "Cold temperatures drastically affect lithium-ion battery performance.",
        );
    }

    #[test]
    fn corrects_policy_effect_eligibility() {
        assert_suggestion_result(
            "The policy change could effect your eligibility for benefits.",
            EffectAffect::default(),
            "The policy change could affect your eligibility for benefits.",
        );
    }

    #[test]
    fn corrects_variables_effect_results() {
        assert_suggestion_result(
            "These confounding variables may effect the study's results.",
            EffectAffect::default(),
            "These confounding variables may affect the study's results.",
        );
    }

    #[test]
    fn corrects_fans_effect_concentration() {
        assert_suggestion_result(
            "The noisy HVAC fans constantly effect concentration in the lab.",
            EffectAffect::default(),
            "The noisy HVAC fans constantly affect concentration in the lab.",
        );
    }

    #[test]
    fn corrects_hormones_effect_immunity() {
        assert_suggestion_result(
            "Stress hormones can effect immune response during recovery.",
            EffectAffect::default(),
            "Stress hormones can affect immune response during recovery.",
        );
    }

    #[test]
    fn corrects_pacing_effect_engagement() {
        assert_suggestion_result(
            "The instructor's pacing tended to effect student engagement.",
            EffectAffect::default(),
            "The instructor's pacing tended to affect student engagement.",
        );
    }

    #[test]
    fn corrects_humidity_effect_paint() {
        assert_suggestion_result(
            "Humidity levels directly effect paint curing time.",
            EffectAffect::default(),
            "Humidity levels directly affect paint curing time.",
        );
    }

    #[test]
    fn corrects_exchange_effect_invoice() {
        assert_suggestion_result(
            "The exchange rate will surely effect the final invoice.",
            EffectAffect::default(),
            "The exchange rate will surely affect the final invoice.",
        );
    }

    #[test]
    fn corrects_brightness_effect_contrast() {
        assert_suggestion_result(
            "Screen brightness settings can effect perceived contrast.",
            EffectAffect::default(),
            "Screen brightness settings can affect perceived contrast.",
        );
    }

    #[test]
    fn corrects_medication_effect_him() {
        assert_suggestion_result(
            "The medication didn't effect him the way the doctor expected.",
            EffectAffect::default(),
            "The medication didn't affect him the way the doctor expected.",
        );
    }

    #[test]
    fn corrects_payments_effect_credit() {
        assert_suggestion_result(
            "Late payments will negatively effect your credit score.",
            EffectAffect::default(),
            "Late payments will negatively affect your credit score.",
        );
    }

    #[test]
    fn corrects_wording_effect_interpretation() {
        assert_suggestion_result(
            "Minor wording tweaks shouldn't effect the legal interpretation.",
            EffectAffect::default(),
            "Minor wording tweaks shouldn't affect the legal interpretation.",
        );
    }

    #[test]
    fn corrects_traffic_effect_delivery() {
        assert_suggestion_result(
            "Traffic patterns often effect delivery windows downtown.",
            EffectAffect::default(),
            "Traffic patterns often affect delivery windows downtown.",
        );
    }

    #[test]
    fn corrects_rumor_effect_confidence() {
        assert_suggestion_result(
            "The rumor started to effect investor confidence by noon.",
            EffectAffect::default(),
            "The rumor started to affect investor confidence by noon.",
        );
    }

    #[test]
    fn corrects_allergies_effect_productivity() {
        assert_suggestion_result(
            "Seasonal allergies badly effect her productivity each April.",
            EffectAffect::default(),
            "Seasonal allergies badly affect her productivity each April.",
        );
    }

    #[test]
    fn corrects_feedback_effect_roadmap() {
        assert_suggestion_result(
            "Your feedback won't immediately effect the roadmap.",
            EffectAffect::default(),
            "Your feedback won't immediately affect the roadmap.",
        );
    }

    #[test]
    fn corrects_rules_effect_honeypot() {
        assert_suggestion_result(
            "I cant seem to get my additional rules to effect the honeypot",
            EffectAffect::default(),
            "I cant seem to get my additional rules to affect the honeypot",
        );
    }

    #[test]
    fn corrects_bandwidth_effect_video() {
        assert_suggestion_result(
            "Fluctuating bandwidth can effect video call quality.",
            EffectAffect::default(),
            "Fluctuating bandwidth can affect video call quality.",
        );
    }

    #[test]
    fn corrects_gradient_effect_sensor() {
        assert_suggestion_result(
            "The temperature gradient might effect the sensor's calibration.",
            EffectAffect::default(),
            "The temperature gradient might affect the sensor's calibration.",
        );
    }

    #[test]
    fn corrects_delays_effect_satisfaction() {
        assert_suggestion_result(
            "Even tiny delays can effect user satisfaction metrics.",
            EffectAffect::default(),
            "Even tiny delays can affect user satisfaction metrics.",
        );
    }

    #[test]
    fn corrects_architecture_effect_gps() {
        assert_suggestion_result(
            "The surrounding architecture can effect GPS accuracy.",
            EffectAffect::default(),
            "The surrounding architecture can affect GPS accuracy.",
        );
    }

    #[test]
    fn corrects_lighting_effect_color() {
        assert_suggestion_result(
            "Lighting conditions strongly effect color perception.",
            EffectAffect::default(),
            "Lighting conditions strongly affect color perception.",
        );
    }

    #[test]
    fn corrects_coach_effect_roles() {
        assert_suggestion_result(
            "The new coach's strategy will effect players' roles.",
            EffectAffect::default(),
            "The new coach's strategy will affect players' roles.",
        );
    }

    #[test]
    fn corrects_overtraining_effect_reaction() {
        assert_suggestion_result(
            "Overtraining can effect reaction time and coordination.",
            EffectAffect::default(),
            "Overtraining can affect reaction time and coordination.",
        );
    }

    #[test]
    fn corrects_label_effect_behavior() {
        assert_suggestion_result(
            "The warning label may effect how consumers use the product.",
            EffectAffect::default(),
            "The warning label may affect how consumers use the product.",
        );
    }

    // `affect` mistakenly used as the noun `effect`.
    #[test]
    fn corrects_because_affect_is() {
        assert_suggestion_result(
            "I worry because affect is hidden.",
            EffectAffect::default(),
            "I worry because effect is hidden.",
        );
    }

    #[test]
    fn ignores_psychology_usage() {
        assert_lint_count("The patient's affect is flat.", EffectAffect::default(), 0);
    }

    #[test]
    fn corrects_positive_affect_on() {
        assert_suggestion_result(
            "The new law had a positive affect on small businesses.",
            EffectAffect::default(),
            "The new law had a positive effect on small businesses.",
        );
    }

    #[test]
    fn corrects_great_affect() {
        assert_suggestion_result(
            "badges that they provide to users to allow them to promote their projects to great affect",
            EffectAffect::default(),
            "badges that they provide to users to allow them to promote their projects to great effect",
        );
    }

    #[test]
    fn corrects_affect_of() {
        assert_suggestion_result(
            "We measured the affect of caffeine on reaction time.",
            EffectAffect::default(),
            "We measured the effect of caffeine on reaction time.",
        );
    }

    #[test]
    fn corrects_side_affects() {
        assert_suggestion_result(
            "The side affects included nausea and fatigue.",
            EffectAffect::default(),
            "The side effects included nausea and fatigue.",
        );
    }

    #[test]
    fn corrects_cause_and_affect() {
        assert_suggestion_result(
            "Cause and affect are not the same thing.",
            EffectAffect::default(),
            "Cause and effect are not the same thing.",
        );
    }

    #[test]
    fn corrects_have_an_affect_on() {
        assert_suggestion_result(
            "The change will have an affect on our revenue.",
            EffectAffect::default(),
            "The change will have an effect on our revenue.",
        );
    }

    #[test]
    fn corrects_took_affect() {
        assert_suggestion_result(
            "The medicine took affect within minutes.",
            EffectAffect::default(),
            "The medicine took effect within minutes.",
        );
    }

    #[test]
    fn corrects_come_into_affect() {
        assert_suggestion_result(
            "The policy will come into affect on October 1.",
            EffectAffect::default(),
            "The policy will come into effect on October 1.",
        );
    }

    #[test]
    fn corrects_in_affect_sentence() {
        assert_suggestion_result(
            "The rules are now in affect.",
            EffectAffect::default(),
            "The rules are now in effect.",
        );
    }

    #[test]
    fn corrects_with_immediate_affect() {
        assert_suggestion_result(
            "With immediate affect, the office is closed.",
            EffectAffect::default(),
            "With immediate effect, the office is closed.",
        );
    }

    #[test]
    fn corrects_special_affects() {
        assert_suggestion_result(
            "The director used stunning special affects.",
            EffectAffect::default(),
            "The director used stunning special effects.",
        );
    }

    #[test]
    fn corrects_placebo_affect() {
        assert_suggestion_result(
            "The placebo affect can be powerful.",
            EffectAffect::default(),
            "The placebo effect can be powerful.",
        );
    }

    #[test]
    fn corrects_ripple_affect() {
        assert_suggestion_result(
            "We felt the ripple affect across the entire market.",
            EffectAffect::default(),
            "We felt the ripple effect across the entire market.",
        );
    }

    #[test]
    fn corrects_snowball_affect() {
        assert_suggestion_result(
            "The snowball affect amplified the problem.",
            EffectAffect::default(),
            "The snowball effect amplified the problem.",
        );
    }

    #[test]
    fn corrects_knock_on_affect() {
        assert_suggestion_result(
            "That decision had a knock-on affect throughout the team.",
            EffectAffect::default(),
            "That decision had a knock-on effect throughout the team.",
        );
    }

    #[test]
    fn corrects_greenhouse_affect() {
        assert_suggestion_result(
            "The greenhouse affect warms the planet.",
            EffectAffect::default(),
            "The greenhouse effect warms the planet.",
        );
    }

    #[test]
    fn corrects_little_affect() {
        assert_suggestion_result(
            "Her apology had little affect.",
            EffectAffect::default(),
            "Her apology had little effect.",
        );
    }

    #[test]
    fn corrects_go_into_affect() {
        assert_suggestion_result(
            "The new settings go into affect after a restart.",
            EffectAffect::default(),
            "The new settings go into effect after a restart.",
        );
    }

    #[test]
    fn corrects_put_plan_into_affect() {
        assert_suggestion_result(
            "They put the new plan into affect last week.",
            EffectAffect::default(),
            "They put the new plan into effect last week.",
        );
    }

    #[test]
    fn corrects_contract_into_affect() {
        assert_suggestion_result(
            "The contract comes into affect at midnight.",
            EffectAffect::default(),
            "The contract comes into effect at midnight.",
        );
    }

    #[test]
    fn corrects_no_affect_on_behavior() {
        assert_suggestion_result(
            "The warning had no affect on his behavior.",
            EffectAffect::default(),
            "The warning had no effect on his behavior.",
        );
    }

    #[test]
    fn corrects_opposite_affect() {
        assert_suggestion_result(
            "Inflation had the opposite affect than expected.",
            EffectAffect::default(),
            "Inflation had the opposite effect than expected.",
        );
    }

    #[test]
    fn corrects_remains_in_affect() {
        assert_suggestion_result(
            "The regulation remains in affect until further notice.",
            EffectAffect::default(),
            "The regulation remains in effect until further notice.",
        );
    }

    #[test]
    fn corrects_take_affect_next_week() {
        assert_suggestion_result(
            "The app changes take affect next week.",
            EffectAffect::default(),
            "The app changes take effect next week.",
        );
    }

    #[test]
    fn corrects_sound_affects() {
        assert_suggestion_result(
            "Sound affects were added in post.",
            EffectAffect::default(),
            "Sound effects were added in post.",
        );
    }

    #[test]
    fn does_not_flag_best_affect() {
        assert_lint_count(
            "Using linear regression to predict and understand what factors best affect house price",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_sound_affect() {
        assert_lint_count(
            "The goal of this study was to learn what properties of sound affect human focus the most.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn corrects_sound_affect() {
        assert_suggestion_result(
            "Diesel Generator's animation returns to 'idle' state, but it's sound affect remains in the 'work' state.",
            EffectAffect::default(),
            "Diesel Generator's animation returns to 'idle' state, but it's sound effect remains in the 'work' state.",
        );
    }

    #[test]
    fn does_not_flag_affect_as_verb() {
        assert_lint_count(
            "The change will affect our revenue significantly.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_affects_as_verb() {
        assert_lint_count(
            "This policy directly affects remote workers.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_correct_effect_noun() {
        assert_lint_count(
            "The placebo effect can be powerful.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_sound_effects() {
        assert_lint_count(
            "Sound effects were added in post.",
            EffectAffect::default(),
            0,
        );
    }
}
