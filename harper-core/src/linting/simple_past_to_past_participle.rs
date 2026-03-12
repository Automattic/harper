use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{All, Expr, FirstMatchOf, SequenceExpr},
    irregular_verbs::IrregularVerbs,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{InflectionOfBe, WordSet},
};

/// Corrects simple past tense verbs to past participle after auxiliary verbs like "have" or "be".
pub struct SimplePastToPastParticiple {
    expr: Box<dyn Expr>,
}

impl Default for SimplePastToPastParticiple {
    fn default() -> Self {
        Self {
            expr: Box::new(All::new(vec![
                // positive: the general case
                Box::new(
                    SequenceExpr::any_of(vec![
                        // for perfect tenses
                        Box::new(WordSet::new(&["have", "had", "has", "having"])),
                        // for passive voice
                        Box::new(InflectionOfBe::default()),
                        // pronoun + have contractions
                        Box::new(WordSet::new(&[
                            "I've", "I'd", "we've", "we'd", "you've", "you'd", "he's", "he'd",
                            "she's", "she'd", "it's", "it'd", "they've", "they'd",
                        ])),
                        // pronoun + have contractions missing apostrophes
                        Box::new(WordSet::new(&[
                            "Ive", "Id", "weve", "wed", "youve", "youd", "hes", "hed", "shes",
                            "shed", "its", "itd", "theyve", "theyd",
                        ])),
                    ])
                    .t_ws()
                    .then_verb_simple_past_form(),
                ),
                // negative: exceptions
                Box::new(SequenceExpr::unless(FirstMatchOf::new(vec![
                    Box::new(
                        SequenceExpr::with(InflectionOfBe::default())
                            .t_any()
                            .t_aco("woke"),
                    ),
                    Box::new(
                        SequenceExpr::aco("id")
                            .t_any()
                            .then_word_set(&["came", "did", "went"]),
                    ),
                ]))),
            ])),
        }
    }
}

impl ExprLinter for SimplePastToPastParticiple {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 3 || !toks[1].kind.is_whitespace() || !toks[2].kind.is_verb() {
            return None;
        }

        let verb_tok = &toks[2];

        let simple_past = verb_tok.span.get_content_string(src);

        if let Some(past_participle) = IrregularVerbs::curated()
            .get_past_participle_for_preterite(&simple_past)
            .filter(|pp| pp != &simple_past)
        {
            let suggestions = vec![Suggestion::replace_with_match_case(
                past_participle.chars().collect(),
                verb_tok.span.get_content(src),
            )];

            let message = format!(
                "Use the past participle `{}` instead of `{}` when using compound tenses or passive voice.",
                past_participle, simple_past
            );

            Some(Lint {
                span: verb_tok.span,
                lint_kind: LintKind::Grammar,
                suggestions,
                message,
                ..Default::default()
            })
        } else {
            None
        }
    }

    fn description(&self) -> &str {
        "Corrects simple past tense verbs to past participle after auxiliary verbs like \"have\" or \"be\"."
    }
}

#[cfg(test)]
mod tests {
    use super::SimplePastToPastParticiple;
    use crate::linting::tests::{
        assert_no_lints, assert_suggestion_result, assert_top3_suggestion_result,
    };

    // "Be" and "have"

    #[test]
    fn correct_have_went() {
        assert_suggestion_result(
            "I have went into the btle.py file and added a print statement in _connect()",
            SimplePastToPastParticiple::default(),
            "I have gone into the btle.py file and added a print statement in _connect()",
        );
    }

    #[test]
    fn correct_had_went() {
        assert_top3_suggestion_result(
            "Not sure if TroLoos had went from Tasmota->minimal->Tasmota, or directly Minimal->Tasmota, but going ESPHome->Minimal->Tasmota is not possible",
            SimplePastToPastParticiple::default(),
            "Not sure if TroLoos had gone from Tasmota->minimal->Tasmota, or directly Minimal->Tasmota, but going ESPHome->Minimal->Tasmota is not possible",
        );
    }

