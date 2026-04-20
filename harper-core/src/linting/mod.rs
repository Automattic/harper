//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

mod a_part;
mod a_while;
mod addicting;
mod adjective_double_degree;
mod adjective_of_a;
mod after_later;
mod all_hell_break_loose;
mod all_intents_and_purposes;
mod allow_to;
mod am_in_the_morning;
mod amounts_for;
mod an_a;
mod and_the_like;
mod another_thing_coming;
mod another_think_coming;
mod apart_from;
mod arrive_to;
mod ask_no_preposition;
mod aspire_to;
mod avoid_curses;
mod back_in_the_day;
mod be_adjective_confusions;
mod be_allowed;
mod behind_the_scenes;
mod best_of_all_time;
mod boring_words;
mod bought;
mod brand_brandish;
mod by_accident;
mod call_them;
mod cant;
mod capitalize_personal_pronouns;
mod cautionary_tale;
mod change_tack;
mod chock_full;
mod closed_compounds;
mod code_in_write_in;
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
mod do_mistake;
mod dot_initialisms;
mod double_click;
mod double_modal;
mod ellipsis_length;
mod else_possessive;
mod ever_every;
mod everyday;
mod except_of;
mod expand_memory_shorthands;
mod expand_people;
mod expand_time_shorthands;
mod expr_linter;
mod far_be_it;
mod fascinated_by;
mod fed_up_with;
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
mod in_time_from_now;
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
mod means_a_lot_to;
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
mod multiple_frequency_adverbs;
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
mod not_only_inversion;
mod noun_verb_confusion;
mod number_suffix_capitalization;
mod numeric_range_en_dash;
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
mod plural_decades;
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
mod reason_for_doing;
mod redundant_acronyms;
mod redundant_additive_adverbs;
mod redundant_progressive_comparative;
mod regionalisms;
mod regular_irregulars;
mod repeated_words;
mod respond;
mod right_click;
mod rise_the_ranks;
mod roller_skated;
mod safe_to_save;
mod save_to_safe;
mod sentence_capitalization;
mod shoot_oneself_in_the_foot;
mod simple_past_to_past_participle;
mod since_duration;
mod single_be;
mod sneaked_snuck;
mod some_without_article;
mod something_is;
mod somewhat_something;
mod soon_to_be;
mod sought_after;
mod spaces;
mod spell_check;
mod spell_check_with_usernames;
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
mod the_point_for;
mod the_proper_noun_possessive;
mod then_than;
mod theres;
mod theses_these;
mod theyre_confusions;
mod thing_think;
mod this_type_of_thing;
mod though_thought;
mod thrive_on;
mod throw_away;
mod throw_rubbish;
mod to_adverb;
mod to_two_too;
mod touristic;
mod transposed_space;
mod try_ones_hand_at;
mod try_ones_luck;
mod unclosed_quotes;
mod update_place_names;
mod use_ellipsis_character;
mod use_title_case;
mod verb_to_adjective;
mod very_unique;
mod vice_versa;
mod vicious_loop;
mod was_aloud;
mod way_too_adjective;
mod web_scraping;
mod weir_rules;
mod well_educated;
mod were_where;
mod whereas;
mod whom_subject_of_verb;
mod widely_accepted;
mod will_non_lemma;
mod win_prize;
mod wish_could;
mod wordpress_dotcom;
mod worth_to_do;
mod would_never_have;
mod wrong_apostrophe;

pub use expr_linter::{Chunk, ExprLinter};
pub use initialism_linter::InitialismLinter;
pub use lint::Lint;
pub use lint_group::{
    FlatConfig, HumanReadableSetting, HumanReadableStructuredConfig, LintGroup, StructuredConfig,
};
pub use lint_kind::LintKind;
pub use map_phrase_linter::MapPhraseLinter;
pub use map_phrase_set_linter::MapPhraseSetLinter;
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

pub mod debug {
    use crate::Token;

