use std::sync::Arc;

use crate::{CharString, Dictionary, Document, FstDictionary};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct SplitWords {
    dict: Arc<FstDictionary>,
}

impl SplitWords {
    pub fn new() -> Self {
        Self {
            dict: FstDictionary::curated(),
        }
    }
}

impl Default for SplitWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for SplitWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let (mut word1, mut word2) = (CharString::new(), CharString::new());

        for w in document.tokens() {
            if !w.kind.is_word() {
                continue;
            }

            if w.span.len() < 2 {
                continue;
            }

            let w_chars = document.get_span_content(w.span);

            if self.dict.contains_word(w_chars) {
                continue;
            }

            let mut found = false;

            for i in 1..w_chars.len() {
                let midpoint = w_chars.len() / 2;
                let midpoint = if i & 1 == 0 {
                    midpoint + i / 2
                } else {
                    midpoint - i / 2
                };

                let first_half = &w_chars[..midpoint];
                let second_half = &w_chars[midpoint..];

                word1.clear();
                word1.extend_from_slice(first_half);
                word2.clear();
                word2.extend_from_slice(second_half);

                if self.dict.contains_exact_word(&word1) && self.dict.contains_exact_word(&word2) {
                    let mut open = word1.clone();
                    open.push(' ');
                    open.extend_from_slice(second_half);

                    lints.push(Lint {
                        span: w.span,
                        lint_kind: LintKind::WordChoice,
                        suggestions: vec![Suggestion::ReplaceWith(open.to_vec())],
                        message: "It seems this is actually two words joined together.".to_owned(),
                        priority: 63,
                    });
                    found = true;
                }

                // The following logic won't be useful unless and until hyphenated words are added to the dictionary

                let mut hyphenated = word1.clone();
                hyphenated.push('-');
                hyphenated.extend_from_slice(second_half);

                if self.dict.contains_exact_word(&hyphenated) {
                    lints.push(Lint {
                        span: w.span,
                        lint_kind: LintKind::WordChoice,
                        suggestions: vec![Suggestion::ReplaceWith(hyphenated.to_vec())],
                        message: "It seems this is actually two words joined together.".to_owned(),
                        priority: 63,
                    });
                    found = true;
                }

                if found {
                    break;
                }
            }
        }
        lints
    }

    fn description(&self) -> &str {
        "Accidentally forgetting a space between words is common. This rule looks for valid words that are joined together without whitespace."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::SplitWords;

    #[test]
    fn heretofore() {
        assert_lint_count(
            "onetwo threefour fivesix seveneight nineten.",
            SplitWords::default(),
            5,
        );
    }

    #[test]
    fn foobar() {
        assert_suggestion_result("moreso", SplitWords::default(), "more so");
    }
}
