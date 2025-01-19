use crate::{
    patterns::{Pattern, SequencePattern, WordSet},
    Document, Token, TokenStringExt,
};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct OxfordComma {
    pattern: SequencePattern,
}

impl OxfordComma {
    pub fn new() -> Self {
        Self {
            pattern: SequencePattern::default()
                .then_one_or_more(Box::new(
                    SequencePattern::default()
                        .then_noun_phrase()
                        .then_comma()
                        .then_whitespace(),
                ))
                .then_noun_phrase()
                .then_whitespace()
                .then(Box::new(WordSet::all(&["and", "or"])))
                .then_whitespace()
                .then_noun_phrase(),
        }
    }

    fn match_to_lint(&self, matched_toks: &[Token], _source: &[char]) -> Lint {
        let conj_index = matched_toks.last_conjunction_index().unwrap();
        let offender = matched_toks[conj_index - 2];

        Lint {
            span: offender.span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::InsertAfter(vec![','])],
            message: "An Oxford comma is necessary here.".to_owned(),
            priority: 31,
        }
    }
}

impl Default for OxfordComma {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for OxfordComma {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let mut tok_cursor = 0;

            loop {
                if tok_cursor >= sentence.len() {
                    break;
                }

                let match_len = self
                    .pattern
                    .matches(&sentence[tok_cursor..], document.get_source());

                if match_len != 0 {
                    let lint = self.match_to_lint(
                        &sentence[tok_cursor..tok_cursor + match_len],
                        document.get_source(),
                    );

                    lints.push(lint);
                    tok_cursor += match_len;
                } else {
                    tok_cursor += 1;
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "The Oxford comma is one of the more controversial rules in common use today. Enabling this lint checks that there is a comma before `and` or `or` when listing out more than two ideas."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::OxfordComma;

    #[test]
    fn fruits() {
        assert_lint_count("An apple, a banana and a pear", OxfordComma::default(), 1);
    }

    #[test]
    fn people() {
        assert_suggestion_result(
            "Nancy, Steve and Carl are going to the coffee shop.",
            OxfordComma::default(),
            "Nancy, Steve, and Carl are going to the coffee shop.",
        );
    }

    #[test]
    fn places() {
        assert_suggestion_result(
            "I've always wanted to visit Paris, Tokyo and Rome.",
            OxfordComma::default(),
            "I've always wanted to visit Paris, Tokyo, and Rome.",
        );
    }

    #[test]
    fn foods() {
        assert_suggestion_result(
            "My favorite foods are pizza, sushi, tacos and burgers.",
            OxfordComma::default(),
            "My favorite foods are pizza, sushi, tacos, and burgers.",
        );
    }

    #[test]
    fn allows_clean_music() {
        assert_lint_count(
            "I enjoy listening to pop music, rock, hip-hop, electronic dance, and classical music.",
            OxfordComma::default(),
            0,
        );
    }

    #[test]
    fn allows_clean_nations() {
        assert_lint_count("The team consists of players from different countries: France, Germany, Italy, and Spain.", OxfordComma::default(), 0);
    }

    #[test]
    fn or_writing() {
        assert_suggestion_result("Harper can be a lifesaver when writing technical documents, emails or other formal forms of communication.", OxfordComma::default(), "Harper can be a lifesaver when writing technical documents, emails, or other formal forms of communication.",);
    }

    #[test]
    fn sports() {
        assert_suggestion_result(
            "They enjoy playing soccer, basketball or tennis.",
            OxfordComma::default(),
            "They enjoy playing soccer, basketball, or tennis.",
        );
    }
}
