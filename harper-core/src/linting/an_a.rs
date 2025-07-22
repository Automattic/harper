use itertools::Itertools;

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, TokenStringExt, starts_with_vowel::starts_with_vowel};

#[derive(Debug, Default)]
pub struct AnA;

impl Linter for AnA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                // [`TokenKind::Unlintable`] might have semantic meaning.
                if chunk[first_idx..second_idx].iter_unlintables().count() > 0
                    || chunk[first_idx + 1..second_idx]
                        .iter_word_like_indices()
                        .count()
                        > 0
                {
                    continue;
                }

                let first = &chunk[first_idx];
                let second = &chunk[second_idx];

                let chars_first = document.get_span_content(&first.span);
                let chars_second = document.get_span_content(&second.span);
                // Break the second word on hyphens for this lint.
                // Example: "An ML-based" is an acceptable noun phrase.
                let chars_second = chars_second
                    .split(|c| !c.is_alphanumeric())
                    .next()
                    .unwrap_or(chars_second);

                let is_a_an = match chars_first {
                    ['a'] => Some(true),
                    ['A'] => Some(true),
                    ['a', 'n'] => Some(false),
                    ['A', 'n'] => Some(false),
                    _ => None,
                };

                let Some(a_an) = is_a_an else {
                    continue;
                };

                let should_be_a_an = !starts_with_vowel(chars_second);

                if a_an != should_be_a_an {
                    let replacement = match a_an {
                        true => vec!['a', 'n'],
                        false => vec!['a'],
                    };

                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement,
                            chars_first,
                        )],
                        message: "Incorrect indefinite article.".to_string(),
                        priority: 31,
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "A rule that looks for incorrect indefinite articles. For example, `this is an mule` would be flagged as incorrect."
    }
}

#[cfg(test)]
mod tests {
    use super::AnA;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn detects_html_as_vowel() {
        assert_lint_count("Here is a HTML document.", AnA, 1);
    }

    #[test]
    fn detects_llm_as_vowel() {
        assert_lint_count("Here is a LLM document.", AnA, 1);
    }

    #[test]
    fn detects_llm_hyphen_as_vowel() {
        assert_lint_count("Here is a LLM-based system.", AnA, 1);
    }

    #[test]
    fn capitalized_fourier() {
        assert_lint_count("Then, perform a Fourier transform.", AnA, 0);
    }

    #[test]
    fn once_over() {
        assert_lint_count("give this a once-over.", AnA, 0);
    }

    #[test]
    fn issue_196() {
        assert_lint_count("This is formatted as an `ext4` file system.", AnA, 0);
    }

    #[test]
    fn allows_lowercase_vowels() {
        assert_lint_count("not an error", AnA, 0);
    }

    #[test]
    fn allows_lowercase_consonants() {
        assert_lint_count("not a crash", AnA, 0);
    }

    #[test]
    fn disallows_lowercase_vowels() {
        assert_lint_count("not a error", AnA, 1);
    }

    #[test]
    fn disallows_lowercase_consonants() {
        assert_lint_count("not an crash", AnA, 1);
    }

    #[test]
    fn allows_uppercase_vowels() {
        assert_lint_count("not an Error", AnA, 0);
    }

    #[test]
    fn allows_uppercase_consonants() {
        assert_lint_count("not a Crash", AnA, 0);
    }

    #[test]
    fn disallows_uppercase_vowels() {
        assert_lint_count("not a Error", AnA, 1);
    }

    #[test]
    fn disallows_uppercase_consonants() {
        assert_lint_count("not an Crash", AnA, 1);
    }

    #[test]
    fn disallows_a_interface() {
        assert_lint_count(
            "A interface for an object that can perform linting actions.",
            AnA,
            1,
        );
    }

    #[test]
    fn allow_issue_751() {
        assert_lint_count("He got a 52% approval rating.", AnA, 0);
    }
}
