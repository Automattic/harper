use super::merge_linters::merge_linters;

mod effect_affect;
mod noun_instead_of_verb;
mod verb_instead_of_noun;

// Common noun-verb pairs that are often confused
// See also [`NounInsteadOfVerb``]
pub(crate) const NOUN_VERB_PAIRS: &[(&str, &str)] = &[
    ("advice", "advise"),
    ("belief", "believe"),
    ("breath", "breathe"),
    ("effect", "affect"), // "Effect" is also a verb meaning "to bring about". "Affect" is a noun in psychology.
    ("emphasis", "emphasize"), // TODO how to handle "emphasise" as well as "emphasize"?
    ("intent", "intend"),
    // ("proof", "prove"),  // "Proof" is also a verb, a synonym of "proofread".
    // Add more pairs here as needed
];

use noun_instead_of_verb::NounInsteadOfVerb;
use verb_instead_of_noun::VerbInsteadOfNoun;

merge_linters! {
    NounVerbConfusion =>
        NounInsteadOfVerb,
        VerbInsteadOfNoun
        => "Handles common confusions between related nouns and verbs (e.g., 'advice/advise', 'breath/breathe')"
}

#[cfg(test)]
mod tests {
    use super::NounVerbConfusion;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_good_advise() {
        assert_suggestion_result("Good advise", NounVerbConfusion::default(), "Good advice");
    }

    #[test]
    fn corrects_bad_advise() {
        assert_suggestion_result(
            "I just wanted to bring attention to this because it stood out to me as potentially bad advise.",
            NounVerbConfusion::default(),
            "I just wanted to bring attention to this because it stood out to me as potentially bad advice.",
        );
    }

