use crate::{CharString, Token};

use super::{Pattern, SinlgeTokenPattern, WhitespacePattern, Word};

pub trait IntoPattern {
    type Output: Pattern + 'static;
    fn into_pattern(self) -> Self::Output;

    fn into_pattern_boxed(self) -> Box<dyn Pattern + 'static>
    where
        Self: Sized,
    {
        Box::new(self.into_pattern())
    }
}
impl<T: Pattern + 'static> IntoPattern for T {
    type Output = T;
    fn into_pattern(self) -> Self::Output {
        self
    }
}
impl IntoPattern for &'static str {
    type Output = Word;
    fn into_pattern(self) -> Self::Output {
        Word::new(self)
    }
}

pub struct Sequence {
    patterns: Vec<Box<dyn Pattern>>,
}
impl Sequence {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
}
impl Pattern for Sequence {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        let mut tok_cursor = 0;

        for pat in &self.patterns {
            let match_length = pat.matches(&tokens[tok_cursor..], source)?;
            tok_cursor += match_length;
        }

        Some(tok_cursor)
    }
}

pub struct Choice {
    patterns: Vec<Box<dyn Pattern>>,
}
impl Choice {
    pub fn new(patterns: Vec<Box<dyn Pattern>>) -> Self {
        Self { patterns }
    }
}
impl Pattern for Choice {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        let mut longest: Option<usize> = None;

        for pattern in self.patterns.iter() {
            let Some(match_len) = pattern.matches(tokens, source) else {
                continue;
            };

            longest = Some(longest.unwrap_or(0).max(match_len));
        }

        longest
    }
}

struct ExactWord {
    word: CharString,
}
impl SinlgeTokenPattern for ExactWord {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        if !token.kind.is_word() {
            return false;
        }
        if token.span.len() != self.word.len() {
            return false;
        }

        let chars = token.span.get_content(source);
        chars == self.word.as_slice()
    }
}
pub fn exact(word: &str) -> impl Pattern {
    ExactWord {
        word: word.chars().collect(),
    }
}

struct Not<P>(P);
impl<P: Pattern> Pattern for Not<P> {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        if self.0.matches(tokens, source).is_some() {
            None
        } else {
            Some(0)
        }
    }
}
pub fn not(pattern: impl Pattern) -> impl Pattern {
    Not(pattern)
}

struct Ahead<P>(P);
impl<P: Pattern> Pattern for Ahead<P> {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        if self.0.matches(tokens, source).is_some() {
            Some(0)
        } else {
            None
        }
    }
}
pub fn ahead(pattern: impl Pattern) -> impl Pattern {
    Ahead(pattern)
}

/// Matches any single token, regardless of its kind.
#[derive(Clone, Copy)]
pub struct AnyToken;
impl SinlgeTokenPattern for AnyToken {
    fn matches_token(&self, _token: &Token, _source: &[char]) -> bool {
        true
    }
}

/// Matches any word token.
#[derive(Clone, Copy)]
pub struct AnyWord;
impl SinlgeTokenPattern for AnyWord {
    fn matches_token(&self, token: &Token, _source: &[char]) -> bool {
        token.kind.is_word()
    }
}

pub const WS: WhitespacePattern = WhitespacePattern;

pub mod predicates {
    use crate::{
        Token, TokenKind,
        patterns::{SinlgeTokenPattern, Word},
    };

