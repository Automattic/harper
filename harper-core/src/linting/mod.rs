//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

mod a_part;
mod a_while;
mod addicting;
mod adjective_double_degree;
mod adjective_of_a;
mod after_later;
mod all_intents_and_purposes;
mod allow_to;
mod am_in_the_morning;
mod amounts_for;
mod an_a;
mod and_in;
mod and_the_like;
mod another_thing_coming;
mod another_think_coming;
mod apart_from;
mod ask_no_preposition;
mod avoid_curses;
mod back_in_the_day;
mod be_allowed;
mod behind_the_scenes;
mod best_of_all_time;
mod boring_words;
mod bought;
mod brand_brandish;
mod call_them;
mod cant;
mod capitalize_personal_pronouns;
mod cautionary_tale;
mod change_tack;
mod chock_full;
mod closed_compounds;
mod comma_fixes;
mod compound_nouns;
mod compound_subject_i;
mod confident;
mod correct_number_suffix;
mod criteria_phenomena;
mod cure_for;
mod currency_placement;
mod damages;
mod dashes;
mod day_and_age;
mod despite_it_is;
mod despite_of;
mod determiner_without_noun;
mod did_past;
mod didnt;
mod discourse_markers;
mod disjoint_prefixes;
mod dot_initialisms;
mod double_click;
mod double_modal;
mod ellipsis_length;
mod else_possessive;
mod ever_every;
mod everyday;
mod expand_memory_shorthands;
mod expand_time_shorthands;
mod expr_linter;
mod far_be_it;
mod fascinated_by;
mod feel_fell;
mod few_units_of_time_ago;
mod filler_words;
mod find_fine;
mod first_aid_kit;
mod flesh_out_vs_full_fledged;
mod for_noun;
mod free_predicate;
mod friend_of_me;
mod go_so_far_as_to;
mod go_to_war;
mod good_at;
mod handful;
mod have_pronoun;
mod have_take_a_look;
mod hedging;
mod hello_greeting;
mod hereby;
mod hop_hope;
mod hope_youre;
mod how_to;
mod hyphenate_number_day;
mod i_am_agreement;
mod if_wouldve;
mod in_on_the_cards;
mod inflected_verb_after_to;
mod initialism_linter;
mod initialisms;
mod interested_in;
mod it_is;
mod it_looks_like_that;
mod it_would_be;
mod its_contraction;
mod its_possessive;
mod jealous_of;
mod johns_hopkins;
mod lead_rise_to;
mod left_right_hand;
mod less_worse;
mod let_to_do;
mod lets_confusion;
mod likewise;
mod lint;
mod lint_group;
mod lint_kind;
mod long_sentences;
mod look_down_ones_nose;
mod looking_forward_to;
mod map_phrase_linter;
mod map_phrase_set_linter;
mod mass_nouns;
mod merge_linters;
mod merge_words;
mod missing_preposition;
mod missing_space;
mod missing_to;
mod misspell;
mod mixed_bag;
mod modal_be_adjective;
mod modal_of;
mod modal_seem;
mod months;
mod more_adjective;
mod more_better;
mod most_number;
mod most_of_the_times;
mod multiple_sequential_pronouns;
mod nail_on_the_head;
mod need_to_noun;
mod no_french_spaces;
mod no_longer;
mod no_match_for;
mod no_oxford_comma;
mod nobody;
mod nominal_wants;
mod nor_modal_pronoun;
mod noun_verb_confusion;
mod number_suffix_capitalization;
mod obsess_preposition;
mod of_course;
mod oldest_in_the_book;
mod on_floor;
mod once_or_twice;
mod one_and_the_same;
mod one_of_the_singular;
mod open_compounds;
mod open_the_light;
mod orthographic_consistency;
mod ought_to_be;
mod out_of_date;
mod oxford_comma;
mod oxymorons;
mod phrasal_verb_as_compound_noun;
mod phrase_set_corrections;
mod pique_interest;
mod plural_wrong_word_of_phrase;
mod possessive_noun;
mod possessive_your;
mod progressive_needs_be;
mod pronoun_are;
mod pronoun_contraction;
mod pronoun_inflection_be;
mod pronoun_knew;
mod pronoun_verb_agreement;
mod proper_noun_capitalization_linters;
mod quantifier_needs_of;
mod quantifier_numeral_conflict;
mod quite_quiet;
mod quote_spacing;
mod redundant_acronyms;
mod redundant_additive_adverbs;
mod regionalisms;
mod repeated_words;
mod respond;
mod right_click;
mod roller_skated;
mod safe_to_save;
mod save_to_safe;
mod semicolon_apostrophe;
mod sentence_capitalization;
mod shoot_oneself_in_the_foot;
mod simple_past_to_past_participle;
mod since_duration;
mod single_be;
mod some_without_article;
mod something_is;
mod somewhat_something;
mod soon_to_be;
mod sought_after;
mod spaces;
mod spell_check;
mod spelled_numbers;
mod split_words;
mod subject_pronoun;
mod suggestion;
mod take_a_look_to;
mod take_medicine;
mod take_serious;
mod that_than;
mod that_which;
mod the_how_why;
mod the_my;
mod the_proper_noun_possessive;
mod then_than;
mod theres;
mod theses_these;
mod thing_think;
mod this_type_of_thing;
mod though_thought;
mod throw_away;
mod throw_rubbish;
mod to_adverb;
mod to_two_too;
mod touristic;
mod transposed_space;
mod try_ones_hand_at;
mod unclosed_quotes;
mod update_place_names;
mod use_title_case;
mod verb_to_adjective;
mod very_unique;
mod vice_versa;
mod vicious_loop;
mod was_aloud;
mod way_too_adjective;
mod weir_rules;
mod well_educated;
mod whereas;
mod whom_subject_of_verb;
mod widely_accepted;
mod win_prize;
mod wish_could;
mod wordpress_dotcom;
mod worth_to_do;
mod would_never_have;

