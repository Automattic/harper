pub mod english;
pub mod expr_linter;
pub mod lint;
pub mod lint_group;
pub mod lint_kind;
mod pooled_linter;
pub mod portuguese;
pub mod suggestion;

use crate::{
    Document, LSend,
    linting::english::{initialism_linter, map_phrase_linter, map_phrase_set_linter},
    render_markdown::render_markdown,
};

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
        let test = Document::new_plain_curated(text, language);
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
        assert_lint_count(text, linter, 0, language)
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

    /// Document types for suggestion search testing
    #[derive(Debug, Clone, Copy)]
    pub enum DocumentType {
        PlainEnglish,
        PlainPortuguese,
        Markdown,
    }

    /// The plain-text document type used to parse the given language.
    fn plain_document_type(language: LanguageFamily) -> DocumentType {
        match language {
            LanguageFamily::English => DocumentType::PlainEnglish,
            LanguageFamily::Portuguese => DocumentType::PlainPortuguese,
        }
    }

    /// Creates a document of the specified type from character data
    fn create_document(chars: &[char], doc_type: DocumentType) -> Document {
        match doc_type {
            DocumentType::PlainEnglish => Document::new_plain_english_curated_chars(chars),
            DocumentType::PlainPortuguese => Document::new_plain_portuguese_curated_chars(chars),
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
        let test = Document::new_plain_curated(text, language);
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
    pub fn assert_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        needle: &str,
        language: LanguageFamily,
    ) {
        if search_for_suggestion(
            plain_document_type(language),
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
    pub fn assert_first_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        needle: &str,
        language: LanguageFamily,
    ) {
        if search_for_suggestion(
            plain_document_type(language),
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
    pub enum SearchStrategy {
        /// Explores all suggestion sequences (the current default behavioral standard).
        AllSuggestions,
        /// Only explores the very first suggestion path (for validating primary/best fixes).
        FirstSuggestionOnly,
    }

    /// Recursively searches all suggestion combinations using depth-first search.
    /// Returns true if any path reaches the expected result, false otherwise.
    pub fn search_for_suggestion(
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
}