    pub trait IntoSingleTokenPattern {
        type Output: SinlgeTokenPattern;
        fn into_single_token_pattern(self) -> Self::Output;
    }
    impl<T: SinlgeTokenPattern> IntoSingleTokenPattern for T {
        type Output = T;
        fn into_single_token_pattern(self) -> Self::Output {
            self
        }
    }
    impl IntoSingleTokenPattern for &'static str {
        type Output = Word;
        fn into_single_token_pattern(self) -> Self::Output {
            Word::new(self)
        }
    }

    #[derive(Clone, Copy)]
    pub struct Not<P: SinlgeTokenPattern>(P);
    impl<P: SinlgeTokenPattern> SinlgeTokenPattern for Not<P> {
        fn matches_token(&self, token: &Token, source: &[char]) -> bool {
            !self.0.matches_token(token, source)
        }
    }
    impl<P: SinlgeTokenPattern> std::ops::Not for Not<P> {
        type Output = P;
        fn not(self) -> Self::Output {
            self.0
        }
    }
    impl<P: SinlgeTokenPattern, R: IntoSingleTokenPattern> std::ops::BitAnd<R> for Not<P> {
        type Output = And<Self, R::Output>;
        fn bitand(self, rhs: R) -> Self::Output {
            And(self, rhs.into_single_token_pattern())
        }
    }
    impl<P: SinlgeTokenPattern, R: IntoSingleTokenPattern> std::ops::BitOr<R> for Not<P> {
        type Output = Or<Self, R::Output>;
        fn bitor(self, rhs: R) -> Self::Output {
            Or(self, rhs.into_single_token_pattern())
        }
    }

    #[derive(Clone, Copy)]
    pub struct And<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern>(P1, P2);
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern> SinlgeTokenPattern for And<P1, P2> {
        fn matches_token(&self, token: &Token, source: &[char]) -> bool {
            self.0.matches_token(token, source) && self.1.matches_token(token, source)
        }
    }
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern> std::ops::Not for And<P1, P2> {
        type Output = Not<Self>;
        fn not(self) -> Self::Output {
            Not(self)
        }
    }
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern, R: IntoSingleTokenPattern>
        std::ops::BitAnd<R> for And<P1, P2>
    {
        type Output = And<Self, R::Output>;
        fn bitand(self, rhs: R) -> Self::Output {
            And(self, rhs.into_single_token_pattern())
        }
    }
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern, R: IntoSingleTokenPattern>
        std::ops::BitOr<R> for And<P1, P2>
    {
        type Output = Or<Self, R::Output>;
        fn bitor(self, rhs: R) -> Self::Output {
            Or(self, rhs.into_single_token_pattern())
        }
    }

    #[derive(Clone, Copy)]
    pub struct Or<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern>(P1, P2);
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern> SinlgeTokenPattern for Or<P1, P2> {
        fn matches_token(&self, token: &Token, source: &[char]) -> bool {
            self.0.matches_token(token, source) || self.1.matches_token(token, source)
        }
    }
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern, R: IntoSingleTokenPattern>
        std::ops::BitAnd<R> for Or<P1, P2>
    {
        type Output = And<Self, R::Output>;
        fn bitand(self, rhs: R) -> Self::Output {
            And(self, rhs.into_single_token_pattern())
        }
    }
    impl<P1: SinlgeTokenPattern, P2: SinlgeTokenPattern, R: IntoSingleTokenPattern>
        std::ops::BitOr<R> for Or<P1, P2>
    {
        type Output = Or<Self, R::Output>;
        fn bitor(self, rhs: R) -> Self::Output {
            Or(self, rhs.into_single_token_pattern())
        }
    }

    macro_rules! add_operators {
        ($name:ident) => {
            impl std::ops::Not for $name {
                type Output = Not<$name>;
                fn not(self) -> Self::Output {
                    Not(self)
                }
            }
            impl<R: IntoSingleTokenPattern> std::ops::BitAnd<R> for $name {
                type Output = And<$name, R::Output>;
                fn bitand(self, rhs: R) -> Self::Output {
                    And(self, rhs.into_single_token_pattern())
                }
            }
            impl<R: IntoSingleTokenPattern> std::ops::BitOr<R> for $name {
                type Output = Or<$name, R::Output>;
                fn bitor(self, rhs: R) -> Self::Output {
                    Or(self, rhs.into_single_token_pattern())
                }
            }
        };
    }
    macro_rules! create_predicte {
        ($name:ident, $fn:expr) => {
            #[derive(Clone, Copy)]
            pub struct $name;
            impl SinlgeTokenPattern for $name {
                fn matches_token(&self, token: &Token, _: &[char]) -> bool {
                    $fn(&token.kind)
                }
            }

            add_operators!($name);
        };
    }

    create_predicte!(Noun, TokenKind::is_noun);
    create_predicte!(NounPl, TokenKind::is_plural_noun);
    create_predicte!(Pronoun, TokenKind::is_pronoun);
    create_predicte!(PronounPl, TokenKind::is_plural_pronoun);
    create_predicte!(Nominal, TokenKind::is_nominal);
    create_predicte!(NominalPl, TokenKind::is_plural_nominal);

    create_predicte!(Verb, TokenKind::is_verb);
    create_predicte!(AuxVerb, TokenKind::is_auxiliary_verb);
    create_predicte!(LinkingVerb, TokenKind::is_linking_verb);
    create_predicte!(Adj, TokenKind::is_adjective);
    create_predicte!(Adverb, TokenKind::is_adverb);
    create_predicte!(Det, TokenKind::is_determiner);
    create_predicte!(Prep, TokenKind::is_preposition);

    create_predicte!(Common, TokenKind::is_common_word);
    create_predicte!(Homograph, TokenKind::is_likely_homograph);

    #[derive(Clone, Copy)]
    pub struct Punct<const P: char>;

    macro_rules! define_punct {
        ($name:ident, $char:literal, $fn:expr) => {
            impl Punct<$char> {
                pub const $name: Punct<$char> = Self;
            }
            impl SinlgeTokenPattern for Punct<$char> {
                fn matches_token(&self, token: &Token, _source: &[char]) -> bool {
                    $fn(&token.kind)
                }
            }
        };
    }

    define_punct!(HYPHEN, '-', TokenKind::is_hyphen);
    define_punct!(COMMA, ',', TokenKind::is_comma);
    define_punct!(QUOTE, '"', TokenKind::is_quote);
    define_punct!(APOS, '\'', TokenKind::is_apostrophe);
    define_punct!(PERIOD, '.', TokenKind::is_period);
    define_punct!(AT, '@', TokenKind::is_at);

    fn foo() {
        let a = (Adj | Noun | Det) & !Verb & Punct::COMMA;
        let b = Verb | Det | "very";
    }
}

