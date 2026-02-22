use std::borrow::Cow;
use std::sync::LazyLock;

use crate::Lrc;
use crate::Token;
use crate::TokenKind;
use hashbrown::HashSet;
use itertools::Itertools;

use crate::Punctuation;
use crate::spell::{CommonDictFuncs, Dictionary, WordMap};
use crate::{CharStringExt, Document, TokenStringExt, parsers::Parser};

/// A helper function for [`make_title_case`] that uses Strings instead of char buffers.
pub fn make_title_case_str(source: &str, parser: &impl Parser, dict: &WordMap) -> String {
    let source: Vec<char> = source.chars().collect();

    make_title_case_chars(Lrc::new(source), parser, dict).to_string()
}

// Make a given string [title case](https://en.wikipedia.org/wiki/Title_case) following the Chicago Manual of Style.
pub fn make_title_case_chars(
    source: Lrc<Vec<char>>,
    parser: &impl Parser,
    dict: &WordMap,
) -> Vec<char> {
    let document = Document::new_from_vec(source.clone(), parser, dict.get_word_map());

    make_title_case(document.get_tokens(), source.as_slice(), dict)
}

pub fn try_make_title_case(toks: &[Token], source: &[char], dict: &WordMap) -> Option<Vec<char>> {
    if toks.is_empty() {
        return None;
    }

    let start_index = toks.first().unwrap().span.start;
    let relevant_text = toks.span().unwrap().get_content(source);

    let mut word_likes = toks.iter_word_like_indices().enumerate().peekable();

    let mut output = None;
    let mut previous_word_index = 0;

    // Checks if the output if the provided char is different from the source. If so, it will
    // set the output. The goal here is to avoid allocating if no edits must be made.
    let mut set_output_char = |idx: usize, new_char: char| {
        if output
            .as_ref()
            .is_some_and(|o: &Vec<char>| o[idx] != new_char)
            || relevant_text[idx] != new_char
        {
            if output.is_none() {
                output = Some(relevant_text.to_vec())
            }

            let Some(mutable) = &mut output else {
                panic!("We just set output to `Some`. This should be impossible.");
            };

            mutable[idx] = new_char;
        }
    };

    while let Some((index, word_idx)) = word_likes.next() {
        let word = &toks[word_idx];

        if let Some(Some(metadata)) = word.kind.as_word()
            && metadata.is_proper_noun()
        {
            // Replace it with the dictionary entry verbatim.
            let orig_text = word.span.get_content(source);

            if let Ok(correct_caps) = dict.get_correct_capitalizations_of(orig_text).exactly_one() {
                // It should match the dictionary verbatim
                for (i, c) in correct_caps.iter().enumerate() {
                    if c.is_alphabetic() {
                        set_output_char(word.span.start - start_index + i, *c);
                    }
                }
            }
        };

        // Capitalize the first word following a colon to match Chicago style.
        let is_after_colon = toks[previous_word_index..word_idx]
            .iter()
            .any(|tok| matches!(tok.kind, TokenKind::Punctuation(Punctuation::Colon)));

        let should_capitalize = is_after_colon
            || should_capitalize_token(word, source)
            || index == 0
            || word_likes.peek().is_none();

        if should_capitalize {
            set_output_char(
                word.span.start - start_index,
                relevant_text[word.span.start - start_index].to_ascii_uppercase(),
            );
        } else {
            // The whole word should be lowercase.
            for i in word.span {
                set_output_char(
                    i - start_index,
                    relevant_text[i - start_index].to_ascii_lowercase(),
                );
            }
        }

        previous_word_index = word_idx
    }

    if let Some(output) = &output
        && output.as_slice() == relevant_text
    {
        return None;
    }

    output
}

pub fn make_title_case(toks: &[Token], source: &[char], dict: &WordMap) -> Vec<char> {
    try_make_title_case(toks, source, dict)
        .unwrap_or_else(|| toks.span().unwrap_or_default().get_content(source).to_vec())
}

