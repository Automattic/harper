//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

pub mod a_part;
pub(super) mod a_while;
pub(super) mod addicting;
pub(super) mod adjective_double_degree;
pub(super) mod adjective_of_a;
pub(super) mod after_later;
pub(super) mod all_hell_break_loose;
pub(super) mod all_intents_and_purposes;
pub(super) mod allow_to;
pub(super) mod am_in_the_morning;
pub(super) mod amounts_for;
pub(super) mod an_a;
pub(super) mod and_the_like;
pub(super) mod another_thing_coming;
pub(super) mod another_think_coming;
pub(super) mod apart_from;
pub(super) mod ask_no_preposition;
pub(super) mod aspire_to;
pub(super) mod avoid_curses;
pub(super) mod back_in_the_day;
pub(super) mod be_adjective_confusions;
pub(super) mod be_allowed;
pub(super) mod behind_the_scenes;
pub(super) mod best_of_all_time;
pub(super) mod boring_words;
pub(super) mod bought;
pub(super) mod brand_brandish;
pub(super) mod by_accident;
pub(super) mod call_them;
pub(super) mod cant;
pub(super) mod capitalize_personal_pronouns;
pub(super) mod cautionary_tale;
pub(super) mod change_tack;
pub(super) mod chock_full;
pub(super) mod closed_compounds;
pub(super) mod comma_fixes;
pub(super) mod compound_nouns;
pub(super) mod compound_subject_i;
pub(super) mod confident;
pub(super) mod correct_number_suffix;
pub(super) mod criteria_phenomena;
pub(super) mod cure_for;
pub(super) mod currency_placement;
pub(super) mod damages;
pub(super) mod dashes;
pub(super) mod day_and_age;
pub(super) mod despite_it_is;
pub(super) mod despite_of;
pub(super) mod determiner_without_noun;
pub(super) mod did_past;
pub(super) mod didnt;
pub(super) mod discourse_markers;
pub(super) mod disjoint_prefixes;
pub(super) mod do_mistake;
pub(super) mod dot_initialisms;
pub(super) mod double_click;
pub(super) mod double_modal;
pub(super) mod ellipsis_length;
pub(super) mod else_possessive;
pub(super) mod ever_every;
pub(super) mod everyday;
pub(super) mod except_of;
pub(super) mod expand_memory_shorthands;
pub(super) mod expand_people;
pub(super) mod expand_time_shorthands;
pub(super) mod far_be_it;
pub(super) mod fascinated_by;
pub(super) mod fed_up_with;
pub(super) mod feel_fell;
pub(super) mod few_units_of_time_ago;
pub(super) mod filler_words;
pub(super) mod find_fine;
pub(super) mod first_aid_kit;
pub(super) mod flesh_out_vs_full_fledged;
pub(super) mod for_noun;
pub(super) mod free_predicate;
pub(super) mod friend_of_me;
pub(super) mod go_so_far_as_to;
pub(super) mod go_to_war;
pub(super) mod good_at;
pub(super) mod handful;
pub(super) mod have_pronoun;
pub(super) mod have_take_a_look;
pub(super) mod hedging;
pub(super) mod hello_greeting;
pub(super) mod hereby;
pub(super) mod hop_hope;
pub(super) mod hope_youre;
pub(super) mod how_to;
pub(super) mod hyphenate_number_day;
pub(super) mod i_am_agreement;
pub(super) mod if_wouldve;
pub(super) mod in_on_the_cards;
pub(super) mod in_time_from_now;
pub(super) mod inflected_verb_after_to;
pub(super) mod initialism_linter;
pub(super) mod initialisms;
pub(super) mod interested_in;
pub(super) mod it_is;
pub(super) mod it_looks_like_that;
pub(super) mod it_would_be;
pub(super) mod its_contraction;
pub(super) mod its_possessive;
pub(super) mod jealous_of;
pub(super) mod johns_hopkins;
pub(super) mod lead_rise_to;
pub(super) mod left_right_hand;
pub(super) mod less_worse;
pub(super) mod let_to_do;
pub(super) mod lets_confusion;
pub(super) mod likewise;
pub(super) mod long_sentences;
pub(super) mod look_down_ones_nose;
pub(super) mod looking_forward_to;
pub(super) mod map_phrase_linter;
pub(super) mod map_phrase_set_linter;
pub(super) mod mass_nouns;
pub(super) mod means_a_lot_to;
pub(super) mod merge_linters;
pub(super) mod merge_words;
pub(super) mod missing_preposition;
pub(super) mod missing_space;
pub(super) mod missing_to;
pub(super) mod misspell;
pub(super) mod mixed_bag;
pub(super) mod modal_be_adjective;
pub(super) mod modal_of;
pub(super) mod modal_seem;
pub(super) mod months;
pub(super) mod more_adjective;
pub(super) mod more_better;
pub(super) mod most_number;
pub(super) mod most_of_the_times;
pub(super) mod multiple_frequency_adverbs;
pub(super) mod multiple_sequential_pronouns;
pub(super) mod nail_on_the_head;
pub(super) mod need_to_noun;
pub(super) mod no_french_spaces;
pub(super) mod no_longer;
pub(super) mod no_match_for;
pub(super) mod no_oxford_comma;
pub(super) mod nobody;
pub(super) mod nominal_wants;
pub(super) mod nor_modal_pronoun;
pub(super) mod not_only_inversion;
pub(super) mod noun_verb_confusion;
pub(super) mod number_suffix_capitalization;
pub(super) mod obsess_preposition;
pub(super) mod of_course;
pub(super) mod oldest_in_the_book;
pub(super) mod on_floor;
pub(super) mod once_or_twice;
pub(super) mod one_and_the_same;
pub(super) mod one_of_the_singular;
pub(super) mod open_compounds;
pub(super) mod open_the_light;
pub(super) mod orthographic_consistency;
pub(super) mod ought_to_be;
pub(super) mod out_of_date;
pub(super) mod oxford_comma;
pub(super) mod oxymorons;
pub(super) mod phrasal_verb_as_compound_noun;
pub(super) mod phrase_set_corrections;
pub(super) mod pique_interest;
pub(super) mod plural_decades;
pub(super) mod plural_wrong_word_of_phrase;
pub(super) mod possessive_noun;
pub(super) mod possessive_your;
pub(super) mod progressive_needs_be;
pub(super) mod pronoun_are;
pub(super) mod pronoun_contraction;
pub(super) mod pronoun_inflection_be;
pub(super) mod pronoun_knew;
pub(super) mod pronoun_verb_agreement;
pub(super) mod proper_noun_capitalization_linters;
pub(super) mod quantifier_needs_of;
pub(super) mod quantifier_numeral_conflict;
pub(super) mod quite_quiet;
pub(super) mod quote_spacing;
pub(super) mod reason_for_doing;
pub(super) mod redundant_acronyms;
pub(super) mod redundant_additive_adverbs;
pub(super) mod redundant_progressive_comparative;
pub(super) mod regionalisms;
pub(super) mod regular_irregulars;
pub(super) mod repeated_words;
pub(super) mod respond;
pub(super) mod right_click;
pub(super) mod rise_the_ranks;
pub(super) mod roller_skated;
pub(super) mod safe_to_save;
pub(super) mod save_to_safe;
pub(super) mod sentence_capitalization;
pub(super) mod shoot_oneself_in_the_foot;
pub(super) mod simple_past_to_past_participle;
pub(super) mod since_duration;
pub(super) mod single_be;
pub(super) mod sneaked_snuck;
pub(super) mod some_without_article;
pub(super) mod something_is;
pub(super) mod somewhat_something;
pub(super) mod soon_to_be;
pub(super) mod sought_after;
pub(super) mod spaces;
pub(super) mod spell_check;
pub(super) mod spelled_numbers;
pub(super) mod split_words;
pub(super) mod subject_pronoun;
pub(super) mod take_a_look_to;
pub(super) mod take_medicine;
pub(super) mod take_serious;
pub(super) mod that_than;
pub(super) mod that_which;
pub(super) mod the_how_why;
pub(super) mod the_my;
pub(super) mod the_point_for;
pub(super) mod the_proper_noun_possessive;
pub(super) mod then_than;
pub(super) mod theres;
pub(super) mod theses_these;
pub(super) mod theyre_confusions;
pub(super) mod thing_think;
pub(super) mod this_type_of_thing;
pub(super) mod though_thought;
pub(super) mod thrive_on;
pub(super) mod throw_away;
pub(super) mod throw_rubbish;
pub(super) mod to_adverb;
pub(super) mod to_two_too;
pub(super) mod touristic;
pub(super) mod transposed_space;
pub(super) mod try_ones_hand_at;
pub(super) mod try_ones_luck;
pub(super) mod unclosed_quotes;
pub(super) mod update_place_names;
pub(super) mod use_title_case;
pub(super) mod verb_to_adjective;
pub(super) mod very_unique;
pub(super) mod vice_versa;
pub(super) mod vicious_loop;
pub(super) mod was_aloud;
pub(super) mod way_too_adjective;
pub(super) mod web_scraping;
pub(super) mod weir_rules;
pub(super) mod well_educated;
pub(super) mod were_where;
pub(super) mod whereas;
pub(super) mod whom_subject_of_verb;
pub(super) mod widely_accepted;
pub(super) mod win_prize;
pub(super) mod wish_could;
pub(super) mod wordpress_dotcom;
pub(super) mod worth_to_do;
pub(super) mod would_never_have;
pub(super) mod wrong_apostrophe;

