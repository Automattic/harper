use hashbrown::HashSet;

use super::token_pattern::TokenPattern;
use super::{Pattern, RepeatingPattern};
use crate::{Lrc, Token, TokenKind};

/// A pattern that checks that a sequence of [`TokenPattern`] matches.
#[derive(Default)]
pub struct SequencePattern {
    token_patterns: Vec<Box<dyn Pattern>>
}

impl SequencePattern {
    pub fn then_exact_word(&mut self, word: &'static str) -> &mut Self {
        self.token_patterns
            .push(Box::new(TokenPattern::WordExact(word)));
        self
    }

    pub fn then_loose(&mut self, kind: TokenKind) -> &mut Self {
        self.token_patterns
            .push(Box::new(TokenPattern::KindLoose(kind)));
        self
    }

    pub fn then_any_word(&mut self) -> &mut Self {
        self.token_patterns
            .push(Box::new(TokenPattern::KindLoose(TokenKind::blank_word())));
        self
    }

    pub fn then_strict(&mut self, kind: TokenKind) -> &mut Self {
        self.token_patterns
            .push(Box::new(TokenPattern::KindStrict(kind)));
        self
    }

    pub fn then_whitespace(&mut self) -> &mut Self {
        self.token_patterns.push(Box::new(TokenPattern::WhiteSpace));
        self
    }

    pub fn then_any_word_in(&mut self, word_set: Lrc<HashSet<&'static str>>) -> &mut Self {
        self.token_patterns
            .push(Box::new(TokenPattern::WordInSet(word_set)));
        self
    }

    pub fn then_one_or_more(&mut self, pat: Box<dyn Pattern>) -> &mut Self {
        self.token_patterns
            .push(Box::new(RepeatingPattern::new(pat)));
        self
    }
}

impl Pattern for SequencePattern {
    fn matches(&self, tokens: &[Token], source: &[char]) -> usize {
        let mut tok_cursor = 0;

        for pat in self.token_patterns.iter() {
            let match_length = pat.matches(&tokens[tok_cursor..], source);

            if match_length == 0 {
                return 0;
            }

            tok_cursor += match_length;
        }

        tok_cursor
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;

    use super::SequencePattern;
    use crate::patterns::Pattern;
    use crate::{Document, Lrc};

    #[test]
    fn matches_n_whitespace_tokens() {
        let mut pat = SequencePattern::default();
        pat.then_any_word().then_whitespace().then_any_word();
        let doc = Document::new_plain_english_curated("word\n    \nword");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_specific_words() {
        let mut pat = SequencePattern::default();
        pat.then_exact_word("she")
            .then_whitespace()
            .then_exact_word("her");
        let doc = Document::new_plain_english_curated("she her");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_sets() {
        let mut pronouns = HashSet::new();
        pronouns.insert("his");
        pronouns.insert("hers");
        let pronouns = Lrc::new(pronouns);

        let mut pat = SequencePattern::default();
        pat.then_exact_word("it")
            .then_whitespace()
            .then_exact_word("was")
            .then_whitespace()
            .then_any_word_in(pronouns);
        let doc = Document::new_plain_english_curated("it was hers");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }
}
