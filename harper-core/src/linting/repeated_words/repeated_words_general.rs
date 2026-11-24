use crate::{
    CharString, CharStringExt, Document, Span, TokenStringExt,
    char_string::char_string,
    linting::{Lint, LintKind, Linter, Suggestion, informal_laughter::is_informal_laughter},
};

#[derive(Debug, Clone)]
pub struct RepeatedWordsGeneral {
    /// Words that we need to make sure are detected.
    /// We use a `Vec` since there aren't a whole lot of 'em.
    special_cases: Vec<CharString>,
}
impl RepeatedWordsGeneral {
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

impl Default for RepeatedWordsGeneral {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for RepeatedWordsGeneral {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let mut iter = chunk.iter_word_indices().zip(chunk.iter_words()).peekable();

            while let (Some((idx_a, tok_a)), Some((idx_b, tok_b))) = (iter.next(), iter.peek()) {
                let word_a = document.get_span_content(&tok_a.span);
                let word_b = document.get_span_content(&tok_b.span);

                if is_informal_laughter(word_a) && is_informal_laughter(word_b) {
                    continue;
                }

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
                        message: "Did you mean to repeat this word?".to_owned(),
                        // Lower priority (higher number) than `TheTheToThatThe`
                        priority: 128,
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
