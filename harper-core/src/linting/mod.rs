//! Frameworks and rules that locate errors in text.
//!
//! See the [`Linter`] trait and the [documentation for authoring a rule](https://writewithharper.com/docs/contributors/author-a-rule) for more information.

mod adjective_of_a;
mod an_a;
mod avoid_curses;
mod back_in_the_day;
mod boring_words;
mod capitalize_personal_pronouns;
mod chock_full;
mod closed_compounds;
mod comma_fixes;
mod compound_nouns;
mod confident;
mod correct_number_suffix;
mod currency_placement;
mod dashes;
mod despite_of;
mod dot_initialisms;
mod ellipsis_length;
mod expand_time_shorthands;
mod first_aid_kit;
mod for_noun;
mod hedging;
mod hereby;
mod hop_hope;
mod hyphenate_number_day;
mod inflected_verb_after_to;
mod it_is;
mod it_would_be;
mod left_right_hand;
mod lets_confusion;
mod likewise;
mod linking_verbs;
mod lint;
mod lint_group;
mod lint_kind;
mod long_sentences;
mod map_phrase_linter;
mod merge_linters;
mod merge_words;
mod modal_of;
mod multiple_sequential_pronouns;
mod no_oxford_comma;
mod nobody;
mod number_suffix_capitalization;
mod of_course;
mod out_of_date;
mod oxford_comma;
mod oxymorons;
mod pattern_linter;
mod phrase_corrections;
mod pique_interest;
mod possessive_your;
mod pronoun_contraction;
mod pronoun_knew;
mod proper_noun_capitalization_linters;
mod repeated_words;
mod sentence_capitalization;
mod somewhat_something;
mod spaces;
mod spell_check;
mod spelled_numbers;
mod suggestion;
mod that_which;
mod the_how_why;
mod the_my;
mod then_than;
mod unclosed_quotes;
mod use_genitive;
mod was_aloud;
mod whereas;
mod widely_accepted;
mod wordpress_dotcom;

pub use adjective_of_a::AdjectiveOfA;
pub use an_a::AnA;
pub use avoid_curses::AvoidCurses;
pub use back_in_the_day::BackInTheDay;
pub use boring_words::BoringWords;
pub use capitalize_personal_pronouns::CapitalizePersonalPronouns;
pub use chock_full::ChockFull;
pub use comma_fixes::CommaFixes;
pub use compound_nouns::CompoundNouns;
pub use confident::Confident;
pub use correct_number_suffix::CorrectNumberSuffix;
pub use currency_placement::CurrencyPlacement;
pub use dashes::Dashes;
pub use despite_of::DespiteOf;
pub use dot_initialisms::DotInitialisms;
pub use ellipsis_length::EllipsisLength;
pub use expand_time_shorthands::ExpandTimeShorthands;
pub use for_noun::ForNoun;
pub use hedging::Hedging;
pub use hereby::Hereby;
pub use hop_hope::HopHope;
pub use hyphenate_number_day::HyphenateNumberDay;
pub use inflected_verb_after_to::InflectedVerbAfterTo;
pub use left_right_hand::LeftRightHand;
pub use lets_confusion::LetsConfusion;
pub use likewise::Likewise;
pub use linking_verbs::LinkingVerbs;
pub use lint::Lint;
pub use lint_group::{LintGroup, LintGroupConfig};
pub use lint_kind::LintKind;
pub use long_sentences::LongSentences;
pub use map_phrase_linter::MapPhraseLinter;
pub use merge_words::MergeWords;
pub use modal_of::ModalOf;
pub use multiple_sequential_pronouns::MultipleSequentialPronouns;
pub use no_oxford_comma::NoOxfordComma;
pub use nobody::Nobody;
pub use number_suffix_capitalization::NumberSuffixCapitalization;
pub use of_course::OfCourse;
pub use out_of_date::OutOfDate;
pub use oxford_comma::OxfordComma;
pub use oxymorons::Oxymorons;
pub use pattern_linter::PatternLinter;
pub use pique_interest::PiqueInterest;
pub use possessive_your::PossessiveYour;
pub use pronoun_contraction::PronounContraction;
pub use repeated_words::RepeatedWords;
pub use sentence_capitalization::SentenceCapitalization;
pub use somewhat_something::SomewhatSomething;
pub use spaces::Spaces;
pub use spell_check::SpellCheck;
pub use spelled_numbers::SpelledNumbers;
pub use suggestion::Suggestion;
pub use that_which::ThatWhich;
pub use the_how_why::TheHowWhy;
pub use the_my::TheMy;
pub use then_than::ThenThan;
pub use unclosed_quotes::UnclosedQuotes;
pub use use_genitive::UseGenitive;
pub use was_aloud::WasAloud;
pub use whereas::Whereas;
pub use widely_accepted::WidelyAccepted;
pub use wordpress_dotcom::WordPressDotcom;

use crate::{Document, LSend};

/// A __stateless__ rule that searches documents for grammatical errors.
///
/// Commonly implemented via [`PatternLinter`].
///
/// See also: [`LintGroup`].
pub trait Linter: LSend {
    /// Analyzes a document and produces zero or more [`Lint`]s.
    /// We pass `self` mutably for caching purposes.
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
    /// A user-facing description of what kinds of grammatical errors this rule looks for.
    /// It is usually shown in settings menus.
    fn description(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::{Document, FstDictionary, parsers::PlainEnglish};

    #[track_caller]
    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        dbg!(&lints);
        if lints.len() != count {
            panic!(
                "Expected \"{text}\" to create {count} lints, but it created {}.",
                lints.len()
            );
        }
    }

    /// Assert the total number of suggestions produced by a [`Linter`], spread across all produced
    /// [`Lint`]s.
    #[track_caller]
    pub fn assert_suggestion_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        assert_eq!(
            lints.iter().map(|l| l.suggestions.len()).sum::<usize>(),
            count
        );
    }

    /// Runs a provided linter on text, applies the nth suggestion from each lint
    /// and asserts whether the result is equal to a given value.
    ///
    /// Note that `n` starts at zero.
    #[track_caller]
    pub fn assert_nth_suggestion_result(
        text: &str,
        mut linter: impl Linter,
        expected_result: &str,
        n: usize,
    ) {
        let mut text_chars: Vec<char> = text.chars().collect();

        let mut iter_count = 0;

        loop {
            iter_count += 1;

            let test = Document::new_from_vec(
                text_chars.clone().into(),
                &PlainEnglish,
                &FstDictionary::curated(),
            );
            let lints = linter.lint(&test);

            if let Some(lint) = lints.first() {
                if let Some(sug) = lint.suggestions.get(n) {
                    sug.apply(lint.span, &mut text_chars);

                    let transformed_str: String = text_chars.iter().collect();
                    dbg!(transformed_str);
                } else {
                    break;
                }
            } else {
                break;
            }

            if iter_count == 100 {
                break;
            }
        }

        eprintln!("Corrected {} times.", iter_count);

        let transformed_str: String = text_chars.iter().collect();

        if transformed_str.as_str() != expected_result {
            panic!(
                "Expected \"{transformed_str}\" to be \"{expected_result}\" after applying the computed suggestions."
            );
        }

        // Applying the suggestions should fix all the lints.
        assert_lint_count(&transformed_str, linter, 0);
    }

    /// Runs a provided linter on text, applies the first suggestion from each lint
    /// and asserts whether the result is equal to a given value.
    #[track_caller]
    pub fn assert_suggestion_result(text: &str, linter: impl Linter, expected_result: &str) {
        assert_nth_suggestion_result(text, linter, expected_result, 0);
    }
}
