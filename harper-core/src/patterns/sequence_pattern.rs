use paste::paste;

use super::whitespace_pattern::WhitespacePattern;
use super::{
    AnyCapitalization, AnyPattern, IndefiniteArticle, Pattern, RepeatingPattern, SingularSubject,
};
use crate::{Token, TokenKind};

/// A pattern that checks that a sequence of other patterns match.
/// There are specific extension methods available, but you can also use [`Self::then`] to add
/// arbitrary patterns.
///
/// ## Example
///
/// Let's say we wanted to locate places in a [`Document`](crate::Document) where an article is followed by a noun.
/// We can do that with a `SequencePattern`.
///
/// ```rust
/// use harper_core::patterns::{SequencePattern, DocPattern};
/// use harper_core::{Document, Span};
///
/// let document = Document::new_markdown_default_curated("This is a test.");
///
/// let pattern = SequencePattern::default().then_article().then_whitespace().then_nominal();
/// let matches = pattern.find_all_matches_in_doc(&document);
///
/// // The pattern found that the tokens at indexes 4, 5, and 6 fit the criteria.
/// assert_eq!(matches, vec![Span::new(4, 7)]);
/// ```
#[derive(Default)]
pub struct SequencePattern {
    token_patterns: Vec<Box<dyn Pattern>>,
}

/// Generate a `then_*` method from an available `is_*` function on [`TokenKind`].
macro_rules! gen_then_from_is {
    ($quality:ident) => {
        paste! {
            pub fn [< then_$quality >] (mut self) -> Self{
                self.token_patterns.push(Box::new(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                }));

                self
            }

            pub fn [< then_one_or_more_$quality s >] (self) -> Self{
                self.then_one_or_more(Box::new(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                }))
            }

            pub fn [< then_anything_but_$quality >] (mut self) -> Self{
                self.token_patterns.push(Box::new(|tok: &Token, _source: &[char]| {
                    if tok.kind.[< is_$quality >](){
                        false
                    }else{
                        true
                    }
                }));

                self
            }
        }
    };
}

impl SequencePattern {
    gen_then_from_is!(nominal);
    gen_then_from_is!(possessive_nominal);
    gen_then_from_is!(plural_nominal);
    gen_then_from_is!(verb);
    gen_then_from_is!(linking_verb);
    gen_then_from_is!(pronoun);
    gen_then_from_is!(punctuation);
    gen_then_from_is!(conjunction);
    gen_then_from_is!(comma);
    gen_then_from_is!(period);
    gen_then_from_is!(number);
    gen_then_from_is!(case_separator);
    gen_then_from_is!(adverb);
    gen_then_from_is!(adjective);
    gen_then_from_is!(apostrophe);
    gen_then_from_is!(hyphen);
    gen_then_from_is!(article);
    gen_then_from_is!(proper_nominal);

    pub fn then_indefinite_article(self) -> Self {
        self.then(IndefiniteArticle::default())
    }

    pub fn then_exact_word(mut self, word: &'static str) -> Self {
        self.token_patterns
            .push(Box::new(|tok: &Token, source: &[char]| {
                if !tok.kind.is_word() {
                    return false;
                }

                let tok_chars = tok.span.get_content(source);

                let mut w_char_count = 0;
                for (i, w_char) in word.chars().enumerate() {
                    w_char_count += 1;

                    if tok_chars.get(i).cloned() != Some(w_char) {
                        return false;
                    }
                }

                w_char_count == tok_chars.len()
            }));
        self
    }

    pub fn then_singular_subject(self) -> Self {
        self.then(SingularSubject::default())
    }

    /// Shorthand for [`Self::any_capitalization_of`].
    pub fn aco(word: &'static str) -> Self {
        Self::any_capitalization_of(word)
    }

    pub fn any_capitalization_of(word: &'static str) -> Self {
        Self::default().then_any_capitalization_of(word)
    }

    /// Shorthand for [`Self::then_any_capitalization_of`].
    pub fn t_aco(self, word: &'static str) -> Self {
        self.then_any_capitalization_of(word)
    }

    /// Match examples of `word` that have any capitalization.
    pub fn then_any_capitalization_of(mut self, word: &'static str) -> Self {
        self.token_patterns
            .push(Box::new(AnyCapitalization::of(word)));
        self
    }

    /// Matches any word.
    pub fn then_any_word(mut self) -> Self {
        self.token_patterns
            .push(Box::new(|tok: &Token, _source: &[char]| tok.kind.is_word()));
        self
    }

    /// Matches any token whose `Kind` exactly matches.
    pub fn then_strict(mut self, kind: TokenKind) -> Self {
        self.token_patterns
            .push(Box::new(move |tok: &Token, _source: &[char]| {
                tok.kind == kind
            }));
        self
    }

    /// Match against one or more whitespace tokens.
    pub fn then_whitespace(mut self) -> Self {
        self.token_patterns.push(Box::new(WhitespacePattern));
        self
    }

    pub fn then_one_or_more(mut self, pat: Box<dyn Pattern>) -> Self {
        self.token_patterns
            .push(Box::new(RepeatingPattern::new(pat, 0)));
        self
    }

    /// Match against any single token.
    /// More of a filler then anything else.
    pub fn then_anything(mut self) -> Self {
        self.token_patterns.push(Box::new(AnyPattern));
        self
    }

    pub fn then(mut self, pat: impl Pattern + 'static) -> Self {
        self.token_patterns.push(Box::new(pat));
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

    use super::SequencePattern;
    use crate::Document;
    use crate::patterns::Pattern;

    #[test]
    fn matches_n_whitespace_tokens() {
        let pat = SequencePattern::default()
            .then_any_word()
            .then_whitespace()
            .then_any_word();
        let doc = Document::new_plain_english_curated("word\n    \nword");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }

    #[test]
    fn matches_specific_words() {
        let pat = SequencePattern::default()
            .then_exact_word("she")
            .then_whitespace()
            .then_exact_word("her");
        let doc = Document::new_plain_english_curated("she her");

        assert_eq!(
            pat.matches(doc.get_tokens(), doc.get_source()),
            doc.get_tokens().len()
        );
    }
}