pub mod prelude {

    pub use super::super::{Pattern, WordSet};
    pub use super::predicates::{
        Adj, Adverb, AuxVerb, Common, Det, Homograph, LinkingVerb, Nominal, NominalPl, Noun,
        NounPl, Prep, Pronoun, PronounPl, Punct, Verb,
    };
    pub use super::{AnyToken, AnyWord, Choice, IntoPattern, Sequence, WS, exact};

    /// Matches a sequence of patterns.
    ///
    /// This is the same as concatenating the patterns together.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use crate::patterns::new_syntax_experiment::preluse::*;
    /// let confession = seq!["I", WS, "love", WS, "you"];
    /// ```
    macro_rules! seq {
        ($item:expr $(,)?) => {
            IntoPattern::into_pattern($item)
        };
        ($($item:expr),* $(,)?) => {
            Sequence::new(vec![$(IntoPattern::into_pattern_boxed($item)),*])
        };
    }
    /// Matches any of the given patterns.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use crate::patterns::new_syntax_experiment::preluse::*;
    /// let fav_animal = choice!["dog", "cat", seq!["black", WS, "bear"]];
    /// ```
    macro_rules! choice {
        ($($item:literal),+ $(,)?) => {
            WordSet::new(&[$($item),*])
        };
        ($($item:expr),* $(,)?) => {
            Choice::new(vec![$(IntoPattern::into_pattern_boxed($item)),*])
        };
    }

    /// An assertion that matches the given sequence of patterns, but does NOT
    /// consume any tokens.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use crate::patterns::new_syntax_experiment::preluse::*;
    /// let love = seq!["I", WS, "love", next![WS, "you"]];
    /// ```
    macro_rules! next {
        ($($item:expr),* $(,)?) => {
            crate::patterns::new_syntax_experiment::ahead(seq![$($item),*])
        };
    }
    /// An assertion that matches anything but the given sequence of patterns.
    /// No tokens are consumed.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use crate::patterns::new_syntax_experiment::preluse::*;
    /// let love_no_ego = seq!["I", WS, "love", not_next![WS, "myself"]];
    /// ```
    macro_rules! not_next {
        ($($item:expr),* $(,)?) => {
            crate::patterns::new_syntax_experiment::not(
                crate::patterns::new_syntax_experiment::ahead(
                    seq![$($item),*]
                )
            )
        };
    }

    pub(crate) use {choice, next, not_next, seq};
}
