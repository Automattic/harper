mod an_a;
mod avoid_curses;
mod boring_words;
mod capitalize_personal_pronouns;
mod correct_number_suffix;
mod currency_placement;
mod dashes;
mod dot_initialisms;
mod ellipsis_length;
mod linking_verbs;
mod lint;
mod lint_group;
mod long_sentences;
mod matcher;
mod merge_linters;
mod merge_words;
mod multiple_sequential_pronouns;
mod number_suffix_capitalization;
mod oxford_comma;
mod pattern_linter;
mod plural_conjugate;
mod pronoun_contraction;
mod proper_noun_capitalization_linters;
mod repeated_words;
mod sentence_capitalization;
mod spaces;
mod spell_check;
mod spelled_numbers;
mod terminating_conjunctions;
mod that_which;
mod unclosed_quotes;
mod use_genitive;
mod wrong_quotes;

pub use an_a::AnA;
pub use avoid_curses::AvoidCurses;
pub use boring_words::BoringWords;
pub use capitalize_personal_pronouns::CapitalizePersonalPronouns;
pub use correct_number_suffix::CorrectNumberSuffix;
pub use currency_placement::CurrencyPlacement;
pub use dot_initialisms::DotInitialisms;
pub use ellipsis_length::EllipsisLength;
pub use linking_verbs::LinkingVerbs;
pub use lint::{Lint, LintKind, Suggestion};
pub use lint_group::{LintGroup, LintGroupConfig};
pub use long_sentences::LongSentences;
pub use matcher::Matcher;
pub use merge_words::MergeWords;
pub use multiple_sequential_pronouns::MultipleSequentialPronouns;
pub use number_suffix_capitalization::NumberSuffixCapitalization;
pub use oxford_comma::OxfordComma;
pub use pattern_linter::PatternLinter;
pub use plural_conjugate::PluralConjugate;
pub use pronoun_contraction::PronounContraction;
pub use proper_noun_capitalization_linters::{
    AmazonNames, Americas, AppleNames, AzureNames, ChineseCommunistParty, GoogleNames, Holidays,
    Koreas, MetaNames, MicrosoftNames, UnitedOrganizations,
};
pub use repeated_words::RepeatedWords;
pub use sentence_capitalization::SentenceCapitalization;
pub use spaces::Spaces;
pub use spell_check::SpellCheck;
pub use spelled_numbers::SpelledNumbers;
pub use terminating_conjunctions::TerminatingConjunctions;
pub use that_which::ThatWhich;
pub use unclosed_quotes::UnclosedQuotes;
pub use use_genitive::UseGenitive;
pub use wrong_quotes::WrongQuotes;

use crate::Document;

#[cfg(not(feature = "concurrent"))]
pub trait Linter {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
    fn description(&self) -> &str;
}

#[cfg(feature = "concurrent")]
pub trait Linter: Send + Sync {
    fn lint(&mut self, document: &Document) -> Vec<Lint>;
    fn description(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::Linter;
    use crate::Document;

    pub fn assert_lint_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        dbg!(&lints);
        assert_eq!(lints.len(), count);
    }

    /// Assert the total number of suggestions produced by a [`Linter`], spread across all produced
    /// [`Lint`]s.
    pub fn assert_suggestion_count(text: &str, mut linter: impl Linter, count: usize) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);
        assert_eq!(
            lints.iter().map(|l| l.suggestions.len()).sum::<usize>(),
            count
        );
    }

    /// Runs a provided linter on text, applies the first suggestion from each
    /// lint and asserts whether the result is equal to a given value.
    pub fn assert_suggestion_result(text: &str, mut linter: impl Linter, expected_result: &str) {
        let test = Document::new_markdown_default_curated(text);
        let lints = linter.lint(&test);

        let mut text: Vec<char> = text.chars().collect();

        for lint in lints {
            dbg!(&lint);
            if let Some(sug) = lint.suggestions.first() {
                sug.apply(lint.span, &mut text);
            }
        }

        let transformed_str: String = text.iter().collect();

        assert_eq!(transformed_str.as_str(), expected_result);

        // Applying the suggestions should fix all the lints.
        assert_lint_count(&transformed_str, linter, 0);
    }
}