pub use expr_linter::{Chunk, ExprLinter};
pub use initialism_linter::InitialismLinter;
pub use lint::Lint;
pub use lint_group::{LintGroup, LintGroupConfig};
pub use lint_kind::LintKind;
pub use map_phrase_linter::MapPhraseLinter;
pub use map_phrase_set_linter::MapPhraseSetLinter;
pub use spell_check::SpellCheck;
pub use suggestion::{Suggestion, SuggestionCollectionExt};

use crate::{Document, LSend, render_markdown};

/// A __stateless__ rule that searches documents for grammatical errors.
///
/// Commonly implemented via [`ExprLinter`].
///
/// See also: [`LintGroup`].
pub trait Linter: LSend {
    /// Analyzes a document and produces zero or more [`Lint`]s.
    /// We pass `self` mutably for caching purposes.
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
    /// A user-facing description of what kinds of grammatical errors this rule looks for.
    /// It is usually shown in settings menus.
    fn description(&self) -> &str;
}

/// A blanket-implemented trait that renders the Markdown description field of a linter to HTML.
pub trait HtmlDescriptionLinter {
    fn description_html(&self) -> String;
}

impl<L: ?Sized> HtmlDescriptionLinter for L
where
    L: Linter,
{
    fn description_html(&self) -> String {
        let desc = self.description();
        render_markdown(desc)
    }
}

pub mod tests {
    use crate::parsers::Markdown;
    use crate::{Document, Span, Token};
    use hashbrown::HashSet;

    /// Extension trait for converting spans of tokens back to their original text
    pub trait SpanVecExt {
        fn to_strings(&self, doc: &Document) -> Vec<String>;
    }

    impl SpanVecExt for Vec<Span<Token>> {
        fn to_strings(&self, doc: &Document) -> Vec<String> {
            self.iter()
                .map(|sp| {
                    doc.get_tokens()[sp.start..sp.end]
                        .iter()
                        .map(|tok| doc.get_span_content_str(&tok.span))
                        .collect::<String>()
                })
                .collect()
        }
    }

    use super::Linter;
    use crate::spell::FstDictionary;

    #[track_caller]
    pub fn assert_no_lints(text: &str, linter: impl Linter) {
        assert_lint_count(text, linter, 0);
    }

