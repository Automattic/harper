use paste::paste;
use serde::{Deserialize, Serialize};

use super::an_a::AnA;
use super::avoid_curses::AvoidCurses;
use super::boring_words::BoringWords;
use super::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::closed_compounds::{
    Altogether, Anybody, Anyhow, Anywhere, Asleep, Backplane, Devops, Downright, Ecommerce,
    Everywhere, Forthwith, Henceforth, Hereafter, Hereby, Hereunder, However, Insofar, Instead,
    Intact, Into, Itself, Likewise, Middleware, Misunderstand, Misunderstood, Misuse, Misused,
    Multicore, Multimedia, Multithreading, Myself, Nobody, Nonetheless, Nothing, Notwithstanding,
    Otherwise, Overall, Overclocking, Overload, Postpone, Proofread, Regardless, Somebody, Somehow,
    Somewhere, Thereafter, Therefore, Therein, Thereupon, Tonight, Underclock, Upset, Upward,
    Whereupon, Widespread, Worldwide,
};
use super::compound_nouns::CompoundNouns;
use super::correct_number_suffix::CorrectNumberSuffix;
use super::despite_of::DespiteOf;
use super::dot_initialisms::DotInitialisms;
use super::ellipsis_length::EllipsisLength;
use super::lets_confusion::LetsConfusion;
use super::linking_verbs::LinkingVerbs;
use super::long_sentences::LongSentences;
use super::matcher::Matcher;
use super::merge_words::MergeWords;
use super::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::number_suffix_capitalization::NumberSuffixCapitalization;
use super::phrase_corrections::{
    AndAlike, BadRap, BatedBreath, BeckAndCall, ChangeTack, EnMasse, HumanLife, HungerPang,
    LetAlone, LoAndBehold, NeedHelp, NoLonger, OfCourse, SneakingSuspicion, SpecialAttention,
    SupposeTo, ThanOthers, ThatChallenged, TurnItOff,
};
use super::plural_conjugate::PluralConjugate;
use super::pronoun_contraction::PronounContraction;
use super::proper_noun_capitalization_linters::{
    AmazonNames, Americas, AppleNames, AzureNames, ChineseCommunistParty, GoogleNames, Holidays,
    Koreas, MetaNames, MicrosoftNames, UnitedOrganizations,
};
use super::repeated_words::RepeatedWords;
use super::sentence_capitalization::SentenceCapitalization;
use super::somewhat_something::SomewhatSomething;
use super::spaces::Spaces;
use super::spell_check::SpellCheck;
use super::spelled_numbers::SpelledNumbers;
use super::terminating_conjunctions::TerminatingConjunctions;
use super::that_which::ThatWhich;
use super::unclosed_quotes::UnclosedQuotes;
use super::use_genitive::UseGenitive;
use super::whereas::Whereas;
use super::wrong_quotes::WrongQuotes;
use super::{CurrencyPlacement, Lint, Linter, NoOxfordComma, OxfordComma};
use crate::{Dictionary, Document};

macro_rules! create_lint_group_config {
    ($($linter:ident => $default:expr),* $(,)?) => {
        paste! {
            /// A collection of all the descriptions from the composing linters.
            #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
            pub struct LintGroupDescriptions<'a> {
                $(
                    #[doc = "The description for the [`" $linter "`] linter."]
                    pub [<$linter:snake>]: &'a str,
                )*
                pub spell_check: &'a str
            }


            impl<'a>  LintGroupDescriptions<'a> {
                /// Create a [`Vec`] containing the key-value pairs of this struct.
                pub fn to_vec_pairs(self) -> Vec<(&'static str, &'a str)>{
                    vec![$((stringify!([<$linter:snake>]), self.[<$linter:snake>],),)* ("spell_check", self.spell_check)]
                }
            }

            /// A collection of all officially supported
            #[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
            pub struct LintGroupConfig {
                $(
                    #[doc = "Configures the use of the [`" $linter "`] linter.
                    If set to [`None`], the default configuration will be used."]
                    pub [<$linter:snake>]: Option<bool>,
                )*
                pub spell_check: Option<bool>
            }

            impl LintGroupConfig {
                /// Creates a config with all lints disabled.
                pub fn none() -> Self{
                    Self {
                        $(
                            [<$linter:snake>]: Some(false),
                        )*
                        spell_check: Some(false)
                    }
                }

                /// Fills the [`None`] values in the configuration with the default values.
                pub fn fill_default_values(&mut self){
                    $(
                        if self.[<$linter:snake>].is_none() {
                            self.[<$linter:snake>] = Some($default);
                        }
                    )*

                    if self.spell_check.is_none() {
                        self.spell_check = Some(true);
                    }
                }
            }

            /// A wrapper that combines all built-in Harper linters
            /// into a single, configurable [`Linter`].
            pub struct LintGroup<T: Dictionary> {
                $(
                    [<$linter:snake>]: $linter,
                )*
                spell_check: SpellCheck<T>,
                pub config: LintGroupConfig
            }


            impl<T: Dictionary> LintGroup<T> {
                pub fn new(config: LintGroupConfig, dictionary: T) -> Self {
                    Self {
                        $(
                            [<$linter:snake>]: $linter::default(),
                        )*
                        spell_check: SpellCheck::new(dictionary),
                        config,
                    }
                }

                pub fn all_descriptions(&self) -> LintGroupDescriptions<'_> {
                    LintGroupDescriptions {
                        $(
                            [<$linter:snake>]: self.[<$linter:snake>].description(),
                        )*
                        spell_check: self.spell_check.description(),
                    }
                }
            }

            impl<T: Dictionary> Linter for LintGroup<T> {
                fn lint(&mut self, document: &Document) -> Vec<Lint> {
                    let mut lints = Vec::new();

                    let mut config = self.config.clone();
                    config.fill_default_values();

                    $(
                        if config.[<$linter:snake>].unwrap() {
                            lints.append(&mut self.[<$linter:snake>].lint(document));
                        }
                    )*

                    if config.spell_check.unwrap() {
                        lints.append(&mut self.spell_check.lint(document));
                    }


                    lints
                }

                fn description(&self) -> &'static str {
                    "A collection of linters that can be run as one."
                }
            }
        }
    };
}

