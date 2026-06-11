//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

pub mod english;
pub mod expr_linter;
pub mod lint;
pub mod lint_group;
pub mod lint_kind;
pub mod suggestion;

mod a_some_time;
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
mod as_how;
mod as_to_interrogative;
mod ask_no_preposition;
mod aspire_to;
mod avoid_contractions;
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
mod by_the_book;
mod call_them;
mod cant;
mod capitalize_personal_pronouns;
mod catch_22;
mod cautionary_tale;
mod change_tack;
mod chock_full;
mod close_tight_knit;
mod closed_compounds;
mod code_in_write_in;
mod comma_fixes;
mod complain_as_noun;
mod compound_nouns;
mod compound_subject_i;
mod confident;
mod correct_number_suffix;
mod crave_for;
mod fellow_co_redundancy;
mod in_favour_of_doing;
mod in_on_the_cards;
mod in_time_from_now;
mod inflected_verb_after_to;
mod informal_laughter;
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
mod leaving_in_droves;
mod left_right_hand;
mod less_worse;
mod let_to_do;
mod lets_confusion;
mod likewise;
mod lint;
mod lint_group;
mod lint_kind;
mod long_sentences;
mod long_time_ago;
mod naked_eye;
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
mod out_of_the_window;
mod oxford_comma;
mod oxymorons;
mod pay_for_price;
mod there_is_agreement;

pub use expr_linter::{Chunk, ExprLinter, Sentence};
pub use english::initialism_linter::InitialismLinter;
pub use lint::Lint;
pub use lint_group::FlatConfig;
pub use lint_group::LintGroup;
pub use lint_kind::LintKind;
pub use suggestion::{Suggestion, SuggestionCollectionExt};

use crate::{Document, LSend, render_markdown::render_markdown};

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
                .map(|t| t.span.get_content_string(src))
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

pub mod tests {
    use hashbrown::HashSet;

    use crate::{Document, Linter, Span, Token, languages::LanguageFamily};

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
                        message: "Test linter for 'linting assertion' tests".to_owned(),
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
    pub fn assert_lint_count_plain_english(text: &str, mut linter: impl Linter, count: usize) {
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

    #[track_caller]
    pub fn assert_lint_count(
        text: &str,
        mut linter: impl Linter,
        count: usize,
        language: LanguageFamily,
    ) {
        let test = match language {
            LanguageFamily::English => Document::new_plain_english_curated(text),
            _ => unimplemented!(),
        };
        let lints = linter.lint(&test);
        // dbg!(&lints);
        if lints.len() != count {
            panic!(
                "Expected \"{text}\" to create {count} lints, but it created {}.",
                lints.len()
            );
        }
    }

    // TODO verify many suggestions including the one we want succeeds
    // TODO verify many suggestions but not the one we want fails

    /// Asserts both that the given text matches the expected good suggestions and that none of the
    /// suggestions are in the bad suggestions list.
    /// TODO: Reimplement similar to `search_suggestion_tree`
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

    #[track_caller]
    pub fn assert_no_lints(text: &str, linter: impl Linter, language: LanguageFamily) {
        match language {
            LanguageFamily::English => assert_lint_count_plain_english(text, linter, 0),
            _ => {}
        }
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

    /// Document types for suggestion search testing
    #[derive(Debug, Clone, Copy)]
    pub enum DocumentType {
        PlainEnglish,
        Markdown,
    }

    /// Creates a document of the specified type from character data
    fn create_english_document(chars: &[char], doc_type: DocumentType) -> Document {
        match doc_type {
            DocumentType::PlainEnglish => Document::new_plain_english_curated_chars(chars),
            DocumentType::Markdown => Document::new_markdown_default_curated_chars(chars),
        }
    }

    /// Assert the total number of suggestions produced by a [`Linter`], spread across all produced
    /// [`Lint`]s.
    #[track_caller]
    pub fn assert_suggestion_count(
        text: &str,
        mut linter: impl Linter,
        count: usize,
        language: LanguageFamily,
    ) {
        match language {
            LanguageFamily::English => {
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
            _ => {}
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
        let doctype = DocumentType::PlainEnglish;
        if search_for_suggestion(doctype, text, &mut linter, needle, 0) {
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
    pub fn search_for_suggestion(
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
        let document = create_english_document(&chars, doc_type);
        let lints = linter.lint(&document);

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
}