    #[track_caller]
    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        // dbg!(&lints);
        if lints.len() != count {
            panic!(
                "Expected \"{text}\" to create {count} lints, but it created {}.",
                lints.len()
            );
        }
    }

    /// Assert the total number of suggestions produced by a [`Linter`], spread across all produced
    /// [`Lint`]s.
    #[track_caller]
    pub fn assert_suggestion_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        assert_eq!(
            lints.iter().map(|l| l.suggestions.len()).sum::<usize>(),
            count
        );
    }

    /// Runs a provided linter on text, applies the first suggestion from each lint
    /// and asserts whether the result is equal to a given value.
    #[track_caller]
    pub fn assert_suggestion_result(text: &str, linter: impl Linter, expected_result: &str) {
        assert_nth_suggestion_result(text, linter, expected_result, 0);
    }

    /// Runs a provided linter on text, applies the nth suggestion from each lint
    /// and asserts whether the result is equal to a given value.
    ///
    /// Note that `n` starts at zero.
    #[track_caller]
    pub fn assert_nth_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        expected_result: &str,
        n: usize,
    ) {
        let transformed_str = transform_nth_str(text, &mut linter, n);

        if transformed_str.as_str() != expected_result {
            panic!("Expected \"{expected_result}\"\n But got \"{transformed_str}\"");
        }

        // Applying the suggestions should fix all the lints.
        assert_lint_count(&transformed_str, linter, 0);
    }

    #[track_caller]
    pub fn assert_top3_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        expected_result: &str,
    ) {
        let zeroth = transform_nth_str(text, &mut linter, 0);
        let first = transform_nth_str(text, &mut linter, 1);
        let second = transform_nth_str(text, &mut linter, 2);

        match (
            zeroth.as_str() == expected_result,
            first.as_str() == expected_result,
            second.as_str() == expected_result,
        ) {
            (true, false, false) => assert_lint_count(&zeroth, linter, 0),
            (false, true, false) => assert_lint_count(&first, linter, 0),
            (false, false, true) => assert_lint_count(&second, linter, 0),
            (false, false, false) => panic!(
                "None of the top 3 suggestions produced the expected result:\n\
                Expected: \"{expected_result}\"\n\
                Got:\n\
                [0]: \"{zeroth}\"\n\
                [1]: \"{first}\"\n\
                [2]: \"{second}\""
            ),
            // I think it's not possible for more than one suggestion to be correct
            _ => {}
        }
    }

    /// Asserts that none of the suggestions from the linter match the given text.
    #[track_caller]
    pub fn assert_not_in_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        bad_suggestion: &str,
    ) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);

        for (i, lint) in lints.iter().enumerate() {
            for (j, suggestion) in lint.suggestions.iter().enumerate() {
                let mut text_chars: Vec<char> = text.chars().collect();
                suggestion.apply(lint.span, &mut text_chars);
                let suggestion_text: String = text_chars.into_iter().collect();

                if suggestion_text == bad_suggestion {
                    panic!(
                        "Found undesired suggestion at lint[{i}].suggestions[{j}]:\n\
                        Expected to not find suggestion: \"{bad_suggestion}\"\n\
                        But found: \"{suggestion_text}\""
                    );
                }
            }
        }
    }

    /// Asserts both that the given text matches the expected good suggestions and that none of the
    /// suggestions are in the bad suggestions list.
    #[track_caller]
    pub fn assert_good_and_bad_suggestions(
        text: &str,
        mut linter: impl Linter,
        good: &[&str],
        bad: &[&str],
    ) {
        let test = Document::new_markdown_default_curated(text);
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

    /// Asserts that the lint's message matches the expected message.
    #[track_caller]
    pub fn assert_lint_message(text: &str, mut linter: impl Linter, expected_message: &str) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);

        // Just check the first lint for now
        if let Some(lint) = lints.first()
            && lint.message != expected_message
        {
            panic!(
                "Expected lint message \"{expected_message}\", but got \"{}\"",
                lint.message
            );
        }
    }

    fn transform_nth_str(text: &str, linter: &mut impl Linter, n: usize) -> String {
        let mut text_chars: Vec<char> = text.chars().collect();

        let mut iter_count = 0;

        loop {
            let test = Document::new_from_vec(
                text_chars.clone().into(),
                &Markdown::default(),
                &FstDictionary::curated(),
            );
            let lints = linter.lint(&test);

            if let Some(lint) = lints.first() {
                if let Some(sug) = lint.suggestions.get(n) {
                    sug.apply(lint.span, &mut text_chars);

                    // let transformed_str: String = text_chars.iter().collect();
                    // dbg!(transformed_str);
                } else {
                    break;
                }
            } else {
                break;
            }

            iter_count += 1;

            if iter_count == 100 {
                break;
            }
        }

        eprintln!("Corrected {iter_count} times.");

        text_chars.iter().collect()
    }
}