create_lint_group_config!(
    CompoundNouns => true,
    Regardless => true,
    Henceforth => true,
    Upward => true,
    Downright => true,
    Thereafter => true,
    Whereupon => true,
    Insofar => true,
    Forthwith => true,
    Hereunder => true,
    Hereby => true,
    Thereupon => true,
    Therein => true,
    Otherwise => true,
    Hereafter => true,
    Nonetheless => true,
    Anyhow => true,
    Notwithstanding => true,
    Widespread => true,
    Ecommerce => true,
    Multimedia => true,
    Multicore => true,
    Multithreading => true,
    Devops => true,
    Underclock => true,
    Overload => true,
    Backplane => true,
    Overclocking => true,
    Middleware => true,
    Somewhere => true,
    Instead => true,
    Anywhere => true,
    Nothing => true,
    Anybody => true,
    Somebody => true,
    Nobody => true,
    Into => true,
    Proofread => true,
    Somehow => true,
    Intact => true,
    Upset => true,
    Misunderstood => true,
    Asleep => true,
    Altogether => true,
    However => true,
    Likewise => true,
    Overall => true,
    Worldwide => true,
    Postpone => true,
    Misused => true,
    Misuse => true,
    Misunderstand => true,
    Therefore => true,
    Myself => true,
    Tonight => true,
    Itself => true,
    Whereas => true,
    SpelledNumbers => false,
    AnA => true,
    SentenceCapitalization => true,
    UnclosedQuotes => true,
    WrongQuotes => false,
    LongSentences => true,
    RepeatedWords => true,
    Spaces => true,
    Matcher => true,
    CorrectNumberSuffix => true,
    NumberSuffixCapitalization => true,
    MultipleSequentialPronouns => true,
    LinkingVerbs => false,
    AvoidCurses => true,
    TerminatingConjunctions => true,
    EllipsisLength => true,
    DotInitialisms => true,
    BoringWords => false,
    UseGenitive => false,
    ThatWhich => true,
    CapitalizePersonalPronouns => true,
    Americas => true,
    Koreas => true,
    ChineseCommunistParty => true,
    UnitedOrganizations => true,
    Holidays => true,
    AmazonNames => true,
    GoogleNames => true,
    MetaNames => true,
    MicrosoftNames => true,
    AppleNames => true,
    AzureNames => true,
    MergeWords => true,
    PluralConjugate => false,
    OxfordComma => true,
    NoOxfordComma => true,
    PronounContraction => true,
    CurrencyPlacement => true,
    SomewhatSomething => true,
    LetsConfusion => true,
    DespiteOf => true,
    HumanLife => true,
    NeedHelp => true,
    NoLonger => true,
    ThatChallenged => true,
    TurnItOff => true,
    OfCourse => true,
    AndAlike => true,
    BadRap => true,
    BatedBreath => true,
    BeckAndCall => true,
    ChangeTack => true,
    HungerPang => true,
    EnMasse => true,
    LetAlone => true,
    LoAndBehold => true,
    SneakingSuspicion => true,
    SupposeTo => true,
    SpecialAttention => true,
    Everywhere => true,
    ThanOthers => true,
);

impl<T: Dictionary + Default> Default for LintGroup<T> {
    fn default() -> Self {
        Self::new(LintGroupConfig::default(), T::default())
    }
}

#[cfg(test)]
mod tests {
    use crate::{linting::Linter, Document, FstDictionary, FullDictionary};

    use super::{LintGroup, LintGroupConfig};

    #[test]
    fn can_get_all_descriptions() {
        let group = LintGroup::<FullDictionary>::default();
        group.all_descriptions();
    }

    #[test]
    fn lint_descriptions_are_clean() {
        let mut group = LintGroup::new(LintGroupConfig::default(), FstDictionary::curated());
        let pairs: Vec<_> = group
            .all_descriptions()
            .to_vec_pairs()
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        for (key, value) in pairs {
            let doc = Document::new_markdown_default_curated(&value);
            eprintln!("{key}: {value}");
            assert!(group.lint(&doc).is_empty())
        }
    }
}
