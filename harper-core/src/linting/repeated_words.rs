use super::{Lint, LintKind, Linter, Suggestion};
use crate::TokenStringExt;
use crate::char_string::char_string;
use crate::{CharString, CharStringExt, Document, Span};

#[derive(Debug, Clone)]
pub struct RepeatedWords {
    /// Words that we need to make sure are detected.
    /// We use a `Vec` since there aren't a whole lot of 'em.
    special_cases: Vec<CharString>,
}
impl RepeatedWords {
    pub fn new() -> Self {
        Self {
            special_cases: vec![char_string!("this")],
        }
    }

    fn is_special_case(&self, chars: &[char]) -> bool {
        let lower = chars.to_lower();

        self.special_cases
            .iter()
            .any(|v| v.as_slice() == lower.as_ref())
    }
}

impl Default for RepeatedWords {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for RepeatedWords {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let mut iter = chunk.iter_word_indices().zip(chunk.iter_words()).peekable();

            while let (Some((idx_a, tok_a)), Some((idx_b, tok_b))) = (iter.next(), iter.peek()) {
                let word_a = document.get_span_content(&tok_a.span);
                let word_b = document.get_span_content(&tok_b.span);

                let prev_tok = document.get_token_offset(idx_a, -1);
                let next_tok = document.get_token_offset(*idx_b, 1);

                if prev_tok.is_some_and(|t| t.kind.is_hyphen())
                    || next_tok.is_some_and(|t| t.kind.is_hyphen())
                {
                    continue;
                }

                if (tok_a.kind.is_preposition()
                    || tok_a.kind.is_conjunction()
                    || !tok_a.kind.is_likely_homograph()
                    || self.is_special_case(word_a)
                    || tok_a.kind.is_adverb()
                    || tok_a.kind.is_determiner())
                    && word_a.to_lower() == word_b.to_lower()
                {
                    let intervening_tokens = &chunk[idx_a + 1..*idx_b];

                    if intervening_tokens.iter().any(|t| !t.kind.is_whitespace()) {
                        continue;
                    }

                    lints.push(Lint {
                        span: Span::new(tok_a.span.start, tok_b.span.end),
                        lint_kind: LintKind::Repetition,
                        suggestions: vec![Suggestion::ReplaceWith(
                            document.get_span_content(&tok_a.span).to_vec(),
                        )],
                        message: "Did you mean to repeat this word?".to_string(),
                        ..Default::default()
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "This rule looks for repetitions of words that are not homographs."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::super::tests::assert_lint_count;
    use super::RepeatedWords;

    #[test]
    fn catches_basic() {
        assert_lint_count("I wanted the the banana.", RepeatedWords::default(), 1)
    }

    #[test]
    fn does_not_lint_homographs_address() {
        assert_lint_count("To address address problems.", RepeatedWords::default(), 0);
    }

    #[test]
    fn does_not_lint_homographs_record() {
        assert_lint_count("To record record profits.", RepeatedWords::default(), 0);
    }

    #[test]
    fn issue_253() {
        assert_lint_count(
            "this paper shows that, while the method may be more accurate accurate, the turnout overestimate suggests that self-selection bias is not sufficiently reduced",
            RepeatedWords::default(),
            1,
        );
    }

    #[test]
    fn issue_333() {
        assert_suggestion_result(
            "This is is a test",
            RepeatedWords::default(),
            "This is a test",
        );
    }

    #[test]
    fn double_a() {
        assert_suggestion_result(
            "This is a a test",
            RepeatedWords::default(),
            "This is a test",
        );
    }

    #[test]
    fn double_and() {
        assert_suggestion_result(
            "And and this is also a test",
            RepeatedWords::default(),
            "And this is also a test",
        );
    }

    #[test]
    fn on_on_github() {
        assert_suggestion_result(
            "Take a look at the project on on GitHub.",
            RepeatedWords::default(),
            "Take a look at the project on GitHub.",
        );
    }

    #[test]
    fn as_as() {
        assert_suggestion_result(
            "he is as as hard as nails",
            RepeatedWords::default(),
            "he is as hard as nails",
        );
    }

    #[test]
    fn dont_flag_first_hyphenated() {
        assert_lint_count(
            "The driver-facing camera and microphone are only logged if you explicitly opt-in in settings.",
            RepeatedWords::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_hyphenated_either_side() {
        assert_lint_count("foo-foo foo bar bar-bar", RepeatedWords::default(), 0);
    }
}