    #[test]
    fn correct_having_went() {
        assert_suggestion_result(
            "Having went through the setup guidelines and picking react starter, running npm run watch results in an error",
            SimplePastToPastParticiple::default(),
            "Having gone through the setup guidelines and picking react starter, running npm run watch results in an error",
        );
    }

    #[test]
    fn correct_has_went() {
        assert_suggestion_result(
            "I would like to report that the package request which you are loading has went into maintenance mode.",
            SimplePastToPastParticiple::default(),
            "I would like to report that the package request which you are loading has gone into maintenance mode.",
        );
    }

    #[test]
    fn correct_have_wrote() {
        assert_suggestion_result(
            "and while people have wrote partial ImGuiStyle save and ...",
            SimplePastToPastParticiple::default(),
            "and while people have written partial ImGuiStyle save and ...",
        );
    }

    #[test]
    fn correct_has_came() {
        assert_suggestion_result(
            "and mail has came to a work account",
            SimplePastToPastParticiple::default(),
            "and mail has come to a work account",
        );
    }

    #[test]
    fn correct_have_took() {
        assert_suggestion_result(
            "The Keychain took longer than I'd like it to have took, but it still works",
            SimplePastToPastParticiple::default(),
            "The Keychain took longer than I'd like it to have taken, but it still works",
        );
    }

    #[test]
    fn correct_have_did() {
        assert_suggestion_result(
            "so I have did like below: cd ~/.pub-cache/hosted/pub.dev/",
            SimplePastToPastParticiple::default(),
            "so I have done like below: cd ~/.pub-cache/hosted/pub.dev/",
        );
    }

    #[test]
    fn correct_has_fell() {
        assert_suggestion_result(
            "ScopedHistory instance has fell out of scope ...",
            SimplePastToPastParticiple::default(),
            "ScopedHistory instance has fallen out of scope ...",
        );
    }

    #[test]
    fn correct_have_broke() {
        assert_suggestion_result(
            "PlanningEnitity to see the hard constraints that it may have broke",
            SimplePastToPastParticiple::default(),
            "PlanningEnitity to see the hard constraints that it may have broken",
        );
    }

    #[test]
    fn correct_had_began() {
        assert_top3_suggestion_result(
            "I had began learning Android App development since Aug 2021",
            SimplePastToPastParticiple::default(),
            "I had begun learning Android App development since Aug 2021",
        );
    }

    #[test]
    fn correct_have_gave() {
        assert_suggestion_result(
            "I'm not aware we have gave up SM75, why are you asking this?",
            SimplePastToPastParticiple::default(),
            "I'm not aware we have given up SM75, why are you asking this?",
        );
    }

    #[test]
    fn correct_have_saw() {
        assert_top3_suggestion_result(
            "I have saw that your paper has been accepted by JAIR",
            SimplePastToPastParticiple::default(),
            "I have seen that your paper has been accepted by JAIR",
        );
    }

    #[test]
    fn correct_have_spoke() {
        assert_suggestion_result(
            "so i may have spoke in error",
            SimplePastToPastParticiple::default(),
            "so i may have spoken in error",
        );
    }

    #[test]
    fn correct_has_became() {
        assert_suggestion_result(
            "but it has became failed after v2.6.1",
            SimplePastToPastParticiple::default(),
            "but it has become failed after v2.6.1",
        );
    }

    #[test]
    fn correct_have_knew() {
        assert_suggestion_result(
            "Oh, I have knew this. You can decrypted it in \"Assetstudio\".",
            SimplePastToPastParticiple::default(),
            "Oh, I have known this. You can decrypted it in \"Assetstudio\".",
        );
    }

    #[test]
    fn correct_have_drank() {
        assert_suggestion_result(
            "User should be able to see approximately how much water they have drank today",
            SimplePastToPastParticiple::default(),
            "User should be able to see approximately how much water they have drunk today",
        );
    }