    /// Formats a lint match with surrounding context for debug output.
    ///
    /// The function takes the same `matched_tokens` and `source`, and `context` parameters
    /// passed to `[match_to_lint_with_context]`.
    ///
    /// # Arguments
    /// * `log` - `matched_tokens`
    /// * `ctx` - `context`, or `None` if calling from `[match_to_lint]`
    /// * `src` - `source` from `[match_to_lint]` / `[match_to_lint_with_context]`
    ///
    /// # Returns
    /// A string with ANSI escape codes where:
    /// - Context tokens are dimmed before and after the matched tokens in normal weight.
    /// - Markup and formatting text hidden in whitespace tokens is filtered out.
    pub fn format_lint_match(
        log: &[Token],
        ctx: Option<(&[Token], &[Token])>,
        src: &[char],
    ) -> String {
        let fmt = |tokens: &[Token]| {
            tokens
                .iter()
                .filter(|t| !t.kind.is_unlintable())
                .map(|t| t.get_str(src))
                .collect::<String>()
        };

        if let Some((pro, epi)) = ctx {
            format!(
                "\x1b[2m{}\x1b[0m{}\x1b[2m{}\x1b[0m",
                fmt(pro),
                fmt(log),
                fmt(epi)
            )
        } else {
            fmt(log)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Document, Span, Token, linting::Linter};
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

    // Special Linter just for testing
    use crate::{
        CharStringExt, Lint, TokenStringExt,
        linting::{LintKind, Suggestion},
    };

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
                    if errors.iter().any(|&e| word_chars.eq_str(e)) {
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
        use super::{TestLinter, assert_suggestion_result};

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

    #[track_caller]
    pub fn assert_no_lints(text: &str, linter: impl Linter) {
        assert_lint_count(text, linter, 0);
    }

    #[test]
    fn verify_no_lints() {
        assert_no_lints("hello world", TestLinter::new(&[]));
    }

    #[track_caller]
    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_plain_english_curated(text);
        let lints = linter.lint(&test);
        // dbg!(&lints);
        if lints.len() != count {
            panic!(
                "Expected \"{text}\" to create {count} lints, but it created {}.",
                lints.len()
            );
        }
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

    /// Document types for suggestion search testing
    #[derive(Debug, Clone, Copy)]
    enum DocumentType {
        PlainEnglish,
        Markdown,
    }

    /// Creates a document of the specified type from character data
    fn create_document(chars: &[char], doc_type: DocumentType) -> Document {
        match doc_type {
            DocumentType::PlainEnglish => Document::new_plain_english_curated_chars(chars),
            DocumentType::Markdown => Document::new_markdown_default_curated_chars(chars),
        }
    }

    /// Applies suggestions iteratively until any combination produces the expected result.
    ///
    /// Explores all possible suggestion branches (depth-first search) until finding a path
    /// that produces the expected result. Stops after 100 iterations to prevent infinite loops.
    ///
    /// Use this when you want to verify that *some* suggestion sequence produces the
    /// expected result, without caring which specific suggestions are used.
    ///
    /// See issue #950: https://github.com/Automattic/harper/issues/950
    #[track_caller]
    pub fn assert_suggestion_result(text: &str, mut linter: impl Linter, needle: &str) {
        if search_for_suggestion(DocumentType::PlainEnglish, text, &mut linter, needle, 0) {
            return;
        }

        panic!(
            "No suggestion sequence produced the expected result.\n\
            Expected: \"{needle}\""
        );
    }

    /// DFS implementation using markdown instead of plain English
    #[track_caller]
    pub fn assert_markdown_suggestion_result(text: &str, mut linter: impl Linter, needle: &str) {
        if !search_for_suggestion(DocumentType::Markdown, text, &mut linter, needle, 0) {
            panic!("No suggestion sequence produced the expected result.\nExpected: {needle}");
        }
    }

    /// Recursively searches all suggestion combinations using depth-first search.
    /// Returns true if any path reaches the expected result, false otherwise.
    fn search_for_suggestion(
        doc_type: DocumentType,
        text: &str,
        linter: &mut impl Linter,
        needle: &str,
        depth: usize,
    ) -> bool {
        // Prevent infinite recursion (e.g. cycles in suggestions)
        if depth > 100 {
            eprintln!("⚠️  Reached depth limit (100)");
            return false;
        }

        // Check if we've reached the expected result
        if text == needle {
            return true;
        }

        // Lint current text and try each suggestion branch
        let chars: Vec<char> = text.chars().collect();
        let document = create_document(&chars, doc_type);
        let mut lints = linter.lint(&document);
        lints.sort_by_key(|l| l.priority);

        if let Some(lint) = lints.first() {
            for sug in lint.suggestions.iter() {
                let mut chars_copy = chars.clone();
                sug.apply(lint.span, &mut chars_copy);
                let next: String = chars_copy.iter().collect();

                // Recursively search this branch
                if search_for_suggestion(doc_type, &next, linter, needle, depth + 1) {
                    return true;
                }
            }
        }

        false
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

            if !found_bad.is_empty() {
                eprintln!("\n❌ Found {} bad suggestions:", found_bad.len());
                for (i, j, text) in &found_bad {
                    eprintln!("  - lint[{i}].suggestions[{j}]: \"{text}\"");
                }
            }

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
