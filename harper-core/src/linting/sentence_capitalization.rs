use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::Dictionary;
use crate::{Token, TokenKind, TokenStringExt};

pub struct SentenceCapitalization<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> SentenceCapitalization<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }
}

impl<T: Dictionary> Linter for SentenceCapitalization<T> {
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
                if !is_full_sentence(sentence) {
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
                        if let Some(canonical_spelling) =
                            self.dictionary.get_correct_capitalization_of(word_chars)
                        {
                            // Skip if it's a proper noun or contains uppercase letters before a separator
                            if first_word.kind.is_proper_noun() {
                                continue;
                            }

                            // Check for uppercase letters in the rest of the word before any separators
                            if canonical_spelling
                                .iter()
                                .skip(1)
                                .take_while(|&c| !c.is_whitespace() && *c != '-' && *c != '\'')
                                .any(|&c| c.is_uppercase())
                            {
                                continue;
                            }
                        }

                        let target_span = first_word.span;
                        let mut replacement_chars =
                            document.get_span_content(&target_span).to_vec();
                        replacement_chars[0] = replacement_chars[0].to_ascii_uppercase();

                        lints.push(Lint {
                            span: target_span,
                            lint_kind: LintKind::Capitalization,
                            suggestions: vec![Suggestion::ReplaceWith(replacement_chars)],
                            priority: 31,
                            message: "This sentence does not start with a capital letter"
                                .to_string(),
                        });
                    }
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "The opening word of a sentence should almost always be capitalized."
    }
}

fn is_full_sentence(toks: &[Token]) -> bool {
    let mut has_nominal = false;
    let mut has_verb = false;

    for tok in toks {
        if let TokenKind::Word(Some(metadata)) = &tok.kind {
            if metadata.is_nominal() {
                has_nominal = true;
            }

            if metadata.is_verb() {
                has_verb = true;
            }
        }
    }

    has_nominal && has_verb
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_lint_count;
    use super::SentenceCapitalization;
    use crate::spell::FstDictionary;

    #[test]
    fn catches_basic() {
        assert_lint_count(
            "there is no way she is not guilty.",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn no_period() {
        assert_lint_count(
            "there is no way she is not guilty",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn two_sentence() {
        assert_lint_count(
            "i have complete conviction in this. she is absolutely guilty",
            SentenceCapitalization::new(FstDictionary::curated()),
            2,
        )
    }

    #[test]
    fn start_with_number() {
        assert_lint_count(
            "53 is the length of the longest word.",
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        );
    }

    #[test]
    fn ignores_unlintable() {
        assert_lint_count(
            "[`misspelled_word`] is assumed to be quite small (n < 100). ",
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        )
    }

    #[test]
    fn unfazed_unlintable() {
        assert_lint_count(
            "the linter should not be affected by `this` unlintable.",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn unfazed_ellipsis() {
        assert_lint_count(
            "the linter should not be affected by... that ellipsis.",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn unfazed_comma() {
        assert_lint_count(
            "the linter should not be affected by, that comma.",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn issue_228_allows_labels() {
        assert_lint_count(
            "python lsp (fork of pyright)",
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        )
    }

    #[test]
    fn allow_camel_case_trademarks() {
        // Some words are marked as proper nouns in `dictionary.dict` but are lower camel case.
        assert_lint_count(
            "macOS 16 could be called something like Redwood or Shasta",
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        )
    }

    #[test]
    #[ignore = "This can't work because currently hyphens are not included in tokenized words\nalthough they are now permitted in `dictionary.dict`"]
    fn uppercase_unamerican_at_start() {
        assert_lint_count(
            "un-American starts with a lowercase letter and contains an uppercase letter, but is not a proper noun or trademark.",
            SentenceCapitalization::new(FstDictionary::curated()),
            1,
        )
    }

    #[test]
    fn allow_lowercase_proper_nouns() {
        // A very few words are marked as proper nouns even though they're all lowercase.
        // https://css-tricks.com/start-sentence-npm/
        assert_lint_count(
            concat!(
                "npm is the world's largest software registry. Open source developers from every ",
                "continent use npm to share and borrow packages, and many organizations use npm to ",
                "manage private development as well."
            ),
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        )
    }

    #[test]
    fn allow_lower_camel_case_non_proper_nouns() {
        // A very few words are not considered proper nouns but still start with a lowercase letter that shouldn't be uppercased at the start of a sentence.
        assert_lint_count(
            "mRNA is synthesized from the coding sequence of a gene during the transcriptional process.",
            SentenceCapitalization::new(FstDictionary::curated()),
            0,
        )
    }
}
