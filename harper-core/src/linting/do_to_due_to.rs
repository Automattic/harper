use crate::{Document, Lint, Token, linting::Linter, spell::Dictionary, weir::WeirLinter};

const RULE: &str = include_str!("do_to_due_to.weir");

pub struct DoToDueTo<D> {
    inner: WeirLinter,
    dictionary: D,
}

impl<D: Dictionary> DoToDueTo<D> {
    pub fn new(dictionary: D) -> Self {
        Self {
            inner: WeirLinter::new(RULE).expect("bundled DoToDueTo rule should be valid Weir"),
            dictionary,
        }
    }

    fn starts_compound_verb_after_auxiliary(&self, document: &Document, lint: &Lint) -> bool {
        let tokens = document.get_tokens();
        let Some(do_index) = tokens
            .iter()
            .position(|token| token.span.start == lint.span.start)
        else {
            return false;
        };

        let Some(previous_index) = previous_word_index(tokens, do_index) else {
            return false;
        };
        let follows_auxiliary = tokens[previous_index].kind.is_auxiliary_verb()
            || (tokens[previous_index].kind.is_adverb()
                && previous_word_index(tokens, previous_index)
                    .is_some_and(|index| tokens[index].kind.is_auxiliary_verb()));
        if !follows_auxiliary {
            return false;
        }

        let Some(to_index) = tokens
            .iter()
            .position(|token| token.span.end == lint.span.end)
        else {
            return false;
        };
        let Some([whitespace, first, separator, second]) = tokens.get(to_index + 1..to_index + 5)
        else {
            return false;
        };

        if !whitespace.kind.is_whitespace()
            || !first.kind.is_word()
            || !separator.kind.is_whitespace()
            || !second.kind.is_word()
        {
            return false;
        }

        self.dictionary
            .get_word_metadata(&document.get_source()[first.span.start..second.span.end])
            .is_some_and(|metadata| metadata.is_verb_lemma())
    }
}

fn previous_word_index(tokens: &[Token], base: usize) -> Option<usize> {
    let whitespace_index = base.checked_sub(1)?;
    if !tokens.get(whitespace_index)?.kind.is_whitespace() {
        return None;
    }

    let word_index = whitespace_index.checked_sub(1)?;
    tokens.get(word_index)?.kind.is_word().then_some(word_index)
}

impl<D: Dictionary> Linter for DoToDueTo<D> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.inner
            .lint(document)
            .into_iter()
            .filter(|lint| !self.starts_compound_verb_after_auxiliary(document, lint))
            .collect()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }
}

#[cfg(test)]
mod tests {
    use super::{DoToDueTo, RULE};
    use crate::{
        linting::tests::{assert_no_lints, assert_suggestion_result},
        spell::FstDictionary,
        weir::{WeirLinter, tests::assert_passes_all},
    };

    #[test]
    fn allows_compound_verb_after_auxiliary() {
        assert_no_lints(
            "If there is anything I can do to beta test a Mac or Windows version, let me know.",
            DoToDueTo::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn allows_compound_verb_after_auxiliary_and_adverb() {
        assert_no_lints(
            "If there is anything I can still do to beta test a Mac version, let me know.",
            DoToDueTo::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn corrects_compound_noun_in_causal_phrase() {
        assert_suggestion_result(
            "The release was delayed do to beta test results.",
            DoToDueTo::new(FstDictionary::curated()),
            "The release was delayed due to beta test results.",
        );
    }

    #[test]
    fn bundled_weir_tests_pass() {
        let mut linter = WeirLinter::new(RULE).unwrap();
        assert_passes_all(&mut linter);
    }
}
