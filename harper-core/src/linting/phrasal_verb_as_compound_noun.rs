use std::sync::Arc;

use crate::{
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::{Dictionary, FstDictionary},
    {CharStringExt, Document, Token, TokenStringExt},
};

/// Detect phrasal verbs written as compound nouns.
pub struct PhrasalVerbAsCompoundNoun {
    dict: Arc<FstDictionary>,
}

#[derive(Debug)]
enum Confidence {
    DefinitelyVerb,
    PossiblyVerb,
}

impl PhrasalVerbAsCompoundNoun {
    pub fn new() -> Self {
        Self {
            dict: FstDictionary::curated(),
        }
    }
}

impl Default for PhrasalVerbAsCompoundNoun {
    fn default() -> Self {
        Self {
            dict: FstDictionary::curated(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Why {
    ItsAProperNounOrVerb,
    ItContainsSpaceHyphenOrApostrophe,
    ItsAKnownFalsePositive,
    ItDoesntEndWithOneOfTheParticles,
    NeitherTheWholeWordNorThePartBeforeTheParticleIsAVerb,
    Contextual(Context),
    TheresNothingWrongWithIt,
}

#[derive(Debug, PartialEq)]
enum Context {
    ItsInIsolation,
    IHaveNoConfidenceThatItsAVerb,
    TheresAnAdjectiveOrDeterminerBeforeIt,
    ItsPrecededByANonPluralNoun,
    ItsAnItemInAListOfNouns,
    ItsPrecededByAPreposition,
    ItsPrecededByAnUnknownWord,
    TheresNothingBeforeItAndAPrepositionAfterIt,
    ItsFollowedByThatOrWhich,
    ItsActuallyPartOfANounPhrase,
    ItsFollowedByVerb,
}

impl PhrasalVerbAsCompoundNoun {
    fn analyse(
        &self,
        document: &Document,
        i: usize,
        token: &Token,
    ) -> Result<(Vec<char>, Confidence), Why> {
        // It would be handy if there could be a dict flag for nouns which are compounds of phrasal verbs.
        // Instead, let's use a few heuristics.

        // * Can't also be a proper noun or a real verb.
        if token.kind.is_proper_noun() || token.kind.is_verb() {
            return Err(Why::ItsAProperNounOrVerb);
        }
        let noun_chars = document.get_span_content(&token.span);
        // * Can't contain space, hyphen or apostrophe
        if noun_chars.contains(&' ')
            || noun_chars.contains(&'-')
            || noun_chars.contains(&'\'')
            || noun_chars.contains(&'’')
        {
            return Err(Why::ItContainsSpaceHyphenOrApostrophe);
        }

        // * Must not be in the set of known false positives.
        //   These end in a particle after a real verb ("gall on", "drag on",
        //   "left over") yet aren't phrasal verbs miswritten as compound nouns.
        if noun_chars.eq_any_ignore_ascii_case_chars(&[
            &['g', 'a', 'l', 'l', 'o', 'n'],
            &['d', 'r', 'a', 'g', 'o', 'n'],
            &['l', 'e', 'f', 't', 'o', 'v', 'e', 'r'],
        ]) {
            return Err(Why::ItsAKnownFalsePositive);
        }

        // * Must end with the same letters as one of the particles used in phrasal verbs.
        let particle_endings: &[&[char]] = &[
            &['a', 'r', 'o', 'u', 'n', 'd'],
            &['b', 'a', 'c', 'k'],
            &['d', 'o', 'w', 'n'],
            &['i', 'n'],
            &['o', 'n'],
            &['o', 'f', 'f'],
            &['o', 'u', 't'],
            &['o', 'v', 'e', 'r'],
            &['u', 'p'],
        ];

        let found_particle_len = particle_endings
            .iter()
            .find(|ending| noun_chars.ends_with_ignore_ascii_case_chars(ending))
            .map(|ending| ending.len())
            .ok_or(Why::ItDoesntEndWithOneOfTheParticles)?;

        let (verb_part, particle_part) = noun_chars.split_at(noun_chars.len() - found_particle_len);
        let phrasal_verb = [verb_part, &[' '], particle_part].concat();

        // Check if both things are verbs.
        // So far we only have a small number of phrasal verbs in the dictionary.
        let (verb_part_is_verb, phrasal_verb_is_verb) = (
            self.dict
                .get_word_metadata(verb_part)
                .is_some_and(|md| md.verb.is_some()),
            self.dict
                .get_word_metadata(&phrasal_verb)
                .is_some_and(|md| md.verb.is_some()),
        );

        // If neither is a verb, then it's not a phrasal verb
        if !verb_part_is_verb && !phrasal_verb_is_verb {
            return Err(Why::NeitherTheWholeWordNorThePartBeforeTheParticleIsAVerb);
        }

        // Now we know it matches the pattern of a phrasal verb erroneously written as a compound noun.
        // But we have to check if it's an actual compound noun rather than an error.
        // For that we need some heuristics based on the surrounding context.
        // Let's try to get the word before and the word after.
        // To do that we have to get the tokens immediately before and after, which we expect to be whitespace.
        let maybe_prev_tok = document.get_next_word_from_offset(i, -1);
        let maybe_next_tok = document.get_next_word_from_offset(i, 1);

        // If it's in isolation, a compound noun is fine.
        if maybe_prev_tok.is_none() && maybe_next_tok.is_none() {
            return Err(Why::Contextual(Context::ItsInIsolation));
        }

        let confidence = match (phrasal_verb_is_verb, verb_part_is_verb) {
            (true, _) => Confidence::DefinitelyVerb,
            (false, true) => Confidence::PossiblyVerb,
            _ => return Err(Why::Contextual(Context::IHaveNoConfidenceThatItsAVerb)),
        };

        let source = document.get_source();

        if let Some(prev_tok) = maybe_prev_tok {
            if prev_tok.kind.is_adjective() || prev_tok.kind.is_determiner() {
                return Err(Why::Contextual(
                    Context::TheresAnAdjectiveOrDeterminerBeforeIt,
                ));
            }

            // "dictionary lookup" is not a mistake but "couples breakup" is.
            // But "settings plugin" is not.
            if prev_tok.kind.is_noun() && !prev_tok.kind.is_plural_noun()
                || prev_tok.get_ch(source).eq_str("settings")
            {
                return Err(Why::Contextual(Context::ItsPrecededByANonPluralNoun));
            }

            if is_part_of_noun_list(document, i) {
                return Err(Why::Contextual(Context::ItsAnItemInAListOfNouns));
            }

            // If the previous word is (only) a preposition, this word is surely a noun
            if prev_tok.kind.is_preposition() && !prev_tok.get_ch(source).eq_str("to") {
                return Err(Why::Contextual(Context::ItsPrecededByAPreposition));
            }

            // If the previous word is OOV, those are most commonly nouns
            if prev_tok.kind.is_oov() {
                return Err(Why::Contextual(Context::ItsPrecededByAnUnknownWord));
            }
        }

        // A preposition may follow either a verb or a noun.
        // A previous word can help us decide. Without one we can't decide so we won't flag it.
        // ❌ I will never breakup with Gym.
        // ✅ Plugin for text editors.
        // ✅ Plug in for faster performance.
        if maybe_prev_tok.is_none() && maybe_next_tok.is_some_and(|t| t.kind.is_preposition()) {
            return Err(Why::Contextual(
                Context::TheresNothingBeforeItAndAPrepositionAfterIt,
            ));
        }

        // "That" or "which" can follow a noun as relative pronouns.
        if let Some(next_tok) = maybe_next_tok
            && next_tok.kind.is_pronoun()
            && next_tok.get_ch(source).eq_any_ignore_ascii_case_chars(&[
                &['t', 'h', 'a', 't'][..],
                &['w', 'h', 'i', 'c', 'h'][..],
            ])
        {
            return Err(Why::Contextual(Context::ItsFollowedByThatOrWhich));
        }

        // If the compound noun is followed by another noun, check for larger compound nouns.
        if let Some(next_tok) = maybe_next_tok.filter(|tok| tok.kind.is_noun())
            && next_tok.get_ch(source).eq_any_ignore_ascii_case_str(
                [
                    (
                        "backup",
                        &[
                            "file",
                            "images",
                            "link",
                            "links",
                            "location",
                            "plan",
                            "sites",
                            "snapshots",
                        ][..],
                    ),
                    ("callback", &["function", "handlers"][..]),
                    ("layout", &["estimation"][..]),
                    ("markup", &["language", "languages"][..]),
                    ("mouseover", &["hints"][..]),
                    ("playback", &["latency", "speed"][..]),
                    (
                        "plugin",
                        &[
                            "architecture",
                            "classes",
                            "development",
                            "developer",
                            "docs",
                            "ecosystem",
                            "files",
                            "interface",
                            "name",
                            "packages",
                            "suite",
                            "support",
                        ][..],
                    ),
                    ("popup", &["window"][..]),
                    ("rollout", &["logic", "status"][..]),
                    ("startup", &["environments"][..]),
                    ("throwback", &["machine"][..]),
                    ("workout", &["constraints", "preference"][..]),
                ]
                .iter()
                .find(|(prefix, _)| noun_chars.eq_str(prefix))
                .map(|(_, suffixes)| *suffixes)
                .unwrap_or(&[]),
            )
        {
            return Err(Why::Contextual(Context::ItsActuallyPartOfANounPhrase));
        }

        // If the next word is a present or simple past verb form compatible with a 3rd person singular noun
        if let Some(next_tok) = maybe_next_tok
            && (next_tok.kind.is_verb_third_person_singular_present_form()
                || next_tok.kind.is_verb_simple_past_form()
                || next_tok.kind.is_verb_past_form())
            && !next_tok.kind.is_plural_noun()
        {
            return Err(Why::Contextual(Context::ItsFollowedByVerb));
        }

        Ok((phrasal_verb, confidence))
    }
}

impl Linter for PhrasalVerbAsCompoundNoun {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        document
            .iter_noun_indices()
            .filter_map(|i| {
                let token = document.get_token(i).unwrap();

                self.analyse(document, i, token)
                    .ok()
                    .map(|(phrasal_verb, confidence)| Lint {
                        span: token.span,
                        lint_kind: LintKind::WordChoice,
                        suggestions: vec![Suggestion::ReplaceWith(phrasal_verb)],
                        message: match confidence {
                            Confidence::DefinitelyVerb => {
                                "This word should be a phrasal verb, not a compound noun."
                            }
                            Confidence::PossiblyVerb => {
                                "This word might be a phrasal verb rather than a compound noun."
                            }
                        }
                        .to_string(),
                        priority: 63,
                    })
            })
            .collect()
    }

    fn description(&self) -> &str {
        "This rule looks for phrasal verbs written as compound nouns."
    }
}

/// Checks if the current token is part of a list of nouns
fn is_part_of_noun_list(document: &Document, current_index: usize) -> bool {
    // Check for a conjunction before the current word (-1 is whitespace, -2 is the conjunction)
    if !matches!(
        document.get_next_word_from_offset(current_index, -1),
        Some(tok) if tok.kind.is_conjunction()
    ) {
        return false;
    }

    // Check the token sequence before the conjunction
    match document.get_token_offset(current_index, -3) {
        // A comma without the space, assume we're in a list of nouns.
        Some(tok) if tok.kind.is_comma() => true,

        // Whitespace. If the token before that is a noun or a comma, assume we're in a list of nouns.
        Some(ws) if ws.kind.is_whitespace() => {
            document
                .get_token_offset(current_index, -4)
                // `noun and` or `, and`
                .is_some_and(|tok| tok.kind.is_noun() || tok.kind.is_comma())
        }

        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::PhrasalVerbAsCompoundNoun;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn flag_breakup_and_workout() {
        assert_lint_count(
            "I will never breakup with Gym. We just seem to workout.",
            PhrasalVerbAsCompoundNoun::default(),
            2,
        );
    }

    #[test]
    fn correct_breakup_and_workout() {
        assert_suggestion_result(
            "I will never breakup with Gym. We just seem to workout.",
            PhrasalVerbAsCompoundNoun::default(),
            "I will never break up with Gym. We just seem to work out.",
        );
    }

    #[test]
    fn dont_flag_random_words_that_happen_to_end_like_a_particle() {
        assert_no_lints("I like bacon.", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_non_verb_particles() {
        assert_no_lints("non", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn correct_after_i() {
        assert_suggestion_result(
            "I backup",
            PhrasalVerbAsCompoundNoun::default(),
            "I back up",
        );
    }

    #[test]
    fn correct_after_we() {
        assert_suggestion_result(
            "we breakup",
            PhrasalVerbAsCompoundNoun::default(),
            "we break up",
        );
    }

    #[test]
    fn dont_flag_checkin() {
        // It's actually not a noun in English.
        assert_no_lints("checkin", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_cleanup() {
        assert_no_lints("cleanup", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn correct_after_you_lowercase() {
        assert_suggestion_result(
            "you checkout",
            PhrasalVerbAsCompoundNoun::default(),
            "you check out",
        );
    }

    #[test]
    fn correct_after_you_capitalized() {
        assert_suggestion_result(
            "You checkout",
            PhrasalVerbAsCompoundNoun::default(),
            "You check out",
        );
    }

    #[test]
    fn flag_checkout_after_you() {
        assert_lint_count("you checkout", PhrasalVerbAsCompoundNoun::default(), 1);
    }

    #[test]
    fn correct_after_they_lowercase() {
        assert_suggestion_result(
            "they cleanup",
            PhrasalVerbAsCompoundNoun::default(),
            "they clean up",
        );
    }

    #[test]
    fn flag_cleanup_after_they() {
        assert_lint_count("they cleanup", PhrasalVerbAsCompoundNoun::default(), 1);
    }

    #[test]
    fn dont_flag_dictionary_lookup() {
        assert_no_lints("dictionary lookup", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn flag_couples_breakup() {
        assert_lint_count("couples breakup", PhrasalVerbAsCompoundNoun::default(), 1);
    }

    #[test]
    fn dont_flag_gallon() {
        assert_no_lints("gallon", PhrasalVerbAsCompoundNoun::default());
    }

    // Maybe this works by accident because "given" is also an adjective.
    // It should be because "funding" is a noun, but it's a gerund, which makes it also a verb.
    // Still, "given start up" doesn't make sense so maybe this test if fine.
    #[test]
    fn dont_flag_startup_funding() {
        assert_no_lints(
            "Yarvin has actually given startup funding. They hang out and party together",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_huge_markup() {
        assert_no_lints(
            "Sell it back to Russia at a huge markup.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_another_layoff() {
        assert_no_lints(
            "And now just announced another layoff",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    #[ignore = "\"Shakedown\" is a compound noun -- it's part of a comma-separated list with another noun \"threat\"\nBut this is not easy to check for so is not implemented yet."]
    fn dont_flag_a_threat_or_shakedown() {
        assert_no_lints(
            "Just a threat or Shakedown.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_a_flyover() {
        assert_no_lints(
            "if I'm the Brits I'm doing a flyover",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_mafia_style_shakedown() {
        assert_no_lints(
            "Basically it's kind of a mafia style shakedown of Ukraine",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_my_meetup_repository() {
        assert_no_lints(
            "I might have in my Meetup repository",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn ignore_multi_word() {
        assert_no_lints("I like this add-on!", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_list_of_nouns_1298() {
        assert_no_lints(
            "A printable format and layout.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_oov_nvim_plugin_1280() {
        assert_no_lints(
            "This is the nvim plugin for you.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn flag_title_case() {
        assert_lint_count(
            "I Will Never Breakup With Gym. We Just Seem To Workout.",
            PhrasalVerbAsCompoundNoun::default(),
            2,
        );
    }

    #[test]
    fn flag_all_caps() {
        assert_lint_count(
            "I WILL NEVER BREAKUP WITH GYM. WE JUST SEEM TO WORKOUT.",
            PhrasalVerbAsCompoundNoun::default(),
            2,
        );
    }

    #[test]
    fn false_positive_issue_1495() {
        assert_no_lints(
            "Color schemes are available by using the Style Settings plugin.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_thanks_a_lot_linter_description() {
        assert_lint_count(
            "Thanks a lot` is the fixed, widely accepted form, while variants like `thanks lot` or `thanks alot` are non-standard and can jar readers.",
            PhrasalVerbAsCompoundNoun::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_backup_location() {
        assert_no_lints(
            "Backup location: `%APPDATA%\\Cursor\\User\\globalStorage\\backups`",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_backup_plan() {
        assert_no_lints(
            "Every backup plan is unique, based on your risk assessment.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_backup_program() {
        assert_no_lints(
            "restic is a backup program that is fast, efficient and secure",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_backup_solution_or_backup_problems() {
        assert_no_lints(
            "NPBackup is a multiparadigm backup solution which tries to solve two major backup problems",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_backup_utilities_backup_system_or_backup_snapshots() {
        assert_no_lints(
            "GitHub Enterprise Server Backup Utilities is a backup system you install on a separate host, which takes backup snapshots",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_backup_images() {
        assert_no_lints(
            "This App creates and stores backup images of your Nextcloud.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn fix_backup_individual_apps() {
        assert_suggestion_result(
            "It requires root and allows you to backup individual apps and their data.",
            PhrasalVerbAsCompoundNoun::default(),
            "It requires root and allows you to back up individual apps and their data.",
        );
    }

    #[test]
    fn dont_flag_backup_strategy() {
        assert_no_lints(
            "This is for you if you want to quickly set up a backup strategy without much fuss.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_helm_backup_plugin() {
        assert_no_lints("Helm Backup Plugin.", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_callback_function() {
        assert_no_lints(
            "By the time the `setTimeout` callback function was invoked",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_playback_latency() {
        assert_no_lints(
            "Low-Latency HLS is a recently standardized variant of the protocol that allows to greatly reduce playback latency.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_workout_constraints() {
        assert_no_lints("Workout constraints", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_workout_preference() {
        assert_no_lints("Workout preference", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn dont_flag_rollout_status() {
        assert_no_lints(
            "Rollout Status of Latest Image Release",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn font_flag_with_plugin() {
        assert_no_lints(
            "**Xcode** (8.0+, otherwise [with plugin](https://github.com/robertvojta/LigatureXcodePlugin))",
            PhrasalVerbAsCompoundNoun::default(),
        )
    }

    #[test]
    fn dont_flag_and_layout_of_data() {
        assert_no_lints(
            "shape, memory space, and layout of data, while performing the complicated indexing for the user",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_in_noun_list_without_space_after_comma() {
        assert_no_lints(
            "shape, memory space,and layout of data",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_layout_estimation() {
        assert_no_lints(
            "Layout estimation focuses on predicting architectural elements, i.e., walls, doors, and windows, within an indoor scene.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_that() {
        assert_no_lints(
            "plugin that provides way for auto-loading of Golang SDK",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_load_balancing_and_failover() {
        assert_no_lints(
            "resilient mid-tier load balancing and failover",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_for() {
        assert_no_lints(
            "Plugin for text editors and IDEs.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_markup_language() {
        assert_no_lints(
            "Markup language used for websites & web apps.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_ecosystem_or_plugin_development() {
        assert_no_lints(
            "## 🧩 Plugin Ecosystem\n### Plugin Development",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_files_or_plugin_packages() {
        assert_no_lints(
            "plugin files between plugin packages installed with pip must have unique filenames.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_docs() {
        assert_no_lints(
            "building your own plugin: [Plugin Docs]",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_suite() {
        assert_no_lints(
            "An all-in-one digital audio workstation (DAW) and plugin suite.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_hacker_news_throwback_machine() {
        assert_no_lints(
            "| Hacker News Throwback Machine | Shows what was popular on Hacker News on this day in previous years.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_plugin_interface() {
        assert_no_lints("[Plugin interface]", PhrasalVerbAsCompoundNoun::default());
    }

    #[test]
    fn issue_1918() {
        assert_no_lints(
            "Boost your productivity with our JetBrains plugin!",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_pop_up_2217() {
        assert_no_lints(
            "Popup window instead of command line.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn issue_1772() {
        assert_no_lints(
            "By default, only one tile size is instantiated for each data type, math instruction, and layout.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn issue_2369() {
        assert_no_lints(
            "## Plugin developer documentation",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn issue_2505_dont_flag_backup_links() {
        assert_no_lints(
            "seamless switching to one of the backup links ideally happens without packet drops",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn issue_2505_correct_setup_but_dont_flag_backup_link() {
        assert_suggestion_result(
            "How to properly setup a backup link (and have it act like a backup again after stop/start of master link)",
            PhrasalVerbAsCompoundNoun::default(),
            "How to properly set up a backup link (and have it act like a backup again after stop/start of master link)",
        );
    }

    #[test]
    fn dont_flag_meltdown_3591() {
        assert_no_lints(
            "Unfortunately, Meltdown ended up being problematic.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }

    #[test]
    fn dont_flag_leftover_adjective_3707() {
        assert_no_lints(
            "\"Remnants\" typically implies passive, leftover debris.",
            PhrasalVerbAsCompoundNoun::default(),
        );
    }
}
