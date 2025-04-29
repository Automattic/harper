use crate::{
    Lrc, Token, TokenStringExt,
    linting::Linter,
    patterns::{OwnedPatternExt, Pattern, SequencePattern, WordSet},
};

use super::{super::Lint, LintKind, Suggestion};

pub struct OxfordComma {
    pattern: Box<dyn Pattern>,
}

impl Default for OxfordComma {
    fn default() -> Self {
        let item = Lrc::new(
            SequencePattern::default()
                .then_determiner()
                .then_whitespace()
                .then_nominal()
                .or(Box::new(SequencePattern::default().then_nominal())),
        );

        let item_chunk = SequencePattern::default()
            .then(item.clone())
            .then_comma()
            .then_whitespace();

        let pattern = SequencePattern::default()
            .then_one_or_more(item_chunk)
            .then(item.clone())
            .then_whitespace()
            .then(WordSet::new(&["and", "or", "nor"]))
            .then_whitespace()
            .then(item.clone());

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl OxfordComma {
    fn match_to_lint(&self, matched_toks: &[Token], _source: &[char]) -> Option<Lint> {
        let conj_index = matched_toks.last_conjunction_index()?;
        let offender = &matched_toks[conj_index - 2];

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::InsertAfter(vec![','])],
            message: "An Oxford comma is necessary here.".to_owned(),
            priority: 31,
        })
    }
}

impl Linter for OxfordComma {
    fn lint(&mut self, document: &crate::Document) -> Vec<crate::linting::Lint> {
        let mut lints = Vec::new();
        for sentence in document.iter_sentences() {
            let mut tok_cursor = 0;

            let mut words = sentence
                .iter_words()
                .filter_map(|v| v.kind.as_word())
                .flatten();

            if let (Some(first), Some(second)) = (words.next(), words.next()) {
                if first.preposition && second.is_likely_homograph() {
                    tok_cursor = sentence
                        .iter()
                        .position(|t| t.kind.is_comma())
                        .unwrap_or(sentence.iter().len())
                }
            }

            loop {
                if tok_cursor >= sentence.len() {
                    break;
                }

                let match_len = self
                    .pattern
                    .matches(&sentence[tok_cursor..], document.get_source());

                if let Some(match_len) = match_len {
                    let lint = self.match_to_lint(
                        &sentence[tok_cursor..tok_cursor + match_len.get()],
                        document.get_source(),
                    );

                    lints.extend(lint);
                    tok_cursor += match_len.get();
                } else {
                    tok_cursor += 1;
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "The Oxford comma is one of the more controversial rules in common use today. Enabling this lint checks that there is a comma before `and`, `or`, or `nor` when listing out more than two ideas."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::OxfordComma;

    #[test]
    fn fruits() {
        assert_lint_count(
            "An apple, a banana and a pear walk into a bar.",
            OxfordComma::default(),
            1,
        );
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
        assert_lint_count(
            "The team consists of players from different countries: France, Germany, Italy, and Spain.",
            OxfordComma::default(),
            0,
        );
    }

    #[test]
    fn or_writing() {
        assert_suggestion_result(
            "Harper can be a lifesaver when writing technical documents, emails or other formal forms of communication.",
            OxfordComma::default(),
            "Harper can be a lifesaver when writing technical documents, emails, or other formal forms of communication.",
        );
    }

    #[test]
    fn sports() {
        assert_suggestion_result(
            "They enjoy playing soccer, basketball or tennis.",
            OxfordComma::default(),
            "They enjoy playing soccer, basketball, or tennis.",
        );
    }

    #[test]
    fn nor_vegetables() {
        assert_suggestion_result(
            "I like carrots, kale nor broccoli.",
            OxfordComma::default(),
            "I like carrots, kale, nor broccoli.",
        );
    }

    #[test]
    fn allow_non_list_transportation() {
        assert_lint_count(
            "In transportation, autonomous vehicles and smart traffic management systems promise to reduce accidents and optimize travel routes.",
            OxfordComma::default(),
            0,
        );
    }

    #[test]
    fn allow_pill() {
        assert_lint_count(
            "Develop a pill that causes partial amnesia, affecting relationships and identity.",
            OxfordComma::default(),
            0,
        );
    }

    #[test]
    fn allow_at_first() {
        assert_lint_count(
            "In the heart of a bustling city, Sarah finds herself trapped in an endless cycle of the same day. Each morning, she awakens to find the date unchanged, her life on repeat. At first, confusion and frustration cloud her thoughts, but soon she notices something peculiar—each day has tiny differences, subtle changes that hint at a larger pattern.",
            OxfordComma::default(),
            0,
        );
    }

    #[test]
    fn allow_standoff() {
        assert_lint_count(
            "In a tense standoff, Alex and his reflection engage in a battle of wills.",
            OxfordComma::default(),
            0,
        );
    }
}
