use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::TokenStringExt;
use crate::document::Document;
use crate::spell::Dictionary;

/// A linter that checks to make sure the first word of each sentence is
/// capitalized in German text.
pub struct GermanSentenceCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSentenceCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }
}

impl<T: Dictionary> Linter for GermanSentenceCapitalization<T> {
    /// A linter that checks to make sure the first word of each sentence is
    /// capitalized.
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            // Allows short, label-like comments in code.
            if paragraph.iter_sentences().count() == 1 {
                let only_sentence = paragraph.iter_sentences().next().unwrap();

                if !only_sentence
                    .iter_chunks()
                    .map(|c| c.iter_words().count())
                    .any(|c| c > 5)
                {
                    continue;
                }
            }

            for sentence in paragraph.iter_sentences() {
                // Basic sentence length check
                if sentence.iter_words().count() < 3 {
                    continue;
                }

                if let Some(first_word) = sentence.first_non_whitespace() {
                    if !first_word.kind.is_word() {
                        continue;
                    }

                    let word_chars = document.get_span_content(&first_word.span);

                    if let Some(first_char) = word_chars.first()
                        && first_char.is_alphabetic()
                        && !first_char.is_uppercase()
                    {
                        let target_span = first_word.span;
                        let mut replacement_chars =
                            document.get_span_content(&target_span).to_vec();
                        if let Some(first_char) = replacement_chars.first_mut() {
                            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                        }

                        lints.push(Lint {
                            span: target_span,
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
                            priority: 30,
                            message: "Sentences must start with a capital letter".to_string(),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks that sentences start with capital letters in German text"
    }
}
