use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::expr::{FirstMatchOf, LongestMatchOf};
use crate::{Lrc, Token, patterns::WordSet};

use super::{ExprLinter, Lint, LintKind, Suggestion};

/// Common noun-verb pairs that are often confused
const NOUN_VERB_PAIRS: &[(&str, &str)] = &[
    ("advice", "advise"),
    ("belief", "believe"),
    ("breath", "breathe"),
    ("effect", "affect"), // "Effect" is also a verb meaning "to bring about".
    ("intent", "intend"),
    // ("proof", "prove"),  // "Proof" is also a verb, a synonym of "proofread".
    // Add more pairs here as needed
];

/// Pronouns that can come before verbs but not nouns
const PRONOUNS: &[&str] = &["he", "I", "it", "she", "they", "we", "who", "you"];

/// Adverbs that can come before verbs but not nouns
/// Note: "Sometimes" can come before a noun.
const ADVERBS: &[&str] = &["always", "never", "often", "seldom"];

/// Modal verbs that can come before other verbs but not nouns
const MODAL_VERBS_ETC: &[&str] = &[
    "can",
    "cannot",
    "can't",
    "could",
    "couldn't",
    "may",
    "might",
    "mightn't",
    "must",
    "mustn't",
    "shall",
    "shan't",
    "should",
    "shouldn't",
    "will",
    "won't",
    "would",
    "wouldn't",
    // not modals per se, but modal-like
    "do",
    "don't",
];

/// Linter that corrects common noun/verb confusions
pub struct NounInsteadOfVerb {
    expr: Box<dyn Expr>,
}

impl Default for NounInsteadOfVerb {
    fn default() -> Self {
        let pre_context = FirstMatchOf::new(vec![
            Box::new(WordSet::new(PRONOUNS)),
            Box::new(WordSet::new(MODAL_VERBS_ETC)),
            Box::new(WordSet::new(ADVERBS)),
        ]);

        let nouns = Lrc::new(WordSet::new(
            &NOUN_VERB_PAIRS
                .iter()
                .map(|&(noun, _)| noun)
                .collect::<Vec<_>>(),
        ));

        let basic_pattern = Lrc::new(
            SequenceExpr::default()
                .then(pre_context)
                .then_whitespace()
                .then(nouns.clone()),
        );

        let pattern_followed_by_punctuation = SequenceExpr::default()
            .then(basic_pattern.clone())
            .then_punctuation();

        let pattern_followed_by_word = SequenceExpr::default()
            .then(basic_pattern.clone())
            .then_whitespace()
            .then_any_word();

        Self {
            expr: Box::new(LongestMatchOf::new(vec![
                Box::new(pattern_followed_by_punctuation),
                Box::new(pattern_followed_by_word),
                Box::new(basic_pattern),
            ])),
        }
    }
}

impl ExprLinter for NounInsteadOfVerb {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // If we have the next word token, try to rule out compound nouns
        if toks.len() > 4 {
            let following_tok = &toks[4];
            if following_tok.kind.is_noun() && !following_tok.kind.is_preposition() {
                // But first rule out marginal "nouns"
                let following_lower = following_tok.span.get_content_string(src).to_lowercase();
                if following_lower != "it"
                    && following_lower != "me"
                    && following_lower != "on"
                    && following_lower != "that"
                {
                    return None;
                }
            }
        }

        let noun_tok = &toks[2];
        let noun_chars = noun_tok.span.get_content(src);
        let noun_text = noun_tok.span.get_content_string(src);
        let noun_lower = noun_text.to_lowercase();

        let verb = NOUN_VERB_PAIRS
            .iter()
            .find(|(noun, _)| *noun == noun_lower)
            .map(|(_, verb)| verb)?;

        Some(Lint {
            span: noun_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                verb.chars().collect(),
                noun_chars,
            )],
            message: format!("`{noun_text}` is a noun, the verb should be `{verb}`."),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects nouns used instead of verbs when the two are related."
    }
}

#[cfg(test)]
mod tests {
    use super::NounInsteadOfVerb;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // made up unit tests

    #[test]
    fn corrects_belief_instead_of_verb() {
        assert_suggestion_result(
            "I belief in you.",
            NounInsteadOfVerb::default(),
            "I believe in you.",
        );
    }

