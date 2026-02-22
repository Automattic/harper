use paste::paste;

use crate::{
    CharStringExt, Lrc, Span, Token, TokenKind,
    expr::{FirstMatchOf, FixedPhrase, LongestMatchOf},
    patterns::{AnyPattern, IndefiniteArticle, WhitespacePattern, Word, WordSet},
};

use super::{Expr, Optional, OwnedExprExt, Repeating, Step, UnlessStep};

#[derive(Default)]
pub struct SequenceExpr {
    exprs: Vec<Box<dyn Expr>>,
}

/// Generate a `then_*` method from an available `is_*` function on [`TokenKind`].
macro_rules! gen_then_from_is {
    ($quality:ident) => {
        paste! {
            #[doc = concat!("Adds a step matching a token where [`TokenKind::is_", stringify!($quality), "()`] returns true.")]
            pub fn [< then_$quality >] (self) -> Self{
                self.then_kind_where(|kind| {
                    kind.[< is_$quality >]()
                })
            }

            #[doc = concat!("Adds an optional step matching a token where [`TokenKind::is_", stringify!($quality), "()`] returns true.")]
            pub fn [< then_optional_$quality >] (self) -> Self{
                self.then_optional(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                })
            }

            #[doc = concat!("Adds a step matching one or more consecutive tokens where [`TokenKind::is_", stringify!($quality), "()`] returns true.")]
            pub fn [< then_one_or_more_$quality s >] (self) -> Self{
                self.then_one_or_more(Box::new(|tok: &Token, _source: &[char]| {
                    tok.kind.[< is_$quality >]()
                }))
            }

            #[doc = concat!("Adds a step matching a token where [`TokenKind::is_", stringify!($quality), "()`] returns false.")]
            pub fn [< then_anything_but_$quality >] (self) -> Self{
                self.then_kind_where(|kind| {
                    !kind.[< is_$quality >]()
                })
            }
        }
    };
}

impl Expr for SequenceExpr {
    /// Run the expression starting at an index, returning the total matched window.
    ///
    /// If any step returns `None`, the entire expression does as well.
    fn run(&self, mut cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let mut window = Span::empty(cursor);

        for cur_expr in &self.exprs {
            let out = cur_expr.run(cursor, tokens, source)?;

            // Only expand the window if the match actually covers some tokens
            if out.end > out.start {
                window.expand_to_include(out.start);
                window.expand_to_include(out.end.checked_sub(1).unwrap_or(out.start));
            }

            // Only advance cursor if we actually matched something
            if out.end > cursor {
                cursor = out.end;
            } else if out.start < cursor {
                cursor = out.start;
            }
            // If both start and end are equal to cursor, don't move the cursor
        }

        Some(window)
    }
}

impl SequenceExpr {
    // Constructor methods

    // Match an [expression](Expr).
    pub fn with(expr: impl Expr + 'static) -> Self {
        Self::default().then(expr)
    }

    // Single token methods

    /// Construct a new sequence with an [`AnyPattern`] at the beginning of the operation list.
    pub fn anything() -> Self {
        Self::default().then_anything()
    }

    // Single word token methods

    /// Construct a new sequence with a [`Word`] at the beginning of the operation list.
    pub fn any_capitalization_of(word: &'static str) -> Self {
        Self::default().then_any_capitalization_of(word)
    }

    /// Shorthand for [`Self::any_capitalization_of`].
    pub fn aco(word: &'static str) -> Self {
        Self::any_capitalization_of(word)
    }

