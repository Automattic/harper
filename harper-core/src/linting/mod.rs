//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

pub(crate) use crate::lang::en::linting as en;
pub(crate) mod expr_linter;
mod initialism_linter;
mod lint;
mod lint_group;
mod lint_kind;
mod map_phrase_linter;
mod map_phrase_set_linter;
pub(crate) mod merge_linters;
pub(crate) mod pooled_linter;
pub(crate) mod spell_check;
mod suggestion;

pub use expr_linter::{Chunk, ExprLinter, Sentence};
pub use initialism_linter::InitialismLinter;
pub use lint::Lint;
pub use lint_group::{
    FlatConfig, HumanReadableSetting, HumanReadableStructuredConfig, LintGroup, StructuredConfig,
};
pub use lint_kind::LintKind;
pub use map_phrase_linter::MapPhraseLinter;
pub use map_phrase_set_linter::MapPhraseSetLinter;
#[cfg(test)]
pub(crate) use pooled_linter::PooledLinter;
#[cfg(test)]
pub(crate) use pooled_linter::for_tests::create_test_pool;
pub use suggestion::{Suggestion, SuggestionCollectionExt};

use crate::{Document, LSend, render_markdown};

/// Maximum number of sequential lint suggestions explored when searching for a
/// transformation path (for example in Weir rule tests or linting test helpers).
///
/// This is a compile-time limit that prevents unbounded search when suggestion
/// application cycles or deep multi-step fixes are involved.
pub const MAX_SUGGESTION_TRANSFORMATION_DEPTH: usize = 100;

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
                "Expected \"{text}\" to create {count} lints, but it created {}.\n{}",
                lints.len(),
                lints
                    .iter()
                    .map(|l| l.message.clone())
                    .collect::<Vec<_>>()
                    .join("\n")
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
    /// that produces the expected result. Stops after
    /// [`MAX_SUGGESTION_TRANSFORMATION_DEPTH`] iterations to prevent infinite loops.
    ///
    /// Use this when you want to verify that *some* suggestion sequence produces the
    /// expected result, without caring which specific suggestions are used.
    ///
    /// See issue #950: https://github.com/Automattic/harper/issues/950
    #[track_caller]
    pub fn assert_suggestion_result(text: &str, mut linter: impl Linter, needle: &str) {
        if search_for_suggestion(
            DocumentType::PlainEnglish,
            text,
            &mut linter,
            needle,
            0,
            SearchStrategy::AllSuggestions,
        ) {
            return;
        }

        panic!(
            "No suggestion sequence produced the expected result.\n\
            Expected: \"{needle}\""
        );
    }

    /// Use this when you want to verify that the first suggestion is the most likely correct fix.
    /// Handy for linters which offer multiple suggestions but prioritize their best guess.
    #[track_caller]
    pub fn assert_first_suggestion_result(text: &str, mut linter: impl Linter, needle: &str) {
        if search_for_suggestion(
            DocumentType::PlainEnglish,
            text,
            &mut linter,
            needle,
            0,
            SearchStrategy::FirstSuggestionOnly,
        ) {
            return;
        }

        panic!(
            "The primary suggestion sequence failed to produce the expected result.\n\
            Expected: \"{needle}\""
        );
    }

    /// DFS implementation using markdown instead of plain English
    #[track_caller]
    pub fn assert_markdown_suggestion_result(text: &str, mut linter: impl Linter, needle: &str) {
        if !search_for_suggestion(
            DocumentType::Markdown,
            text,
            &mut linter,
            needle,
            0,
            SearchStrategy::AllSuggestions,
        ) {
            panic!("No suggestion sequence produced the expected result.\nExpected: {needle}");
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    enum SearchStrategy {
        /// Explores all suggestion sequences (the current default behavioral standard).
        AllSuggestions,
        /// Only explores the very first suggestion path (for validating primary/best fixes).
        FirstSuggestionOnly,
    }

    /// Recursively searches all suggestion combinations using depth-first search.
    /// Returns true if any path reaches the expected result, false otherwise.
    fn search_for_suggestion(
        doc_type: DocumentType,
        text: &str,
        linter: &mut impl Linter,
        needle: &str,
        depth: usize,
        strategy: SearchStrategy,
    ) -> bool {
        // Prevent infinite recursion (e.g. cycles in suggestions)
        if depth > super::MAX_SUGGESTION_TRANSFORMATION_DEPTH {
            eprintln!(
                "⚠️  Reached depth limit ({})",
                super::MAX_SUGGESTION_TRANSFORMATION_DEPTH
            );
            return false;
        }

        // Check if we've reached the expected result
        if text == needle {
            // When tests are made via cut & paste it's easy to miss editing some of the corrections
            // and the test will silently pass
            if depth == 0 {
                eprintln!("⚠️  Input and expected are both '{needle}' - is the test correct?");
            }
            return true;
        } else if cfg!(debug_assertions) {
            eprintln!(" 🔎 Checking... \"{text}\"");
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
                if search_for_suggestion(doc_type, &next, linter, needle, depth + 1, strategy) {
                    return true;
                }

                // If we're only checking the first suggestion, stop after the first one
                if strategy == SearchStrategy::FirstSuggestionOnly {
                    break;
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
            SearchStrategy::AllSuggestions,
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
                } else {
                    eprintln!(
                        "  ⚠️  Found unexpected suggestion at lint[{i}].suggestions[{j}]: \"{suggestion_text}\""
                    );
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
        match lints.first() {
            Some(lint) => {
                assert_eq!(
                    lint.message, expected_message,
                    "Expected lint message \"{expected_message}\", but got \"{}\"",
                    lint.message
                );
            }
            None => panic!("Expected lint message \"{expected_message}\", but no lints were found"),
        }
    }
}