    #[test]
    #[ignore = "`to` can't disambiguate since it's valid between verbs and nouns"]
    fn corrects_breath_instead_of_verb() {
        assert_suggestion_result(
            "Remember to breath deeply.",
            NounInsteadOfVerb::default(),
            "Remember to breathe deeply.",
        );
    }

    #[test]
    fn does_not_flag_correct_believe() {
        assert_lint_count("I believe in you.", NounInsteadOfVerb::default(), 0);
    }

    #[test]
    fn does_not_flag_correct_breath() {
        assert_lint_count("Take a deep breath.", NounInsteadOfVerb::default(), 0);
    }

    // real-world example unit tests

    #[test]
    fn fix_when_i_breath_you_breath() {
        assert_suggestion_result(
            "When I breath, you breath!",
            NounInsteadOfVerb::default(),
            "When I breathe, you breathe!",
        );
    }

    #[test]
    fn fix_weather_climate_and_the_air_we_breath() {
        assert_suggestion_result(
            "Weather Climate and the Air We Breath",
            NounInsteadOfVerb::default(),
            "Weather Climate and the Air We Breathe",
        );
    }

    #[test]
    fn fix_always_breath() {
        assert_suggestion_result(
            "breathing. remember to always breath.",
            NounInsteadOfVerb::default(),
            "breathing. remember to always breathe.",
        );
    }

    #[test]
    fn fix_never_breath_a_word() {
        assert_suggestion_result(
            "And never breath a word about your loss; If you can force your heart and nerve and sinew.",
            NounInsteadOfVerb::default(),
            "And never breathe a word about your loss; If you can force your heart and nerve and sinew.",
        );
    }

    #[test]
    fn fix_breath_for_seconds() {
        assert_suggestion_result(
            "Once turned on, the LED on the TX unit would breath for a few seconds, then go completely dead and not responding to objects in front of the sensors.",
            NounInsteadOfVerb::default(),
            "Once turned on, the LED on the TX unit would breathe for a few seconds, then go completely dead and not responding to objects in front of the sensors.",
        );
    }

    #[test]
    fn fix_breath_a_little_more_life() {
        assert_suggestion_result(
            "... up to 12% more performance, could breath a little more life into systems as old as Sandy Bridge.",
            NounInsteadOfVerb::default(),
            "... up to 12% more performance, could breathe a little more life into systems as old as Sandy Bridge.",
        );
    }

    #[test]
    fn fix_the_diversity_we_breath() {
        assert_suggestion_result(
            "The Diversity We Breath: Community Diversity",
            NounInsteadOfVerb::default(),
            "The Diversity We Breathe: Community Diversity",
        );
    }

    #[test]
    fn fix_belief() {
        assert_suggestion_result(
            "While I have no plans to return to aerospace I belief it gives me a unique perspective to many challenges.",
            NounInsteadOfVerb::default(),
            "While I have no plans to return to aerospace I believe it gives me a unique perspective to many challenges.",
        );
    }

    #[test]
    fn fix_we_belief() {
        assert_suggestion_result(
            "In contrast to other vendors in e-mobility, we belief that true transparency is only trustworthy if the entire process ...",
            NounInsteadOfVerb::default(),
            "In contrast to other vendors in e-mobility, we believe that true transparency is only trustworthy if the entire process ...",
        );
    }

    #[test]
    #[ignore = "`underwater` is a marginal noun so `breath underwater` matches the compound noun test."]
    fn fix_i_can_breath() {
        assert_suggestion_result(
            "Steps to reproduce Expected behaviour I can breath underwater.",
            NounInsteadOfVerb::default(),
            "Steps to reproduce Expected behaviour I can breathe underwater.",
        );
    }

    #[test]
    fn fix_caps_should_breath() {
        assert_suggestion_result(
            "CAPS 1 2 3 4 5 A B C D SHOULD BREATH A BIT MORE ?",
            NounInsteadOfVerb::default(),
            "CAPS 1 2 3 4 5 A B C D SHOULD BREATHE A BIT MORE ?",
        );
    }

