use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::{Token, TokenStringExt};

/// A linter that checks for words that are capitalized but shouldn't be
/// (i.e., not at the start of a sentence/heading and not proper nouns).
pub struct SentenceCasing;

impl Default for SentenceCasing {
    fn default() -> Self {
        Self
    }
}

impl SentenceCasing {
    /// Check a sequence of tokens for incorrect capitalization.
    /// `first_word_idx` is the index of the first word that should be capitalized.
    fn check_tokens(&self, tokens: &[Token], document: &Document, lints: &mut Vec<Lint>) {
        // Get the index of the first word in the sequence
        let first_word_idx = tokens.iter().position(|t| t.kind.is_word());

        let Some(first_word_idx) = first_word_idx else {
            return;
        };

        // Check all words after the first one
        for (idx, token) in tokens.iter().enumerate() {
            // Skip the first word (it should be capitalized)
            if idx <= first_word_idx {
                continue;
            }

            // Only check actual words
            if !token.kind.is_word() {
                continue;
            }

            // Check if the word is capitalized
            let word_chars = document.get_span_content(&token.span);
            let Some(first_char) = word_chars.first() else {
                continue;
            };

            // Skip if not capitalized
            if !first_char.is_uppercase() {
                continue;
            }

            // Skip proper nouns - these should be capitalized
            if token.kind.is_proper_noun() {
                continue;
            }

            // Skip words that are all uppercase (likely acronyms/initialisms)
            if word_chars
                .iter()
                .all(|c| !c.is_alphabetic() || c.is_uppercase())
            {
                continue;
            }

            // Skip words after a colon (might be starting a new clause)
            if let Some(prev_non_ws) = tokens[..idx].iter().rev().find(|t| !t.kind.is_whitespace())
                && prev_non_ws.kind.is_punctuation()
            {
                let prev_chars = document.get_span_content(&prev_non_ws.span);
                if prev_chars == [':'] {
                    continue;
                }
            }

            // Skip single-letter capitalizations (often used for proper context like "Plan A")
            if word_chars.len() == 1 {
                continue;
            }

            // Skip words after opening quotes (might be a quoted sentence start)
            if let Some(prev_non_ws) = tokens[..idx].iter().rev().find(|t| !t.kind.is_whitespace())
                && prev_non_ws.kind.is_quote()
            {
                continue;
            }

            // Check if this word follows a sentence terminator within the same sequence
            // (This handles cases where parsing might not have split sentences correctly)
            let has_terminator_before = tokens[first_word_idx + 1..idx]
                .iter()
                .any(|t| t.kind.is_sentence_terminator());

            if has_terminator_before {
                continue;
            }

            // Create the lowercase suggestion
            let mut replacement_chars = word_chars.to_vec();
            replacement_chars[0] = replacement_chars[0].to_ascii_lowercase();

            lints.push(Lint {
                span: token.span,
                lint_kind: LintKind::Capitalization,
                suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
                priority: 63,
                message: "This word is capitalized but does not appear to be a proper noun. Consider using lowercase.".to_string(),
            });
        }
    }
}

impl Linter for SentenceCasing {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // Check headings
        for heading in document.iter_headings() {
            self.check_tokens(heading, document, &mut lints);
        }

        // Check regular sentences (but skip those in headings)
        for paragraph in document.iter_paragraphs() {
            // Skip paragraphs that are headings (they're already checked above)
            if paragraph.iter().any(|t| t.kind.is_heading_start()) {
                continue;
            }

            for sentence in paragraph.iter_sentences() {
                self.check_tokens(sentence, document, &mut lints);
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Flags words that are capitalized mid-sentence or mid-heading but are not proper nouns."
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{assert_lint_count, assert_suggestion_result};
    use super::SentenceCasing;

    #[test]
    fn catches_mid_sentence_capital() {
        assert_lint_count(
            "The quick Brown fox jumps over the lazy dog.",
            SentenceCasing,
            1,
        );
    }

    #[test]
    fn allows_proper_nouns() {
        assert_lint_count("I visited Paris last summer.", SentenceCasing, 0);
    }

    #[test]
    fn allows_sentence_start() {
        assert_lint_count("The fox is quick. The dog is lazy.", SentenceCasing, 0);
    }

    #[test]
    fn allows_acronyms() {
        assert_lint_count("The NASA mission was successful.", SentenceCasing, 0);
    }

    #[test]
    fn allows_after_colon() {
        assert_lint_count("Here is the answer: True or false.", SentenceCasing, 0);
    }

    #[test]
    fn allows_single_letter() {
        assert_lint_count("This is plan A for the mission.", SentenceCasing, 0);
    }

    #[test]
    fn fixes_capitalization() {
        assert_suggestion_result(
            "The quick Brown fox.",
            SentenceCasing,
            "The quick brown fox.",
        );
    }

    #[test]
    fn allows_names() {
        assert_lint_count("I talked to John yesterday.", SentenceCasing, 0);
    }

    #[test]
    fn multiple_errors() {
        assert_lint_count(
            "The Quick Brown Fox jumps over the Lazy Dog.",
            SentenceCasing,
            4,
        );
    }

    #[test]
    fn allows_quoted_start() {
        assert_lint_count("She said \"Hello there\" to him.", SentenceCasing, 0);
    }

    // Heading tests

    #[test]
    fn catches_heading_mid_word_capital() {
        // Markdown heading with incorrect capitalization
        assert_lint_count("# The Quick Brown Fox", SentenceCasing, 3);
    }

    #[test]
    fn allows_heading_proper_nouns() {
        assert_lint_count("# A trip to Paris", SentenceCasing, 0);
    }

    #[test]
    fn allows_heading_start_capital() {
        assert_lint_count("# Introduction to the topic", SentenceCasing, 0);
    }

    #[test]
    fn fixes_heading_capitalization() {
        assert_suggestion_result("# The Quick fox", SentenceCasing, "# The quick fox");
    }

    #[test]
    fn heading_with_acronym() {
        assert_lint_count("# Working with NASA and SpaceX", SentenceCasing, 0);
    }
}
