pub mod english;
pub mod expr_linter;
pub mod lint;
pub mod lint_group;
pub mod lint_kind;
pub mod portuguese;
pub mod suggestion;

pub use expr_linter::ExprLinter;
pub use lint::Lint;
pub use lint_group::LintGroup;
pub use lint_kind::LintKind;
pub use suggestion::Suggestion;

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
    pub fn assert_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        needle: &str,
        language: LanguageFamily,
    ) {
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