use super::expr_linter::ExprLinter;
use super::lint::Lint;
use super::lint_group::LintGroup;
use super::lint_kind::LintKind;
use super::suggestion::Suggestion;
pub use initialism_linter::InitialismLinter;
pub use map_phrase_linter::MapPhraseLinter;
pub use map_phrase_set_linter::MapPhraseSetLinter;

#[cfg(test)]
pub mod tests {
    use crate::languages::LanguageFamily;
    use crate::linting::Linter;
    use crate::{Document, Span};
    use hashbrown::HashSet;

    /// Extension trait for converting spans of tokens back to their original text
    use crate::linting::tests::{SpanVecExt, assert_no_lints};

    // Special Linter just for testing
    use super::{LintKind, Suggestion};
    use crate::{CharStringExt, Lint, TokenStringExt};

    /// Type alias for many:many error-to-fix mappings used in testing
    /// Each error pattern can map to multiple possible fixes
    pub type TestLinterMap<'a> = &'a [(&'a [&'a str], &'a [&'a str])];

    #[derive(Clone)]
    pub struct TestLinter<'a> {
        map: TestLinterMap<'a>,
    }
    impl<'a> TestLinter<'a> {
        pub fn new(map: TestLinterMap<'a>) -> Self {
            Self { map }
        }
    }
    impl<'a> Linter for TestLinter<'a> {
        fn lint(&mut self, doc: &Document) -> Vec<Lint> {
            let mut corr: Vec<(Span<char>, &[char], &[&str])> = Vec::new();
            for wordtok in doc.iter_words() {
                let wordspan = wordtok.span;
                let word_chars = wordspan.get_content(doc.get_source());
                // Check if word matches any of the patterns in the map
                for (errors, fixes) in self.map {
                    // if any of the errors match, add all of the corrections
                    if errors
                        .iter()
                        .any(|&e| word_chars.eq_ignore_ascii_case_str(e))
                    {
                        corr.push((wordspan, word_chars, fixes))
                    }
                }
            }
            corr.iter()
                .map(|(ws, wch, cstr)| {
                    // Create suggestions for all possible fixes
                    let suggestions: Vec<Suggestion> = cstr
                        .iter()
                        .map(|&suggestion_str| {
                            Suggestion::replace_with_match_case(
                                suggestion_str.chars().collect(),
                                wch.to_owned(),
                            )
                        })
                        .collect();

                    Lint {
                        span: *ws,
                        lint_kind: LintKind::Spelling,
                        suggestions,
                        message: "Test linter for 'linting assertion' tests".to_string(),
                        ..Default::default()
                    }
                })
                .collect()
        }
        fn description(&self) -> &str {
            "Test linter for 'linting assertion' tests"
        }
    }

    // Before the asserts, let's test that the test linter itself has the behaviours we intend
    mod linter_tests {
        use super::linter_tests::{assert_lint_count, assert_suggestion_result};

        #[test]
        fn test_1_to_1_error_to_fix() {
            assert_suggestion_result("bad", TestLinter::new(&[(&["bad"], &["good"])]), "good");
        }

        #[test]
        fn test_1_to_2_error_to_fixes() {
            let linter = TestLinter::new(&[(&["bad"], &["good1", "good2"])]);
            assert_suggestion_result("bad", linter.clone(), "good1");
            assert_suggestion_result("bad", linter, "good2");
        }

        #[test]
        fn test_2_to_1_errors_to_fix() {
            let linter = TestLinter::new(&[(&["bad1", "bad2"], &["good"])]);
            assert_suggestion_result("bad1", linter.clone(), "good");
            assert_suggestion_result("bad2", linter, "good");
        }

        #[test]
        fn test_2_to_2_errors_to_fixes() {
            let linter = TestLinter::new(&[(&["bad1", "bad2"], &["good1", "good2"])]);
            assert_suggestion_result("bad1", linter.clone(), "good1");
            assert_suggestion_result("bad2", linter.clone(), "good2");
            assert_suggestion_result("bad1", linter.clone(), "good2");
            assert_suggestion_result("bad2", linter, "good1");
        }
    }

    #[test]
    fn verify_no_lints() {
        assert_no_lints("hello world", TestLinter::new(&[]), LanguageFamily::English);
    }

    #[test]
    fn verify_1_lint() {
        assert_lint_count(
            "heloo world",
            TestLinter::new(&[(&["heloo"], &["hello"])]),
            1,
        );
    }

    #[test]
    fn verify_2_lints() {
        assert_lint_count(
            "heloo wolrd",
            TestLinter::new(&[(&["heloo"], &["hello"]), (&["wolrd"], &["world"])]),
            2,
        );
    }

    /// Assert the total number of suggestions produced by a [`Linter`], spread across all produced
    /// [`Lint`]s.
    #[track_caller]
    pub fn assert_suggestion_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_plain_english_curated(text);
        let lints = linter.lint(&test);
        eprintln!(
            "{}",
            lints
                .iter()
                .map(|l| l
                    .suggestions
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", "))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert_eq!(
            lints.iter().map(|l| l.suggestions.len()).sum::<usize>(),
            count
        );
    }

    #[test]
    fn verify_no_suggestions() {
        assert_suggestion_count("afjehwkf", TestLinter::new(&[]), 0);
    }

    #[test]
    fn verify_1_suggestion() {
        assert_suggestion_count(
            "dictionery",
            TestLinter::new(&[(&["dictionery"], &["dictionary"])]),
            1,
        );
    }

    #[test]
    fn verify_fix_one_lint() {
        assert_suggestion_result(
            "find the misstake and fix it",
            TestLinter::new(&[(&["misstake"], &["mistake"])]),
            "find the mistake and fix it",
        );
    }

    #[test]
    #[should_panic]
    fn verify_unable_to_fix_one_spanish_lint() {
        assert_suggestion_result("Hay una orrrer", TestLinter::new(&[]), "Hay una error");
    }

    #[test]
    fn verify_fix_two_lints() {
        assert_suggestion_result(
            "find two misstakes and fix theem",
            TestLinter::new(&[(&["misstakes"], &["mistakes"]), (&["theem"], &["them"])]),
            "find two mistakes and fix them",
        );
    }

    // Stress test: multiple errors in one sentence, DFS must find correct suggestion path
    // Note: This test is known to be brittle - it depends on SpellCheck dictionary and
    // suggestion ranking. If it fails after a dictionary update, try different word combinations.
    // Uses common misspellings that have unambiguous correct suggestions in the top 3.
    #[test]
    fn verify_fix_five_typos() {
        assert_suggestion_result(
            "Please recieve teh payment untill thier authorization occured",
            TestLinter::new(&[
                (&["recieve"], &["receive"]),
                (&["teh"], &["the"]),
                (&["untill"], &["until"]),
                (&["thier"], &["their"]),
                (&["occured"], &["occurred"]),
            ]),
            "Please receive the payment until their authorization occurred",
        );
    }

    /// Asserts that none of the suggestions from the linter match the given text.
    #[track_caller]
    pub fn assert_not_in_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        bad_suggestion: &str,
    ) {
        if !search_for_suggestion(
            DocumentType::PlainEnglish,
            text,
            &mut linter,
            bad_suggestion,
            0,
        ) {
            return;
        }

        panic!(
            "A suggestion sequence produced the undesired result.\n\
            Undesired: \"{bad_suggestion}\""
        );
    }

    #[test]
    fn verify_sole_suggestion_is_the_one_we_wanted() {
        assert_not_in_suggestion_result(
            "Baby cats are called kitens",
            TestLinter::new(&[]),
            "Baby cats are called puppies",
        );
    }

    // TODO verify sole suggestion is not the one we wanted fails

    #[test]
    #[should_panic]
    fn verify_sole_suggestion_not_in_result_fails() {
        assert_not_in_suggestion_result(
            "heloo",
            TestLinter::new(&[(&["heloo"], &["hello"])]),
            "hello",
        );
    }

    // TODO verify many suggestions including the one we want succeeds
    // TODO verify many suggestions but not the one we want fails

    /// Asserts both that the given text matches the expected good suggestions and that none of the
    /// suggestions are in the bad suggestions list.
    /// TODO: Reimplement similar to `search_suggestion_tree`
    #[track_caller]
    pub fn assert_good_and_bad_suggestions(
        text: &str,
        mut linter: impl Linter,
        good: &[&str],
        bad: &[&str],
    ) {
        let test = Document::new_plain_english_curated(text);
        let lints = linter.lint(&test);

        let mut unseen_good: HashSet<_> = good.iter().cloned().collect();
        let mut found_bad = Vec::new();
        let mut found_good = Vec::new();

        for (i, lint) in lints.into_iter().enumerate() {
            for (j, suggestion) in lint.suggestions.into_iter().enumerate() {
                let mut text_chars: Vec<char> = text.chars().collect();
                suggestion.apply(lint.span, &mut text_chars);
                let suggestion_text: String = text_chars.into_iter().collect();

                // Check for bad suggestions
                if bad.contains(&&*suggestion_text) {
                    found_bad.push((i, j, suggestion_text.clone()));
                    eprintln!(
                        "  ❌ Found bad suggestion at lint[{i}].suggestions[{j}]: \"{suggestion_text}\""
                    );
                }
                // Check for good suggestions
                else if good.contains(&&*suggestion_text) {
                    found_good.push((i, j, suggestion_text.clone()));
                    eprintln!(
                        "  ✅ Found good suggestion at lint[{i}].suggestions[{j}]: \"{suggestion_text}\""
                    );
                    unseen_good.remove(suggestion_text.as_str());
                }
            }
        }

        // Print summary
        if !found_bad.is_empty() || !unseen_good.is_empty() {
            eprintln!("\n=== Test Summary ===");

            // In the summary section, change these loops:
            if !found_bad.is_empty() {
                eprintln!("\n❌ Found {} bad suggestions:", found_bad.len());
                for (i, j, text) in &found_bad {
                    eprintln!("  - lint[{i}].suggestions[{j}]: \"{text}\"");
                }
            }

            // And for the good suggestions:
            if !unseen_good.is_empty() {
                eprintln!(
                    "\n❌ Missing {} expected good suggestions:",
                    unseen_good.len()
                );
                for text in &unseen_good {
                    eprintln!("  - \"{text}\"");
                }
            }

            eprintln!("\n✅ Found {} good suggestions", found_good.len());
            eprintln!("==================\n");

            if !found_bad.is_empty() || !unseen_good.is_empty() {
                panic!("Test failed - see error output above");
            }
        } else {
            eprintln!(
                "\n✅ All {} good suggestions found, no bad suggestions\n",
                found_good.len()
            );
        }
    }

    // TODO test that having all the good and none of the bad succeeds
    // TODO test that missing one of the good fails
    // TODO test that having one of the bads fails

    #[test]
    #[should_panic]
    fn verify_mutal_corrections_cause_failure() {
        assert_suggestion_result(
            "gooder",
            TestLinter::new(&[(&["gooder"], &["more good"])]),
            "better",
        );
    }

    /// Asserts that the lint's message matches the expected message.
    #[track_caller]
    pub fn assert_lint_message(text: &str, mut linter: impl Linter, expected_message: &str) {
        let test = Document::new_plain_english_curated(text);
        let lints = linter.lint(&test);

        // Just check the first lint for now - TODO
        if let Some(lint) = lints.first()
            && lint.message != expected_message
        {
            panic!(
                "Expected lint message \"{expected_message}\", but got \"{}\"",
                lint.message
            );
        }
    }
}
