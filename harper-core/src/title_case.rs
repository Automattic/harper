use std::borrow::Cow;

use crate::Lrc;
use crate::Token;
use crate::TokenKind;
use hashbrown::HashSet;
use lazy_static::lazy_static;

use crate::spell::Dictionary;
use crate::{CharStringExt, Document, TokenStringExt, parsers::Parser};

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

    let start_index = toks.first().unwrap().span.start;

    let mut word_likes = toks.iter_word_likes().enumerate().peekable();
    let mut output = toks.span().unwrap().get_content(source).to_vec();

    while let Some((index, word)) = word_likes.next() {
        if let Some(Some(metadata)) = word.kind.as_word()
            && metadata.is_proper_noun()
        {
            // Replace it with the dictionary entry verbatim.
            let orig_text = word.span.get_content(source);

            if let Some(correct_caps) = dict.get_correct_capitalization_of(orig_text) {
                // It should match the dictionary verbatim
                output[word.span.start - start_index..word.span.end - start_index]
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, c)| *c = correct_caps[idx]);
            }
        };

        let should_capitalize = should_capitalize_token(word, source, dict)
            || index == 0
            || word_likes.peek().is_none();

        if should_capitalize {
            output[word.span.start - start_index] =
                output[word.span.start - start_index].to_ascii_uppercase();
        } else {
            // The whole word should be lowercase.
            for i in word.span {
                output[i - start_index] = output[i - start_index].to_ascii_lowercase();
            }
        }
    }

    output
}

/// Determines whether a token should be capitalized.
/// Is not responsible for capitalization requirements that are dependent on token position.
fn should_capitalize_token(tok: &Token, source: &[char], dict: &impl Dictionary) -> bool {
    match &tok.kind {
        TokenKind::Word(Some(metadata)) => {
            // Only specific conjunctions are not capitalized.
            lazy_static! {
                static ref SPECIAL_CONJUNCTIONS: HashSet<Vec<char>> =
                    ["and", "but", "for", "or", "nor"]
                        .iter()
                        .map(|v| v.chars().collect())
                        .collect();
            }

            let chars = tok.span.get_content(source);
            let chars_lower = chars.to_lower();

            let mut metadata = Cow::Borrowed(metadata);

            if let Some(metadata_lower) = dict.get_word_metadata(&chars_lower) {
                metadata = Cow::Owned(metadata.clone().or(metadata_lower));
            }

            let is_short_preposition = metadata.preposition && tok.span.len() <= 4;

            !is_short_preposition
                && !metadata.is_determiner()
                && !SPECIAL_CONJUNCTIONS.contains(chars_lower.as_ref())
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::make_title_case_str;
    use crate::parsers::{Markdown, PlainEnglish};
    use crate::spell::FstDictionary;

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

    /// Check that "about" remains uppercase
    #[test]
    fn about_uppercase_with_numbers() {
        assert_eq!(
            make_title_case_str("0 about 0", &PlainEnglish, &FstDictionary::curated()),
            "0 About 0"
        )
    }

    #[test]
    fn pipe_does_not_cause_crash() {
        assert_eq!(
            make_title_case_str("|", &Markdown::default(), &FstDictionary::curated()),
            "|"
        )
    }

    #[test]
    fn a_paragraph_does_not_cause_crash() {
        assert_eq!(
            make_title_case_str("A\n", &Markdown::default(), &FstDictionary::curated()),
            "A"
        )
    }

    #[test]
    fn tab_a_becomes_upcase() {
        assert_eq!(
            make_title_case_str("\ta", &PlainEnglish, &FstDictionary::curated()),
            "\tA"
        )
    }

    #[test]
    fn fixes_video_press() {
        assert_eq!(
            make_title_case_str("videopress", &PlainEnglish, &FstDictionary::curated()),
            "VideoPress"
        )
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
    fn first_word_is_upcase(text: String) -> TestResult {
        let title_case: Vec<_> =
            make_title_case_str(&text, &PlainEnglish, &FstDictionary::curated())
                .chars()
                .collect();

        if let Some(first) = title_case.first() {
            if first.is_ascii_alphabetic() {
                TestResult::from_bool(first.is_ascii_uppercase())
            } else {
                TestResult::discard()
            }
        } else {
            TestResult::discard()
        }
    }

    #[test]
    fn united_states() {
        assert_eq!(
            make_title_case_str("united states", &PlainEnglish, &FstDictionary::curated()),
            "United States"
        )
    }
}