/// Determines whether a token should be capitalized.
/// Is not responsible for capitalization requirements that are dependent on token position.
fn should_capitalize_token(tok: &Token, source: &[char]) -> bool {
    match &tok.kind {
        TokenKind::Word(Some(metadata)) => {
            // Only specific conjunctions are not capitalized.
            static SPECIAL_CONJUNCTIONS: LazyLock<HashSet<Vec<char>>> = LazyLock::new(|| {
                ["and", "but", "for", "or", "nor", "as"]
                    .iter()
                    .map(|v| v.chars().collect())
                    .collect()
            });
            static SPECIAL_ARTICLES: LazyLock<HashSet<Vec<char>>> = LazyLock::new(|| {
                ["a", "an", "the"]
                    .iter()
                    .map(|v| v.chars().collect())
                    .collect()
            });

            let chars = tok.span.get_content(source);
            let chars_lower = chars.to_lower();

            let metadata = Cow::Borrowed(metadata);

            let is_short_preposition = metadata.preposition && tok.span.len() <= 4;

            if chars_lower.as_ref() == ['a', 'l', 'l'] {
                return true;
            }

            !is_short_preposition
                && !SPECIAL_CONJUNCTIONS.contains(chars_lower.as_ref())
                && !SPECIAL_ARTICLES.contains(chars_lower.as_ref())
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
    use crate::spell::WordMap;

    #[test]
    fn normal() {
        assert_eq!(
            make_title_case_str("this is a test", &PlainEnglish, &WordMap::curated()),
            "This Is a Test"
        )
    }

    #[test]
    fn complex() {
        assert_eq!(
            make_title_case_str(
                "the first and last words should be capitalized, even if it is \"the\"",
                &PlainEnglish,
                &WordMap::curated()
            ),
            "The First and Last Words Should Be Capitalized, Even If It Is \"The\""
        )
    }

    /// Check that "about" remains uppercase
    #[test]
    fn about_uppercase_with_numbers() {
        assert_eq!(
            make_title_case_str("0 about 0", &PlainEnglish, WordMap::curated()),
            "0 About 0"
        )
    }

    #[test]
    fn pipe_does_not_cause_crash() {
        assert_eq!(
            make_title_case_str("|", &Markdown::default(), WordMap::curated()),
            "|"
        )
    }

    #[test]
    fn a_paragraph_does_not_cause_crash() {
        assert_eq!(
            make_title_case_str("A\n", &Markdown::default(), WordMap::curated()),
            "A"
        )
    }

    #[test]
    fn tab_a_becomes_upcase() {
        assert_eq!(
            make_title_case_str("\ta", &PlainEnglish, WordMap::curated()),
            "\tA"
        )
    }

    #[test]
    fn fixes_video_press() {
        assert_eq!(
            make_title_case_str("videopress", &PlainEnglish, WordMap::curated()),
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
            WordMap::curated(),
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
            WordMap::curated(),
        )
        .chars()
        .collect();

        TestResult::from_bool(title_case[prefix.chars().count() + 1] == 'A')
    }

    #[quickcheck]
    fn first_word_is_upcase(text: String) -> TestResult {
        let title_case: Vec<_> = make_title_case_str(&text, &PlainEnglish, WordMap::curated())
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
            make_title_case_str("united states", &PlainEnglish, WordMap::curated()),
            "United States"
        )
    }

    #[test]
    fn keeps_decimal() {
        assert_eq!(
            make_title_case_str("harper turns 1.0 today", &PlainEnglish, WordMap::curated()),
            "Harper Turns 1.0 Today"
        )
    }

    #[test]
    fn fixes_odd_capitalized_proper_nouns() {
        assert_eq!(
            make_title_case_str(
                "i spoke at wordcamp u.s. in 2025",
                &PlainEnglish,
                WordMap::curated()
            ),
            "I Spoke at WordCamp U.S. in 2025",
        );
    }

    #[test]
    fn fixes_your_correctly() {
        assert_eq!(
            make_title_case_str("it is not your friend", &PlainEnglish, WordMap::curated()),
            "It Is Not Your Friend",
        );
    }

    #[test]
    fn handles_old_man_and_the_sea() {
        assert_eq!(
            make_title_case_str("the old man and the sea", &PlainEnglish, WordMap::curated()),
            "The Old Man and the Sea",
        );
    }

    #[test]
    fn handles_great_story_with_subtitle() {
        assert_eq!(
            make_title_case_str(
                "the great story: a tale of two cities",
                &PlainEnglish,
                WordMap::curated()
            ),
            "The Great Story: A Tale of Two Cities",
        );
    }

    #[test]
    fn handles_lantern_and_moths() {
        assert_eq!(
            make_title_case_str(
                "lantern flickered; moths began their worship",
                &PlainEnglish,
                WordMap::curated()
            ),
            "Lantern Flickered; Moths Began Their Worship",
        );
    }

    #[test]
    fn handles_static_with_ghosts() {
        assert_eq!(
            make_title_case_str(
                "static filled the room with ghosts",
                &PlainEnglish,
                WordMap::curated()
            ),
            "Static Filled the Room with Ghosts",
        );
    }

    #[test]
    fn handles_glass_trembled_before_thunder() {
        assert_eq!(
            make_title_case_str(
                "glass trembled before thunder arrived.",
                &PlainEnglish,
                WordMap::curated()
            ),
            "Glass Trembled Before Thunder Arrived.",
        );
    }

    #[test]
    fn handles_hepatitis_b_shots() {
        assert_eq!(
            make_title_case_str(
                "an end to hepatitis b shots for all newborns",
                &PlainEnglish,
                WordMap::curated()
            ),
            "An End to Hepatitis B Shots for All Newborns",
        );
    }

    #[test]
    fn handles_trump_approval_rating() {
        assert_eq!(
            make_title_case_str(
                "trump's approval rating dips as views of his handling of the economy sour",
                &PlainEnglish,
                WordMap::curated()
            ),
            "Trump's Approval Rating Dips as Views of His Handling of the Economy Sour",
        );
    }

    #[test]
    fn handles_last_door() {
        assert_eq!(
            make_title_case_str("the last door", &PlainEnglish, WordMap::curated()),
            "The Last Door",
        );
    }

    #[test]
    fn handles_midnight_river() {
        assert_eq!(
            make_title_case_str("midnight river", &PlainEnglish, WordMap::curated()),
            "Midnight River",
        );
    }

    #[test]
    fn handles_a_quiet_room() {
        assert_eq!(
            make_title_case_str("a quiet room", &PlainEnglish, WordMap::curated()),
            "A Quiet Room",
        );
    }

    #[test]
    fn handles_broken_map() {
        assert_eq!(
            make_title_case_str("broken map", &PlainEnglish, WordMap::curated()),
            "Broken Map",
        );
    }

    #[test]
    fn handles_fire_in_autumn() {
        assert_eq!(
            make_title_case_str("fire in autumn", &PlainEnglish, WordMap::curated()),
            "Fire in Autumn",
        );
    }

    #[test]
    fn handles_hidden_path() {
        assert_eq!(
            make_title_case_str("the hidden path", &PlainEnglish, WordMap::curated()),
            "The Hidden Path",
        );
    }

    #[test]
    fn handles_under_blue_skies() {
        assert_eq!(
            make_title_case_str("under blue skies", &PlainEnglish, WordMap::curated()),
            "Under Blue Skies",
        );
    }

    #[test]
    fn handles_lost_and_found() {
        assert_eq!(
            make_title_case_str("lost and found", &PlainEnglish, WordMap::curated()),
            "Lost and Found",
        );
    }

    #[test]
    fn handles_silent_watcher() {
        assert_eq!(
            make_title_case_str("the silent watcher", &PlainEnglish, WordMap::curated()),
            "The Silent Watcher",
        );
    }

    #[test]
    fn handles_winter_road() {
        assert_eq!(
            make_title_case_str("winter road", &PlainEnglish, WordMap::curated()),
            "Winter Road",
        );
    }

    #[test]
    fn maintains_same_apostrophe_type() {
        assert_eq!(
            make_title_case_str(
                "Alice’s Adventures in Wonderland",
                &PlainEnglish,
                WordMap::curated()
            ),
            "Alice’s Adventures in Wonderland",
        );
    }

    #[test]
    fn doesnt_lowercase_this_in_github_template_title() {
        assert_eq!(
            make_title_case_str(
                "# How Has This Been Tested?",
                &PlainEnglish,
                WordMap::curated()
            ),
            "# How Has This Been Tested?",
        );
    }
}