    #[test]
    #[ignore = "'Woke' is also an adjective these days"]
    fn being_woke() {
        assert_suggestion_result(
            "and the containers will not being woke up until I execute a \"docker ps\"",
            SimplePastToPastParticiple::default(),
            "and the containers will not being woken up until I execute a \"docker ps\"",
        );
    }

    #[test]
    fn correct_has_flew() {
        assert_suggestion_result(
            "Well time has flew and I was quite busy but I remember this conversation so I am sharing this with you.",
            SimplePastToPastParticiple::default(),
            "Well time has flown and I was quite busy but I remember this conversation so I am sharing this with you.",
        );
    }

    #[test]
    fn correct_being_stole() {
        assert_suggestion_result(
            "any requests to obtain the hostname will return the hostname of the container being stole",
            SimplePastToPastParticiple::default(),
            "any requests to obtain the hostname will return the hostname of the container being stolen",
        );
    }

    #[test]
    fn correct_are_broke() {
        assert_suggestion_result(
            "They all worked wonderfully under 3.4.2 and all are broke under 3.5.1.",
            SimplePastToPastParticiple::default(),
            "They all worked wonderfully under 3.4.2 and all are broken under 3.5.1.",
        );
    }

    #[test]
    fn correct_were_gave() {
        assert_suggestion_result(
            "Some devices were gave up during a storm recently, but some are still the same as before.",
            SimplePastToPastParticiple::default(),
            "Some devices were given up during a storm recently, but some are still the same as before.",
        );
    }

    #[test]
    fn correct_be_saw() {
        assert_suggestion_result(
            "Currently, it's 14560/14550 for default mavlink RX/TX, which can be saw in wfb-cli .",
            SimplePastToPastParticiple::default(),
            "Currently, it's 14560/14550 for default mavlink RX/TX, which can be seen in wfb-cli .",
        );
    }

    #[test]
    fn correct_was_began() {
        assert_suggestion_result(
            "The initial intent, when v1alpha3 was began, was that almost all usages of InitConfiguration outside of kubeadm init code, could be easily replaced",
            SimplePastToPastParticiple::default(),
            "The initial intent, when v1alpha3 was begun, was that almost all usages of InitConfiguration outside of kubeadm init code, could be easily replaced",
        );
    }

    #[test]
    fn correct_was_gave() {
        assert_suggestion_result(
            "you will find the config file path was gave by -c argument",
            SimplePastToPastParticiple::default(),
            "you will find the config file path was given by -c argument",
        );
    }

    #[test]
    fn correct_be_began() {
        assert_suggestion_result(
            "Ticket requires something from design before it can be began.",
            SimplePastToPastParticiple::default(),
            "Ticket requires something from design before it can be begun.",
        );
    }

    #[test]
    fn correct_being_took() {
        assert_suggestion_result(
            "Dunno, I saw some old threads about port not being took into account in asw-sdk library but seems fixed on aws side.",
            SimplePastToPastParticiple::default(),
            "Dunno, I saw some old threads about port not being taken into account in asw-sdk library but seems fixed on aws side.",
        );
    }

    #[test]
    fn correct_are_took() {
        assert_suggestion_result(
            "In the example provided, TP53 and LMNB1 genes are took as seeds.",
            SimplePastToPastParticiple::default(),
            "In the example provided, TP53 and LMNB1 genes are taken as seeds.",
        );
    }

    // Contractions

    #[test]
    fn correct_ive_went() {
        assert_suggestion_result(
            "I've went through some tutorials and went back and forth with AI translating programs from one language to the other.",
            SimplePastToPastParticiple::default(),
            "I've gone through some tutorials and went back and forth with AI translating programs from one language to the other.",
        );
    }