    #[test]
    fn dont_flag_correct_better_advise() {
        assert_lint_count(
            "Hello! I am an engineer at Plexon and am conducting tests with Kilosort4 so we can better advise our clients.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    #[ignore = "'better advise' can be correct as above, or a mistake like here"]
    fn correct_better_advise() {
        assert_suggestion_result(
            "Maybe this will be a decent idea, .or anybody has better advise :)",
            NounVerbConfusion::default(),
            "Maybe this will be a decent idea, .or anybody has better advice :)",
        );
    }

    #[test]
    fn dont_flag_correct_better_believe() {
        assert_lint_count(
            "You'd better believe this is bbedit-gist-maker.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn correct_strong_believe() {
        assert_suggestion_result(
            "cause my strong believe is that we must give any user to describe whether a post is meant factual",
            NounVerbConfusion::default(),
            "cause my strong belief is that we must give any user to describe whether a post is meant factual",
        );
    }

    #[test]
    fn correct_deep_breathe() {
        assert_suggestion_result(
            "Take deep breathe and Do it again!",
            NounVerbConfusion::default(),
            "Take deep breath and Do it again!",
        );
    }

    #[test]
    fn correct_bad_intend() {
        assert_suggestion_result(
            "What do you do if you only see slightly longer posts that may still be acceptable (and not bad intend from the poster)",
            NounVerbConfusion::default(),
            "What do you do if you only see slightly longer posts that may still be acceptable (and not bad intent from the poster)",
        );
    }

    #[test]
    fn corrects_belief_instead_of_verb() {
        assert_suggestion_result(
            "I belief in you.",
            NounVerbConfusion::default(),
            "I believe in you.",
        );
    }

    #[test]
    #[ignore = "`to` can't disambiguate since it's valid between verbs and nouns"]
    fn corrects_breath_instead_of_verb() {
        assert_suggestion_result(
            "Remember to breath deeply.",
            NounVerbConfusion::default(),
            "Remember to breathe deeply.",
        );
    }

    #[test]
    fn does_not_flag_correct_believe() {
        assert_lint_count("I believe in you.", NounVerbConfusion::default(), 0);
    }

    #[test]
    fn does_not_flag_correct_breath() {
        assert_lint_count("Take a deep breath.", NounVerbConfusion::default(), 0);
    }

    // real-world example unit tests

    #[test]
    fn fix_when_i_breath_you_breath() {
        assert_suggestion_result(
            "When I breath, you breath!",
            NounVerbConfusion::default(),
            "When I breathe, you breathe!",
        );
    }

    #[test]
    fn fix_weather_climate_and_the_air_we_breath() {
        assert_suggestion_result(
            "Weather Climate and the Air We Breath",
            NounVerbConfusion::default(),
            "Weather Climate and the Air We Breathe",
        );
    }

    #[test]
    fn fix_always_breath() {
        assert_suggestion_result(
            "breathing. remember to always breath.",
            NounVerbConfusion::default(),
            "breathing. remember to always breathe.",
        );
    }

    #[test]
    fn fix_never_breath_a_word() {
        assert_suggestion_result(
            "And never breath a word about your loss; If you can force your heart and nerve and sinew.",
            NounVerbConfusion::default(),
            "And never breathe a word about your loss; If you can force your heart and nerve and sinew.",
        );
    }

    #[test]
    fn fix_breath_for_seconds() {
        assert_suggestion_result(
            "Once turned on, the LED on the TX unit would breath for a few seconds, then go completely dead and not responding to objects in front of the sensors.",
            NounVerbConfusion::default(),
            "Once turned on, the LED on the TX unit would breathe for a few seconds, then go completely dead and not responding to objects in front of the sensors.",
        );
    }

    #[test]
    fn fix_breath_a_little_more_life() {
        assert_suggestion_result(
            "... up to 12% more performance, could breath a little more life into systems as old as Sandy Bridge.",
            NounVerbConfusion::default(),
            "... up to 12% more performance, could breathe a little more life into systems as old as Sandy Bridge.",
        );
    }

    #[test]
    fn fix_the_diversity_we_breath() {
        assert_suggestion_result(
            "The Diversity We Breath: Community Diversity",
            NounVerbConfusion::default(),
            "The Diversity We Breathe: Community Diversity",
        );
    }

    #[test]
    fn fix_belief() {
        assert_suggestion_result(
            "While I have no plans to return to aerospace I belief it gives me a unique perspective to many challenges.",
            NounVerbConfusion::default(),
            "While I have no plans to return to aerospace I believe it gives me a unique perspective to many challenges.",
        );
    }

    #[test]
    fn fix_we_belief() {
        assert_suggestion_result(
            "In contrast to other vendors in e-mobility, we belief that true transparency is only trustworthy if the entire process ...",
            NounVerbConfusion::default(),
            "In contrast to other vendors in e-mobility, we believe that true transparency is only trustworthy if the entire process ...",
        );
    }

    #[test]
    #[ignore = "`underwater` is a marginal noun so `breath underwater` matches the compound noun test."]
    fn fix_i_can_breath() {
        assert_suggestion_result(
            "Steps to reproduce Expected behaviour I can breath underwater.",
            NounVerbConfusion::default(),
            "Steps to reproduce Expected behaviour I can breathe underwater.",
        );
    }

    #[test]
    fn fix_caps_should_breath() {
        assert_suggestion_result(
            "CAPS 1 2 3 4 5 A B C D SHOULD BREATH A BIT MORE ?",
            NounVerbConfusion::default(),
            "CAPS 1 2 3 4 5 A B C D SHOULD BREATHE A BIT MORE ?",
        );
    }

    #[test]
    fn fix_can_you_advice_me() {
        assert_suggestion_result(
            "Can you advice me how to train?",
            NounVerbConfusion::default(),
            "Can you advise me how to train?",
        );
    }

    #[test]
    fn fix_we_can_advice_you() {
        assert_suggestion_result(
            "Feel free to share more details about your use case, so we can advice you specifically based on your case.",
            NounVerbConfusion::default(),
            "Feel free to share more details about your use case, so we can advise you specifically based on your case.",
        );
    }

    #[test]
    fn fix_would_advice_against() {
        assert_suggestion_result(
            "So that I would advice against using a spindle in laser mode.",
            NounVerbConfusion::default(),
            "So that I would advise against using a spindle in laser mode.",
        );
    }

    #[test]
    fn fix_advice_to_listen() {
        assert_suggestion_result(
            "The idea of this applicaton was inspired by Ray Dalio, who always advice to listen to people who know more than us by experience.",
            NounVerbConfusion::default(),
            "The idea of this applicaton was inspired by Ray Dalio, who always advise to listen to people who know more than us by experience.",
        );
    }

    #[test]
    #[ignore = "`You` is an object pronoun in this example. `It` is also both subject and object."]
    fn dont_fix_advice_on_that() {
        assert_lint_count(
            "I don't do table returning functions in my code so can't offer you advice on that.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn fix_advice_to_stick_with_openvscode() {
        assert_suggestion_result(
            "But unless you really need it, I would advice to stick with openvscode as there are nearly the same.",
            NounVerbConfusion::default(),
            "But unless you really need it, I would advise to stick with openvscode as there are nearly the same.",
        );
    }

    #[test]
    fn fix_advice_to_back_up_os_image() {
        assert_suggestion_result(
            "I would advice to back up all OS image before any update, because you could lose something what was working previously.",
            NounVerbConfusion::default(),
            "I would advise to back up all OS image before any update, because you could lose something what was working previously.",
        );
    }

    #[test]
    fn fix_advice_to_use_ms_store() {
        assert_suggestion_result(
            "I know we can always advice to use the MS store to download JASP instead",
            NounVerbConfusion::default(),
            "I know we can always advise to use the MS store to download JASP instead",
        );
    }

    #[test]
    fn fix_should_intent_be() {
        assert_suggestion_result(
            "Should intent be on the blocklist?",
            NounVerbConfusion::default(),
            "Should intent be on the blocklist?",
        );
    }

    #[test]
    fn fix_if_you_intent() {
        assert_suggestion_result(
            "If you intent to use a 64 bits machine, change line 74",
            NounVerbConfusion::default(),
            "If you intend to use a 64 bits machine, change line 74",
        );
    }

    #[test]
    fn fix_what_you_would_intent_to_do() {
        assert_suggestion_result(
            "May I ask what you would intent to do with such a feature?",
            NounVerbConfusion::default(),
            "May I ask what you would intend to do with such a feature?",
        );
    }

    #[test]
    fn dont_flag_intent_records() {
        assert_lint_count(
            "there are always intent records associated to the txns",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn fix_did_you_always_intent_to() {
        assert_suggestion_result(
            "Did you always intent to fight malware? No.",
            NounVerbConfusion::default(),
            "Did you always intend to fight malware? No.",
        );
    }

    #[test]
    fn fix_we_recommend_you_create_a_new_issue_on_github_explaining_what_you_intent_to_do() {
        assert_suggestion_result(
            "... we recommend you create a new issue on github explaining what you intent to do.",
            NounVerbConfusion::default(),
            "... we recommend you create a new issue on github explaining what you intend to do.",
        );
    }

    #[test]
    fn fix_intent_to_use_non_imported_symbol() {
        assert_suggestion_result(
            "There's a warning reported for this code, saying that it may intent to use non-imported symbol",
            NounVerbConfusion::default(),
            "There's a warning reported for this code, saying that it may intend to use non-imported symbol",
        );
    }

    // tests for preceding "to"

    #[test]
    fn fix_to_emphasis_the() {
        assert_suggestion_result(
            "This one could be used in a dialog to emphasis the surprise.",
            NounVerbConfusion::default(),
            "This one could be used in a dialog to emphasize the surprise.",
        );
    }

    #[test]
    fn allow_to_emphasis_at_end() {
        assert_lint_count(
            "Changes literal underscores to emphasis",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn allow_to_intent_adjective() {
        assert_lint_count(
            "Cleanup passing statistics to intent aware iterator",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn fix_to_advice_a_class() {
        assert_suggestion_result(
            "How to advice a class that have been intercepted by another javaagent",
            NounVerbConfusion::default(),
            "How to advise a class that have been intercepted by another javaagent",
        );
    }

    #[test]
    fn fix_to_breath_some() {
        assert_suggestion_result(
            "You go to the balcony to breath some fresh air and look down at the things outside.",
            NounVerbConfusion::default(),
            "You go to the balcony to breathe some fresh air and look down at the things outside.",
        );
    }

    #[test]
    fn fix_to_emphasis_a() {
        assert_suggestion_result(
            "we'd like to emphasis a few points below",
            NounVerbConfusion::default(),
            "we'd like to emphasize a few points below",
        );
    }

    #[test]
    fn fix_to_advice_their() {
        assert_suggestion_result(
            "People who are managing this situation tend to advice their users to lock+unlock their screen",
            NounVerbConfusion::default(),
            "People who are managing this situation tend to advise their users to lock+unlock their screen",
        );
    }

    // affect vs. effect sentences gathered from user reports

    #[test]
    fn fix_positive_affect_on_small_businesses() {
        assert_suggestion_result(
            "The new law had a positive affect on small businesses.",
            NounVerbConfusion::default(),
            "The new law had a positive effect on small businesses.",
        );
    }

    #[test]
    fn fix_measured_the_affect_of_caffeine() {
        assert_suggestion_result(
            "We measured the affect of caffeine on reaction time.",
            NounVerbConfusion::default(),
            "We measured the effect of caffeine on reaction time.",
        );
    }

    #[test]
    fn fix_side_affects_included_nausea() {
        assert_suggestion_result(
            "The side affects included nausea and fatigue.",
            NounVerbConfusion::default(),
            "The side effects included nausea and fatigue.",
        );
    }

    #[test]
    fn fix_cause_and_affect_not_same() {
        assert_suggestion_result(
            "Cause and affect are not the same thing.",
            NounVerbConfusion::default(),
            "Cause and effect are not the same thing.",
        );
    }

    #[test]
    fn fix_change_will_have_an_affect_on_revenue() {
        assert_suggestion_result(
            "The change will have an affect on our revenue.",
            NounVerbConfusion::default(),
            "The change will have an effect on our revenue.",
        );
    }

    #[test]
    fn fix_medicine_took_affect_within_minutes() {
        assert_suggestion_result(
            "The medicine took affect within minutes.",
            NounVerbConfusion::default(),
            "The medicine took effect within minutes.",
        );
    }

    #[test]
    fn fix_policy_will_come_into_affect() {
        assert_suggestion_result(
            "The policy will come into affect on October 1.",
            NounVerbConfusion::default(),
            "The policy will come into effect on October 1.",
        );
    }

    #[test]
    fn fix_rules_are_now_in_affect() {
        assert_suggestion_result(
            "The rules are now in affect.",
            NounVerbConfusion::default(),
            "The rules are now in effect.",
        );
    }

    #[test]
    fn fix_with_immediate_affect_office_closed() {
        assert_suggestion_result(
            "With immediate affect, the office is closed.",
            NounVerbConfusion::default(),
            "With immediate effect, the office is closed.",
        );
    }

    #[test]
    fn fix_stunning_special_affects() {
        assert_suggestion_result(
            "The director used stunning special affects.",
            NounVerbConfusion::default(),
            "The director used stunning special effects.",
        );
    }

    #[test]
    fn fix_placebo_affect_can_be_powerful() {
        assert_suggestion_result(
            "The placebo affect can be powerful.",
            NounVerbConfusion::default(),
            "The placebo effect can be powerful.",
        );
    }

    #[test]
    fn fix_ripple_affect_across_market() {
        assert_suggestion_result(
            "We felt the ripple affect across the entire market.",
            NounVerbConfusion::default(),
            "We felt the ripple effect across the entire market.",
        );
    }

    #[test]
    fn fix_snowball_affect_amplified_problem() {
        assert_suggestion_result(
            "The snowball affect amplified the problem.",
            NounVerbConfusion::default(),
            "The snowball effect amplified the problem.",
        );
    }

    #[test]
    fn fix_knock_on_affect_throughout_team() {
        assert_suggestion_result(
            "That decision had a knock-on affect throughout the team.",
            NounVerbConfusion::default(),
            "That decision had a knock-on effect throughout the team.",
        );
    }

    #[test]
    fn fix_greenhouse_affect_warms_planet() {
        assert_suggestion_result(
            "The greenhouse affect warms the planet.",
            NounVerbConfusion::default(),
            "The greenhouse effect warms the planet.",
        );
    }

    #[test]
    fn fix_apology_had_little_affect() {
        assert_suggestion_result(
            "Her apology had little affect.",
            NounVerbConfusion::default(),
            "Her apology had little effect.",
        );
    }

    #[test]
    fn fix_settings_go_into_affect() {
        assert_suggestion_result(
            "The new settings go into affect after a restart.",
            NounVerbConfusion::default(),
            "The new settings go into effect after a restart.",
        );
    }

    #[test]
    fn fix_put_plan_into_affect() {
        assert_suggestion_result(
            "They put the new plan into affect last week.",
            NounVerbConfusion::default(),
            "They put the new plan into effect last week.",
        );
    }

    #[test]
    fn fix_contract_comes_into_affect() {
        assert_suggestion_result(
            "The contract comes into affect at midnight.",
            NounVerbConfusion::default(),
            "The contract comes into effect at midnight.",
        );
    }

    #[test]
    fn fix_warning_had_no_affect_on_behavior() {
        assert_suggestion_result(
            "The warning had no affect on his behavior.",
            NounVerbConfusion::default(),
            "The warning had no effect on his behavior.",
        );
    }

    #[test]
    fn fix_inflation_had_opposite_affect() {
        assert_suggestion_result(
            "Inflation had the opposite affect than expected.",
            NounVerbConfusion::default(),
            "Inflation had the opposite effect than expected.",
        );
    }

    #[test]
    fn fix_regulation_remains_in_affect() {
        assert_suggestion_result(
            "The regulation remains in affect until further notice.",
            NounVerbConfusion::default(),
            "The regulation remains in effect until further notice.",
        );
    }

    #[test]
    fn fix_app_changes_take_affect() {
        assert_suggestion_result(
            "The app changes take affect next week.",
            NounVerbConfusion::default(),
            "The app changes take effect next week.",
        );
    }

    #[test]
    fn fix_sound_affects_were_added() {
        assert_suggestion_result(
            "Sound affects were added in post.",
            NounVerbConfusion::default(),
            "Sound effects were added in post.",
        );
    }

    // Effect/affect-specific checks
    // `effect` mistakenly used as the verb `affect`.
    #[test]
    fn corrects_noun_subject_effects_object() {
        assert_suggestion_result(
            "System outages effect our customers.",
            NounVerbConfusion::default(),
            "System outages affect our customers.",
        );
    }

    #[test]
    fn corrects_effects_variant() {
        assert_suggestion_result(
            "This policy effects employee morale.",
            NounVerbConfusion::default(),
            "This policy affects employee morale.",
        );
    }

    #[test]
    fn ignores_effect_change_idiom() {
        assert_lint_count(
            "Leaders work to effect change in their communities.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_noun_phrase() {
        assert_lint_count(
            "The effect your plan had was dramatic.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_as_result_noun() {
        assert_lint_count(
            "The effect was immediate and obvious.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_to_effect_substitutions() {
        assert_lint_count(
            "or it may be desired to effect substitutions",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_followed_by_of_phrase() {
        assert_lint_count(
            "We measured the effect of caffeine on sleep.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_side_effects_usage() {
        assert_lint_count(
            "Side effects may include mild nausea.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_special_effects_phrase() {
        assert_lint_count(
            "She admired the special effects in the film.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_in_cause_and_effect() {
        assert_lint_count(
            "The diagram explains cause and effect relationships.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn ignores_effects_with_pronoun_subject() {
        assert_lint_count(
            "Those effects were less severe than expected.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn corrects_tariff_effect_import_prices() {
        assert_suggestion_result(
            "The new tariff will effect import prices next quarter.",
            NounVerbConfusion::default(),
            "The new tariff will affect import prices next quarter.",
        );
    }

    #[test]
    fn corrects_droughts_effect_crop_yields() {
        assert_suggestion_result(
            "Prolonged droughts severely effect crop yields across the valley.",
            NounVerbConfusion::default(),
            "Prolonged droughts severely affect crop yields across the valley.",
        );
    }

    #[test]
    fn corrects_caffeine_effect_sleep() {
        assert_suggestion_result(
            "Caffeine can effect your sleep architecture.",
            NounVerbConfusion::default(),
            "Caffeine can affect your sleep architecture.",
        );
    }

    #[test]
    fn corrects_bug_effect_devices() {
        assert_suggestion_result(
            "The firmware bug doesn't effect older devices.",
            NounVerbConfusion::default(),
            "The firmware bug doesn't affect older devices.",
        );
    }

    #[test]
    fn corrects_sarcasm_effect_morale() {
        assert_suggestion_result(
            "Her sarcasm seemed to effect the team's morale.",
            NounVerbConfusion::default(),
            "Her sarcasm seemed to affect the team's morale.",
        );
    }

    #[test]
    fn corrects_outage_effect_timeline() {
        assert_suggestion_result(
            "How will this outage effect our deployment timeline?",
            NounVerbConfusion::default(),
            "How will this outage affect our deployment timeline?",
        );
    }

    #[test]
    fn corrects_temperatures_effect_battery() {
        assert_suggestion_result(
            "Cold temperatures drastically effect lithium-ion battery performance.",
            NounVerbConfusion::default(),
            "Cold temperatures drastically affect lithium-ion battery performance.",
        );
    }

    #[test]
    fn corrects_policy_effect_eligibility() {
        assert_suggestion_result(
            "The policy change could effect your eligibility for benefits.",
            NounVerbConfusion::default(),
            "The policy change could affect your eligibility for benefits.",
        );
    }

    #[test]
    fn corrects_variables_effect_results() {
        assert_suggestion_result(
            "These confounding variables may effect the study's results.",
            NounVerbConfusion::default(),
            "These confounding variables may affect the study's results.",
        );
    }

    #[test]
    fn corrects_fans_effect_concentration() {
        assert_suggestion_result(
            "The noisy HVAC fans constantly effect concentration in the lab.",
            NounVerbConfusion::default(),
            "The noisy HVAC fans constantly affect concentration in the lab.",
        );
    }

    #[test]
    fn corrects_hormones_effect_immunity() {
        assert_suggestion_result(
            "Stress hormones can effect immune response during recovery.",
            NounVerbConfusion::default(),
            "Stress hormones can affect immune response during recovery.",
        );
    }

    #[test]
    fn corrects_pacing_effect_engagement() {
        assert_suggestion_result(
            "The instructor's pacing tended to effect student engagement.",
            NounVerbConfusion::default(),
            "The instructor's pacing tended to affect student engagement.",
        );
    }

    #[test]
    fn corrects_humidity_effect_paint() {
        assert_suggestion_result(
            "Humidity levels directly effect paint curing time.",
            NounVerbConfusion::default(),
            "Humidity levels directly affect paint curing time.",
        );
    }

    #[test]
    fn corrects_exchange_effect_invoice() {
        assert_suggestion_result(
            "The exchange rate will surely effect the final invoice.",
            NounVerbConfusion::default(),
            "The exchange rate will surely affect the final invoice.",
        );
    }

    #[test]
    fn corrects_brightness_effect_contrast() {
        assert_suggestion_result(
            "Screen brightness settings can effect perceived contrast.",
            NounVerbConfusion::default(),
            "Screen brightness settings can affect perceived contrast.",
        );
    }

    #[test]
    fn corrects_medication_effect_him() {
        assert_suggestion_result(
            "The medication didn't effect him the way the doctor expected.",
            NounVerbConfusion::default(),
            "The medication didn't affect him the way the doctor expected.",
        );
    }

    #[test]
    fn corrects_payments_effect_credit() {
        assert_suggestion_result(
            "Late payments will negatively effect your credit score.",
            NounVerbConfusion::default(),
            "Late payments will negatively affect your credit score.",
        );
    }

    #[test]
    fn corrects_wording_effect_interpretation() {
        assert_suggestion_result(
            "Minor wording tweaks shouldn't effect the legal interpretation.",
            NounVerbConfusion::default(),
            "Minor wording tweaks shouldn't affect the legal interpretation.",
        );
    }

    #[test]
    fn corrects_traffic_effect_delivery() {
        assert_suggestion_result(
            "Traffic patterns often effect delivery windows downtown.",
            NounVerbConfusion::default(),
            "Traffic patterns often affect delivery windows downtown.",
        );
    }

    #[test]
    fn corrects_rumor_effect_confidence() {
        assert_suggestion_result(
            "The rumor started to effect investor confidence by noon.",
            NounVerbConfusion::default(),
            "The rumor started to affect investor confidence by noon.",
        );
    }

    #[test]
    fn corrects_allergies_effect_productivity() {
        assert_suggestion_result(
            "Seasonal allergies badly effect her productivity each April.",
            NounVerbConfusion::default(),
            "Seasonal allergies badly affect her productivity each April.",
        );
    }

    #[test]
    fn corrects_feedback_effect_roadmap() {
        assert_suggestion_result(
            "Your feedback won't immediately effect the roadmap.",
            NounVerbConfusion::default(),
            "Your feedback won't immediately affect the roadmap.",
        );
    }

    #[test]
    fn corrects_rules_effect_honeypot() {
        assert_suggestion_result(
            "I cant seem to get my additional rules to effect the honeypot",
            NounVerbConfusion::default(),
            "I cant seem to get my additional rules to affect the honeypot",
        );
    }

    #[test]
    fn corrects_bandwidth_effect_video() {
        assert_suggestion_result(
            "Fluctuating bandwidth can effect video call quality.",
            NounVerbConfusion::default(),
            "Fluctuating bandwidth can affect video call quality.",
        );
    }

    #[test]
    fn corrects_gradient_effect_sensor() {
        assert_suggestion_result(
            "The temperature gradient might effect the sensor's calibration.",
            NounVerbConfusion::default(),
            "The temperature gradient might affect the sensor's calibration.",
        );
    }

    #[test]
    fn corrects_delays_effect_satisfaction() {
        assert_suggestion_result(
            "Even tiny delays can effect user satisfaction metrics.",
            NounVerbConfusion::default(),
            "Even tiny delays can affect user satisfaction metrics.",
        );
    }

    #[test]
    fn corrects_architecture_effect_gps() {
        assert_suggestion_result(
            "The surrounding architecture can effect GPS accuracy.",
            NounVerbConfusion::default(),
            "The surrounding architecture can affect GPS accuracy.",
        );
    }

    #[test]
    fn corrects_lighting_effect_color() {
        assert_suggestion_result(
            "Lighting conditions strongly effect color perception.",
            NounVerbConfusion::default(),
            "Lighting conditions strongly affect color perception.",
        );
    }

    #[test]
    fn corrects_coach_effect_roles() {
        assert_suggestion_result(
            "The new coach's strategy will effect players' roles.",
            NounVerbConfusion::default(),
            "The new coach's strategy will affect players' roles.",
        );
    }

    #[test]
    fn corrects_overtraining_effect_reaction() {
        assert_suggestion_result(
            "Overtraining can effect reaction time and coordination.",
            NounVerbConfusion::default(),
            "Overtraining can affect reaction time and coordination.",
        );
    }

    #[test]
    fn corrects_label_effect_behavior() {
        assert_suggestion_result(
            "The warning label may effect how consumers use the product.",
            NounVerbConfusion::default(),
            "The warning label may affect how consumers use the product.",
        );
    }

    // `affect` mistakenly used as the noun `effect`.
    #[test]
    fn corrects_because_affect_is() {
        assert_suggestion_result(
            "I worry because affect is hidden.",
            NounVerbConfusion::default(),
            "I worry because effect is hidden.",
        );
    }

    #[test]
    fn ignores_psychology_usage() {
        assert_lint_count(
            "The patient's affect is flat.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn corrects_positive_affect_on() {
        assert_suggestion_result(
            "The new law had a positive affect on small businesses.",
            NounVerbConfusion::default(),
            "The new law had a positive effect on small businesses.",
        );
    }

    #[test]
    fn corrects_great_affect() {
        assert_suggestion_result(
            "badges that they provide to users to allow them to promote their projects to great affect",
            NounVerbConfusion::default(),
            "badges that they provide to users to allow them to promote their projects to great effect",
        );
    }

    #[test]
    fn corrects_affect_of() {
        assert_suggestion_result(
            "We measured the affect of caffeine on reaction time.",
            NounVerbConfusion::default(),
            "We measured the effect of caffeine on reaction time.",
        );
    }

    #[test]
    fn corrects_side_affects() {
        assert_suggestion_result(
            "The side affects included nausea and fatigue.",
            NounVerbConfusion::default(),
            "The side effects included nausea and fatigue.",
        );
    }

    #[test]
    fn corrects_cause_and_affect() {
        assert_suggestion_result(
            "Cause and affect are not the same thing.",
            NounVerbConfusion::default(),
            "Cause and effect are not the same thing.",
        );
    }

    #[test]
    fn corrects_have_an_affect_on() {
        assert_suggestion_result(
            "The change will have an affect on our revenue.",
            NounVerbConfusion::default(),
            "The change will have an effect on our revenue.",
        );
    }

    #[test]
    fn corrects_took_affect() {
        assert_suggestion_result(
            "The medicine took affect within minutes.",
            NounVerbConfusion::default(),
            "The medicine took effect within minutes.",
        );
    }

    #[test]
    fn corrects_come_into_affect() {
        assert_suggestion_result(
            "The policy will come into affect on October 1.",
            NounVerbConfusion::default(),
            "The policy will come into effect on October 1.",
        );
    }

    #[test]
    fn corrects_in_affect_sentence() {
        assert_suggestion_result(
            "The rules are now in affect.",
            NounVerbConfusion::default(),
            "The rules are now in effect.",
        );
    }

    #[test]
    fn corrects_with_immediate_affect() {
        assert_suggestion_result(
            "With immediate affect, the office is closed.",
            NounVerbConfusion::default(),
            "With immediate effect, the office is closed.",
        );
    }

    #[test]
    fn corrects_special_affects() {
        assert_suggestion_result(
            "The director used stunning special affects.",
            NounVerbConfusion::default(),
            "The director used stunning special effects.",
        );
    }

    #[test]
    fn corrects_placebo_affect() {
        assert_suggestion_result(
            "The placebo affect can be powerful.",
            NounVerbConfusion::default(),
            "The placebo effect can be powerful.",
        );
    }

    #[test]
    fn corrects_ripple_affect() {
        assert_suggestion_result(
            "We felt the ripple affect across the entire market.",
            NounVerbConfusion::default(),
            "We felt the ripple effect across the entire market.",
        );
    }

    #[test]
    fn corrects_snowball_affect() {
        assert_suggestion_result(
            "The snowball affect amplified the problem.",
            NounVerbConfusion::default(),
            "The snowball effect amplified the problem.",
        );
    }

    #[test]
    fn corrects_knock_on_affect() {
        assert_suggestion_result(
            "That decision had a knock-on affect throughout the team.",
            NounVerbConfusion::default(),
            "That decision had a knock-on effect throughout the team.",
        );
    }

    #[test]
    fn corrects_greenhouse_affect() {
        assert_suggestion_result(
            "The greenhouse affect warms the planet.",
            NounVerbConfusion::default(),
            "The greenhouse effect warms the planet.",
        );
    }

    #[test]
    fn corrects_little_affect() {
        assert_suggestion_result(
            "Her apology had little affect.",
            NounVerbConfusion::default(),
            "Her apology had little effect.",
        );
    }

    #[test]
    fn corrects_go_into_affect() {
        assert_suggestion_result(
            "The new settings go into affect after a restart.",
            NounVerbConfusion::default(),
            "The new settings go into effect after a restart.",
        );
    }

    #[test]
    fn corrects_put_plan_into_affect() {
        assert_suggestion_result(
            "They put the new plan into affect last week.",
            NounVerbConfusion::default(),
            "They put the new plan into effect last week.",
        );
    }

    #[test]
    fn corrects_contract_into_affect() {
        assert_suggestion_result(
            "The contract comes into affect at midnight.",
            NounVerbConfusion::default(),
            "The contract comes into effect at midnight.",
        );
    }

    #[test]
    fn corrects_no_affect_on_behavior() {
        assert_suggestion_result(
            "The warning had no affect on his behavior.",
            NounVerbConfusion::default(),
            "The warning had no effect on his behavior.",
        );
    }

    #[test]
    fn corrects_opposite_affect() {
        assert_suggestion_result(
            "Inflation had the opposite affect than expected.",
            NounVerbConfusion::default(),
            "Inflation had the opposite effect than expected.",
        );
    }

    #[test]
    fn corrects_remains_in_affect() {
        assert_suggestion_result(
            "The regulation remains in affect until further notice.",
            NounVerbConfusion::default(),
            "The regulation remains in effect until further notice.",
        );
    }

    #[test]
    fn corrects_take_affect_next_week() {
        assert_suggestion_result(
            "The app changes take affect next week.",
            NounVerbConfusion::default(),
            "The app changes take effect next week.",
        );
    }

    #[test]
    fn corrects_sound_affects() {
        assert_suggestion_result(
            "Sound affects were added in post.",
            NounVerbConfusion::default(),
            "Sound effects were added in post.",
        );
    }

    #[test]
    fn does_not_flag_best_affect() {
        assert_lint_count(
            "Using linear regression to predict and understand what factors best affect house price",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_sound_affect() {
        assert_lint_count(
            "The goal of this study was to learn what properties of sound affect human focus the most.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn corrects_sound_affect() {
        assert_suggestion_result(
            "Diesel Generator's animation returns to 'idle' state, but it's sound affect remains in the 'work' state.",
            NounVerbConfusion::default(),
            "Diesel Generator's animation returns to 'idle' state, but it's sound effect remains in the 'work' state.",
        );
    }

    #[test]
    fn does_not_flag_affect_as_verb() {
        assert_lint_count(
            "The change will affect our revenue significantly.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_affects_as_verb() {
        assert_lint_count(
            "This policy directly affects remote workers.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_correct_effect_noun() {
        assert_lint_count(
            "The placebo effect can be powerful.",
            NounVerbConfusion::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_sound_effects() {
        assert_lint_count(
            "Sound effects were added in post.",
            NounVerbConfusion::default(),
            0,
        );
    }
}