    #[test]
    fn fix_can_you_advice_me() {
        assert_suggestion_result(
            "Can you advice me how to train?",
            NounInsteadOfVerb::default(),
            "Can you advise me how to train?",
        );
    }

    #[test]
    fn fix_we_can_advice_you() {
        assert_suggestion_result(
            "Feel free to share more details about your use case, so we can advice you specifically based on your case.",
            NounInsteadOfVerb::default(),
            "Feel free to share more details about your use case, so we can advise you specifically based on your case.",
        );
    }

    #[test]
    fn fix_would_advice_against() {
        assert_suggestion_result(
            "So that I would advice against using a spindle in laser mode.",
            NounInsteadOfVerb::default(),
            "So that I would advise against using a spindle in laser mode.",
        );
    }

    #[test]
    fn fix_advice_to_listen() {
        assert_suggestion_result(
            "The idea of this applicaton was inspired by Ray Dalio, who always advice to listen to people who know more than us by experience.",
            NounInsteadOfVerb::default(),
            "The idea of this applicaton was inspired by Ray Dalio, who always advise to listen to people who know more than us by experience.",
        );
    }

    #[test]
    #[ignore = "`You` is an object pronoun in this example. `It` is also both subject and object."]
    fn dont_fix_advice_on_that() {
        assert_lint_count(
            "I don't do table returning functions in my code so can't offer you advice on that.",
            NounInsteadOfVerb::default(),
            0,
        );
    }

    #[test]
    fn fix_advice_to_stick_with_openvscode() {
        assert_suggestion_result(
            "But unless you really need it, I would advice to stick with openvscode as there are nearly the same.",
            NounInsteadOfVerb::default(),
            "But unless you really need it, I would advise to stick with openvscode as there are nearly the same.",
        );
    }

    #[test]
    fn fix_advice_to_back_up_os_image() {
        assert_suggestion_result(
            "I would advice to back up all OS image before any update, because you could lose something what was working previously.",
            NounInsteadOfVerb::default(),
            "I would advise to back up all OS image before any update, because you could lose something what was working previously.",
        );
    }

    #[test]
    fn fix_advice_to_use_ms_store() {
        assert_suggestion_result(
            "I know we can always advice to use the MS store to download JASP instead",
            NounInsteadOfVerb::default(),
            "I know we can always advise to use the MS store to download JASP instead",
        );
    }

    #[test]
    fn fix_should_intent_be() {
        assert_suggestion_result(
            "Should intent be on the blocklist?",
            NounInsteadOfVerb::default(),
            "Should intent be on the blocklist?",
        );
    }

    #[test]
    fn fix_if_you_intent() {
        assert_suggestion_result(
            "If you intent to use a 64 bits machine, change line 74",
            NounInsteadOfVerb::default(),
            "If you intend to use a 64 bits machine, change line 74",
        );
    }

    #[test]
    fn fix_what_you_would_intent_to_do() {
        assert_suggestion_result(
            "May I ask what you would intent to do with such a feature?",
            NounInsteadOfVerb::default(),
            "May I ask what you would intend to do with such a feature?",
        );
    }

    #[test]
    fn dont_flag_intent_records() {
        assert_lint_count(
            "there are always intent records associated to the txns",
            NounInsteadOfVerb::default(),
            0,
        );
    }

    #[test]
    fn fix_did_you_always_intent_to() {
        assert_suggestion_result(
            "Did you always intent to fight malware? No.",
            NounInsteadOfVerb::default(),
            "Did you always intend to fight malware? No.",
        );
    }

    #[test]
    fn fix_we_recommend_you_create_a_new_issue_on_github_explaining_what_you_intent_to_do() {
        assert_suggestion_result(
            "... we recommend you create a new issue on github explaining what you intent to do.",
            NounInsteadOfVerb::default(),
            "... we recommend you create a new issue on github explaining what you intend to do.",
        );
    }

    #[test]
    fn fix_intent_to_use_non_imported_symbol() {
        assert_suggestion_result(
            "There's a warning reported for this code, saying that it may intent to use non-imported symbol",
            NounInsteadOfVerb::default(),
            "There's a warning reported for this code, saying that it may intend to use non-imported symbol",
        );
    }
}