    /// Match any word from the given set of words, case-insensitive.
    pub fn word_set(words: &'static [&'static str]) -> Self {
        Self::default().then_word_set(words)
    }

    /// Match any word.
    pub fn any_word() -> Self {
        Self::default().then_any_word()
    }

    // Expressions of more than one token

    /// Optionally match an expression.
    pub fn optional(expr: impl Expr + 'static) -> Self {
        Self::default().then_optional(expr)
    }

    /// Match a fixed phrase.
    pub fn fixed_phrase(phrase: &'static str) -> Self {
        Self::default().then_fixed_phrase(phrase)
    }

    // Multiple expressions

    /// Match the first of multiple expressions.
    pub fn any_of(exprs: Vec<Box<dyn Expr>>) -> Self {
        Self::default().then_any_of(exprs)
    }

    /// Match the longest of multiple expressions.
    pub fn longest_of(exprs: Vec<Box<dyn Expr>>) -> Self {
        Self::default().then_longest_of(exprs)
    }

    pub fn whitespace() -> Self {
        Self::default().then_whitespace()
    }

    /// Will be accepted unless the condition matches.
    pub fn unless(condition: impl Expr + 'static) -> Self {
        Self::default().then_unless(condition)
    }

    // Builder methods

    /// Push an [expression](Expr) to the operation list.
    pub fn then(mut self, expr: impl Expr + 'static) -> Self {
        self.exprs.push(Box::new(expr));
        self
    }

    /// Push an already-boxed [expression](Expr) to the operation list.
    pub fn then_boxed(mut self, expr: Box<dyn Expr>) -> Self {
        self.exprs.push(expr);
        self
    }

    /// Pushes an expression that could move the cursor to the sequence, but does not require it.
    pub fn then_optional(mut self, expr: impl Expr + 'static) -> Self {
        self.exprs.push(Box::new(Optional::new(expr)));
        self
    }

    /// Pushes an expression that will match any of the provided expressions.
    ///
    /// If more than one of the provided expressions match, this function provides no guarantee
    /// as to which match will end up being used. If you need to get the longest of multiple
    /// matches, use [`Self::then_longest_of()`] instead.
    pub fn then_any_of(mut self, exprs: Vec<Box<dyn Expr>>) -> Self {
        self.exprs.push(Box::new(FirstMatchOf::new(exprs)));
        self
    }

    /// Pushes an expression that will match the longest of the provided expressions.
    ///
    /// If you don't need the longest match, prefer using the short-circuiting
    /// [`Self::then_any_of()`] instead.
    pub fn then_longest_of(mut self, exprs: Vec<Box<dyn Expr>>) -> Self {
        self.exprs.push(Box::new(LongestMatchOf::new(exprs)));
        self
    }

    /// Appends the steps in `other` onto the end of `self`.
    /// This is more efficient than [`Self::then`] because it avoids pointer redirection.
    pub fn then_seq(mut self, mut other: Self) -> Self {
        self.exprs.append(&mut other.exprs);
        self
    }

    /// Pushes an expression that will match any word from the given set of words, case-insensitive.
    pub fn then_word_set(self, words: &'static [&'static str]) -> Self {
        self.then(WordSet::new(words))
    }

    /// Shorthand for [`Self::then_word_set`].
    pub fn t_set(self, words: &'static [&'static str]) -> Self {
        self.then_word_set(words)
    }

    /// Match against one or more whitespace tokens.
    pub fn then_whitespace(self) -> Self {
        self.then(WhitespacePattern)
    }

    /// Shorthand for [`Self::then_whitespace`].
    pub fn t_ws(self) -> Self {
        self.then_whitespace()
    }

    /// Match against one or more whitespace tokens.
    pub fn then_whitespace_or_hyphen(self) -> Self {
        self.then(WhitespacePattern.or(|tok: &Token, _: &[char]| tok.kind.is_hyphen()))
    }

    /// Shorthand for [`Self::then_whitespace_or_hyphen`].
    pub fn t_ws_h(self) -> Self {
        self.then_whitespace_or_hyphen()
    }

    /// Match against zero or more occurrences of the given expression. Like `*` in regex.
    pub fn then_zero_or_more(self, expr: impl Expr + 'static) -> Self {
        self.then(Repeating::new(Box::new(expr), 0))
    }

    /// Match against one or more occurrences of the given expression. Like `+` in regex.
    pub fn then_one_or_more(self, expr: impl Expr + 'static) -> Self {
        self.then(Repeating::new(Box::new(expr), 1))
    }

    /// Match against zero or more whitespace-separated occurrences of the given expression.
    pub fn then_zero_or_more_spaced(self, expr: impl Expr + 'static) -> Self {
        let expr = Lrc::new(expr);
        self.then(SequenceExpr::with(expr.clone()).then(Repeating::new(
            Box::new(SequenceExpr::default().t_ws().then(expr)),
            0,
        )))
    }

    /// Create a new condition that will step one token forward if met.
    /// If the condition is _not_ met, the whole expression returns `None`.
    ///
    /// This can be used to build out exceptions to other rules.
    ///
    /// See [`UnlessStep`] for more info.
    pub fn then_unless(self, condition: impl Expr + 'static) -> Self {
        self.then(UnlessStep::new(condition, |_tok: &Token, _src: &[char]| {
            true
        }))
    }

    /// Match any single token.
    ///
    /// See [`AnyPattern`] for more info.
    pub fn then_anything(self) -> Self {
        self.then(AnyPattern)
    }

    /// Match any single token.
    ///
    /// Shorthand for [`Self::then_anything`].
    pub fn t_any(self) -> Self {
        self.then_anything()
    }

    // Word matching methods

    /// Matches any word.
    pub fn then_any_word(self) -> Self {
        self.then_kind_where(|kind| kind.is_word())
    }

    /// Match examples of `word` that have any capitalization.
    pub fn then_any_capitalization_of(self, word: &'static str) -> Self {
        self.then(Word::new(word))
    }

    /// Shorthand for [`Self::then_any_capitalization_of`].
    pub fn t_aco(self, word: &'static str) -> Self {
        self.then_any_capitalization_of(word)
    }

    /// Match examples of `word` case-sensitively.
    pub fn then_exact_word(self, word: &'static str) -> Self {
        self.then(Word::new_exact(word))
    }

    /// Match a fixed phrase.
    pub fn then_fixed_phrase(self, phrase: &'static str) -> Self {
        self.then(FixedPhrase::from_phrase(phrase))
    }

    /// Match any word except the ones in `words`.
    pub fn then_word_except(self, words: &'static [&'static str]) -> Self {
        self.then(move |tok: &Token, src: &[char]| {
            !tok.kind.is_word()
                || !words
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    // Token kind/predicate matching methods

    // One kind

    /// Matches any token whose `Kind` exactly matches.
    pub fn then_kind(self, kind: TokenKind) -> Self {
        self.then_kind_where(move |k| kind == *k)
    }

    /// Matches a token where the provided closure returns true for the token's kind.
    pub fn then_kind_where<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.exprs
            .push(Box::new(move |tok: &Token, _source: &[char]| {
                predicate(&tok.kind)
            }));
        self
    }

    /// Match a token of a given kind which is not in the list of words.
    pub fn then_kind_except<F>(self, pred_is: F, ex: &'static [&'static str]) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            pred_is(&tok.kind)
                && !ex
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    // Two kinds

    /// Match a token where both token kind predicates return true.
    /// For instance, a word that can be both noun and verb.
    pub fn then_kind_both<F1, F2>(self, pred_is_1: F1, pred_is_2: F2) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| pred_is_1(k) && pred_is_2(k))
    }

    /// Match a token where either of the two token kind predicates returns true.
    /// For instance, an adjective or an adverb.
    pub fn then_kind_either<F1, F2>(self, pred_is_1: F1, pred_is_2: F2) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| pred_is_1(k) || pred_is_2(k))
    }

    /// Match a token where neither of the two token kind predicates returns true.
    /// For instance, a word that can't be a verb or a noun.
    pub fn then_kind_neither<F1, F2>(self, pred_isnt_1: F1, pred_isnt_2: F2) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| !pred_isnt_1(k) && !pred_isnt_2(k))
    }

    /// Match a token where the first token kind predicate returns true and the second returns false.
    /// For instance, a word that can be a noun but cannot be a verb.
    pub fn then_kind_is_but_is_not<F1, F2>(self, pred_is: F1, pred_not: F2) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| pred_is(k) && !pred_not(k))
    }

    /// Match a token where the first token kind predicate returns true and the second returns false,
    /// and the token is not in the list of exceptions.
    pub fn then_kind_is_but_is_not_except<F1, F2>(
        self,
        pred_is: F1,
        pred_not: F2,
        ex: &'static [&'static str],
    ) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            pred_is(&tok.kind)
                && !pred_not(&tok.kind)
                && !ex
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    /// Match a token where the first token kind predicate returns true and all of the second return false.
    /// For instance, a word that can be a verb but not a noun or an adjective.
    pub fn then_kind_is_but_isnt_any_of<F1, F2>(
        self,
        pred_is: F1,
        preds_isnt: &'static [F2],
    ) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| pred_is(k) && !preds_isnt.iter().any(|pred| pred(k)))
    }

    /// Match a token where the first token kind predicate returns true and all of the second return false,
    /// and the token is not in the list of exceptions.
    /// For instance, an adjective that isn't also a verb or adverb or the word "likely".
    pub fn then_kind_is_but_isnt_any_of_except<F1, F2>(
        self,
        pred_is: F1,
        preds_isnt: &'static [F2],
        ex: &'static [&'static str],
    ) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            pred_is(&tok.kind)
                && !preds_isnt.iter().any(|pred| pred(&tok.kind))
                && !ex
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    // More than two kinds

    /// Match a token where both of the first two token kind predicates return true,
    /// and the third returns false.
    /// For instance, a word that must be both noun and verb, but not adjective.
    pub fn then_kind_both_but_not<F1, F2, F3>(
        self,
        (pred_is_1, pred_is_2): (F1, F2),
        pred_not: F3,
    ) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F3: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| pred_is_1(k) && pred_is_2(k) && !pred_not(k))
    }

    /// Match a token where any of the token kind predicates returns true.
    /// Like `then_kind_either` but for more than two predicates.
    pub fn then_kind_any<F>(self, preds_is: &'static [F]) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| preds_is.iter().any(|pred| pred(k)))
    }

    /// Match a token where none of the token kind predicates returns true.
    /// Like `then_kind_neither` but for more than two predicates.
    pub fn then_kind_none_of<F>(self, preds_isnt: &'static [F]) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then_kind_where(move |k| preds_isnt.iter().all(|pred| !pred(k)))
    }

    /// Match a token where any of the token kind predicates returns true,
    /// and the word is not in the list of exceptions.
    pub fn then_kind_any_except<F>(
        self,
        preds_is: &'static [F],
        ex: &'static [&'static str],
    ) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            preds_is.iter().any(|pred| pred(&tok.kind))
                && !ex
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    /// Match a token where any of the token kind predicates returns true,
    /// or the token is in the list of words.
    pub fn then_kind_any_or_words<F>(
        self,
        preds: &'static [F],
        words: &'static [&'static str],
    ) -> Self
    where
        F: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            preds.iter().any(|pred| pred(&tok.kind))
                || words
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    /// Match a token where any of the first token kind predicates returns true,
    /// the second returns false, and the token is not in the list of exceptions.    
    pub fn then_kind_any_but_not_except<F1, F2>(
        self,
        preds_is: &'static [F1],
        pred_not: F2,
        ex: &'static [&'static str],
    ) -> Self
    where
        F1: Fn(&TokenKind) -> bool + Send + Sync + 'static,
        F2: Fn(&TokenKind) -> bool + Send + Sync + 'static,
    {
        self.then(move |tok: &Token, src: &[char]| {
            preds_is.iter().any(|pred| pred(&tok.kind))
                && !pred_not(&tok.kind)
                && !ex
                    .iter()
                    .any(|&word| tok.span.get_content(src).eq_ignore_ascii_case_str(word))
        })
    }

    // Word property matching methods

    // Out-of-vocabulary word. (Words not in the dictionary)
    gen_then_from_is!(oov);
    gen_then_from_is!(swear);

    // Part-of-speech matching methods

    // Nominals (nouns and pronouns)

    gen_then_from_is!(nominal);
    gen_then_from_is!(plural_nominal);
    gen_then_from_is!(non_plural_nominal);
    gen_then_from_is!(possessive_nominal);

    // Nouns

    gen_then_from_is!(noun);
    gen_then_from_is!(proper_noun);
    gen_then_from_is!(plural_noun);
    gen_then_from_is!(singular_noun);
    gen_then_from_is!(mass_noun_only);

    // Pronouns

    gen_then_from_is!(pronoun);
    gen_then_from_is!(personal_pronoun);
    gen_then_from_is!(first_person_singular_pronoun);
    gen_then_from_is!(first_person_plural_pronoun);
    gen_then_from_is!(second_person_pronoun);
    gen_then_from_is!(third_person_pronoun);
    gen_then_from_is!(third_person_singular_pronoun);
    gen_then_from_is!(third_person_plural_pronoun);
    gen_then_from_is!(subject_pronoun);
    gen_then_from_is!(object_pronoun);

    // Verbs

    gen_then_from_is!(verb);
    gen_then_from_is!(auxiliary_verb);
    gen_then_from_is!(linking_verb);
    gen_then_from_is!(verb_lemma);
    gen_then_from_is!(verb_simple_past_form);
    gen_then_from_is!(verb_past_participle_form);
    gen_then_from_is!(verb_progressive_form);
    gen_then_from_is!(verb_third_person_singular_present_form);

    // Adjectives

    gen_then_from_is!(adjective);
    gen_then_from_is!(positive_adjective);
    gen_then_from_is!(comparative_adjective);
    gen_then_from_is!(superlative_adjective);

    // Adverbs

    gen_then_from_is!(adverb);
    gen_then_from_is!(frequency_adverb);
    gen_then_from_is!(degree_adverb);

    // Determiners

    gen_then_from_is!(determiner);
    gen_then_from_is!(demonstrative_determiner);
    gen_then_from_is!(possessive_determiner);
    gen_then_from_is!(quantifier);
    gen_then_from_is!(non_quantifier_determiner);
    gen_then_from_is!(non_demonstrative_determiner);

    /// Push an [`IndefiniteArticle`] to the end of the operation list.
    pub fn then_indefinite_article(self) -> Self {
        self.then(IndefiniteArticle::default())
    }

    // Other parts of speech

    gen_then_from_is!(conjunction);
    gen_then_from_is!(preposition);

    // Numbers

    gen_then_from_is!(number);
    gen_then_from_is!(cardinal_number);
    gen_then_from_is!(ordinal_number);

    // Punctuation

    gen_then_from_is!(punctuation);
    gen_then_from_is!(apostrophe);
    gen_then_from_is!(comma);
    gen_then_from_is!(hyphen);
    gen_then_from_is!(period);
    gen_then_from_is!(semicolon);
    gen_then_from_is!(quote);
    gen_then_from_is!(backslash);
    gen_then_from_is!(slash);
    gen_then_from_is!(percent);

    // Other

    gen_then_from_is!(case_separator);
    gen_then_from_is!(likely_homograph);
    gen_then_from_is!(sentence_terminator);
}