    #[test]
    fn correct_ive_went_no_apostrophe() {
        assert_suggestion_result(
            "I've went thru all the steps to help fix this Virus issue and im locked up.",
            SimplePastToPastParticiple::default(),
            "I've gone thru all the steps to help fix this Virus issue and im locked up.",
        );
    }

    #[test]
    fn correct_id_did() {
        assert_suggestion_result(
            "I'd did a calibration after the FW update now.",
            SimplePastToPastParticiple::default(),
            "I'd done a calibration after the FW update now.",
        );
    }

    #[test]
    fn correct_weve_went() {
        assert_suggestion_result(
            "Thanks for the feedback, but the issue is no longer relevant since we've went with different approach.",
            SimplePastToPastParticiple::default(),
            "Thanks for the feedback, but the issue is no longer relevant since we've gone with different approach.",
        );
    }

    #[test]
    fn correct_wed_chose() {
        assert_suggestion_result(
            "whatever number we'd chose, only one tab will be allowed to run",
            SimplePastToPastParticiple::default(),
            "whatever number we'd chosen, only one tab will be allowed to run",
        );
    }

    #[test]
    fn correct_youve_wrote() {
        assert_suggestion_result(
            "I love this project, it's impressing how many refactoring you've wrote in a limited amount of time.",
            SimplePastToPastParticiple::default(),
            "I love this project, it's impressing how many refactoring you've written in a limited amount of time.",
        );
    }

    #[test]
    fn correct_youve_ran_no_apostrophe() {
        assert_suggestion_result(
            "after youve ran it, execute the file_mover.ps1 using powershell",
            SimplePastToPastParticiple::default(),
            "after youve run it, execute the file_mover.ps1 using powershell",
        );
    }

    #[test]
    fn correct_youd_wrote() {
        assert_suggestion_result(
            "When I saw you'd wrote a terminal emulator I had to try it and so far it's amazing.",
            SimplePastToPastParticiple::default(),
            "When I saw you'd written a terminal emulator I had to try it and so far it's amazing.",
        );
    }

    #[test]
    fn correct_its_broke() {
        assert_suggestion_result(
            "Not sure why it's broke for me but not for you.",
            SimplePastToPastParticiple::default(),
            "Not sure why it's broken for me but not for you.",
        );
    }

    #[test]
    fn correct_its_broke_no_apostrophe() {
        assert_suggestion_result(
            "Now its broke and won't do batch images (decoding error).",
            SimplePastToPastParticiple::default(),
            "Now its broken and won't do batch images (decoding error).",
        );
    }

    #[test]
    fn correct_theyve_broke() {
        assert_suggestion_result(
            "They've broke something again :D.",
            SimplePastToPastParticiple::default(),
            "They've broken something again :D.",
        );
    }

    #[test]
    fn correct_theyd_forgot() {
        assert_suggestion_result(
            "they found the process they'd forgot they were running",
            SimplePastToPastParticiple::default(),
            "they found the process they'd forgotten they were running",
        );
    }

    // Known exceptions

    #[test]
    fn dont_flag_being_woke() {
        assert_no_lints(
            "Being woke to gender discrimination is difficult",
            SimplePastToPastParticiple::default(),
        );
    }

    #[test]
    fn dont_flag_be_woke() {
        assert_no_lints(
            "So You Want To Be Woke. The path to becoming woke is hard",
            SimplePastToPastParticiple::default(),
        );
    }

    #[test]
    fn dont_flag_id_did() {
        assert_no_lints(
            "Prop id did not match.",
            SimplePastToPastParticiple::default(),
        );
    }

    #[test]
    fn dont_flag_id_came() {
        assert_no_lints(
            "I'm a longtime user of UniFi and site ID came around after my account was established.",
            SimplePastToPastParticiple::default(),
        );
    }

    #[test]
    fn dont_flag_id_went() {
        assert_no_lints(
            "Could not determine debug ID went away after cleaning the dist/ before the build, so that's unrelated.",
            SimplePastToPastParticiple::default(),
        );
    }
}
