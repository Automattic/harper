use crate::Lrc;
use crate::Token;
use hashbrown::HashSet;
use lazy_static::lazy_static;

use crate::{parsers::Parser, CharStringExt, Dictionary, Document, TokenStringExt};

/// A helper function for [`make_title_case`] that uses Strings instead of char buffers.
pub fn make_title_case_str(source: &str, parser: &impl Parser, dict: &impl Dictionary) -> String {
    let source: Vec<char> = source.chars().collect();

    make_title_case_chars(Lrc::new(source), parser, dict).to_string()
}

// Make a given string [title case](https://en.wikipedia.org/wiki/Title_case) following the Chicago Manual of Style.
pub fn make_title_case_chars(
    source: Lrc<Vec<char>>,
    parser: &impl Parser,
    dict: &impl Dictionary,
) -> Vec<char> {
    let document = Document::new_from_vec(source.clone(), parser, dict);

    make_title_case(document.get_tokens(), source.as_slice(), dict)
}

pub fn make_title_case(toks: &[Token], source: &[char], dict: &impl Dictionary) -> Vec<char> {
    if toks.is_empty() {
        return Vec::new();
    }

    let mut words = toks.iter_word_likes().enumerate().peekable();
    let mut output = source.to_vec();

    // Only specific conjunctions are not capitalized.
    lazy_static! {
        static ref SPECIAL_CONJUNCTIONS: HashSet<Vec<char>> = ["and", "but", "for", "or", "nor"]
            .iter()
            .map(|v| v.chars().collect())
            .collect();
    }

    while let Some((index, word)) = words.next() {
        if !word.kind.is_word() {
            continue;
        }

        let chars = word.span.get_content(source);
        let chars_lower = chars.to_lower();

        let metadata = word
            .kind
            .as_word()
            .unwrap()
            .or(&dict.get_word_metadata(&chars_lower));

        let should_capitalize = !metadata.preposition
            && !metadata.article
            && !SPECIAL_CONJUNCTIONS.contains(chars_lower.as_slice())
            || index == 0
            || words.peek().is_none();

        if should_capitalize {
            output[word.span.start] = output[word.span.start].to_ascii_uppercase();

            // The rest of the word should be lowercase.
            for v in &mut output[word.span.start + 1..word.span.end] {
                *v = v.to_ascii_lowercase();
            }
        } else {
            // The whole word should be lowercase.
            for i in word.span {
                output[i] = output[i].to_ascii_lowercase();
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {

    use quickcheck::{Arbitrary, TestResult};
    use quickcheck_macros::quickcheck;

    use super::make_title_case_str;
    use crate::{
        parsers::{Markdown, PlainEnglish},
        FstDictionary,
    };

    #[test]
    fn normal() {
        assert_eq!(
            make_title_case_str("this is a test", &PlainEnglish, &FstDictionary::curated()),
            "This Is a Test"
        )
    }

    #[test]
    fn complex() {
        assert_eq!(
            make_title_case_str(
                "the first and last words should be capitalized, even if it is \"the\"",
                &PlainEnglish,
                &FstDictionary::curated()
            ),
            "The First and Last Words Should Be Capitalized, Even If It Is \"The\""
        )
    }

    #[test]
    fn start_as_uppercase() {
        assert_eq!(
            make_title_case_str("THIS IS A TEST", &PlainEnglish, &FstDictionary::curated()),
            "This Is a Test"
        )
    }

    #[test]
    fn span_breaks_in_source() {
        use super::make_title_case;
        use crate::{CharStringExt, Span, Token, TokenKind, WordMetadata};
        use itertools::Itertools;

        let source = "// linted
// linted";
        let source = source.chars().collect_vec();

        let toks = &[
            Token::new(Span::new(3, 9), TokenKind::Word(WordMetadata::default())),
            Token::new(Span::new(9, 10), TokenKind::Newline(1)),
            Token::new(Span::new(13, 19), TokenKind::Word(WordMetadata::default())),
        ];

        assert_eq!(
            make_title_case(toks, &source, &FstDictionary::curated()).to_string(),
            "// Linted
// Linted"
        )
    }

    #[derive(Debug, Clone)]
    struct Word(String);

    impl Arbitrary for Word {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut s = String::new();

            for _ in 0..g.size() {
                let c = loop {
                    let gen = char::arbitrary(g);

                    if gen.is_ascii_alphanumeric() {
                        break gen;
                    }
                };

                s.push(c);
            }

            Self(s)
        }
    }

    #[derive(Debug, Clone)]
    struct Sentence(String);

    /// Builds a sentence out of words from the curated [`FullDictionary`].
    impl Arbitrary for Sentence {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut s = String::new();

            let Word(first_word) = Word::arbitrary(g);
            s.push_str(&first_word);

            for _ in 0..g.size() {
                let Word(word) = Word::arbitrary(g);

                s.push(' ');
                s.push_str(&word);
            }

            Self(s)
        }
    }

    #[quickcheck]
    fn a_stays_lowercase(prefix: String, postfix: String) -> TestResult {
        // There must be words other than the `a`.
        if prefix.chars().any(|c| !c.is_ascii_alphanumeric())
            || prefix.is_empty()
            || postfix.chars().any(|c| !c.is_ascii_alphanumeric())
            || postfix.is_empty()
        {
            return TestResult::discard();
        }

        let title_case: Vec<_> = make_title_case_str(
            &format!("{prefix} a {postfix}"),
            &Markdown::default(),
            &FstDictionary::curated(),
        )
        .chars()
        .collect();

        dbg!(&title_case);

        TestResult::from_bool(title_case[prefix.chars().count() + 1] == 'a')
    }

    #[quickcheck]
    fn about_becomes_uppercase(prefix: String, postfix: String) -> TestResult {
        // There must be words other than the `a`.
        if prefix.chars().any(|c| !c.is_ascii_alphanumeric())
            || prefix.is_empty()
            || postfix.chars().any(|c| !c.is_ascii_alphanumeric())
            || postfix.is_empty()
        {
            return TestResult::discard();
        }

        let title_case: Vec<_> = make_title_case_str(
            &format!("{prefix} about {postfix}"),
            &Markdown::default(),
            &FstDictionary::curated(),
        )
        .chars()
        .collect();

        TestResult::from_bool(title_case[prefix.chars().count() + 1] == 'A')
    }

    #[quickcheck]
    fn first_word_is_upcase(sentence: Sentence) -> TestResult {
        let title_case: Vec<_> =
            make_title_case_str(&sentence.0, &Markdown::default(), &FstDictionary::curated())
                .chars()
                .collect();

        if let Some(first) = title_case.first() {
            if first.is_alphabetic() {
                TestResult::from_bool(first.is_uppercase())
            } else {
                TestResult::discard()
            }
        } else {
            TestResult::discard()
        }
    }
}