impl<S> From<S> for SequenceExpr
where
    S: Step + 'static,
{
    fn from(step: S) -> Self {
        Self {
            exprs: vec![Box::new(step)],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document, TokenKind,
        expr::{ExprExt, SequenceExpr},
        linting::tests::SpanVecExt,
    };

    #[test]
    fn test_kind_both() {
        let noun_and_verb =
            SequenceExpr::default().then_kind_both(TokenKind::is_noun, TokenKind::is_verb);
        let doc = Document::new_plain_english_curated("Use a good example.");
        let matches = noun_and_verb.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["Use", "good", "example"]);
    }

    #[test]
    fn test_adjective_or_determiner() {
        let expr = SequenceExpr::default()
            .then_kind_either(TokenKind::is_adjective, TokenKind::is_determiner);
        let doc = Document::new_plain_english_curated("Use a good example.");
        let matches = expr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["a", "good"]);
    }

    #[test]
    fn test_noun_but_not_adjective() {
        let expr = SequenceExpr::default()
            .then_kind_is_but_is_not(TokenKind::is_noun, TokenKind::is_adjective);
        let doc = Document::new_plain_english_curated("Use a good example.");
        let matches = expr.iter_matches_in_doc(&doc).collect::<Vec<_>>();
        assert_eq!(matches.to_strings(&doc), vec!["Use", "example"]);
    }
}
