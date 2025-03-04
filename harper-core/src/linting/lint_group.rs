use std::collections::BTreeMap;
use std::mem;
use std::sync::Arc;

use cached::proc_macro::cached;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::Lint;
use super::an_a::AnA;
use super::avoid_curses::AvoidCurses;
use super::back_in_the_day::BackInTheDay;
use super::boring_words::BoringWords;
use super::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::chock_full::ChockFull;
use super::compound_nouns::CompoundNouns;
use super::confident::Confident;
use super::correct_number_suffix::CorrectNumberSuffix;
use super::despite_of::DespiteOf;
use super::dot_initialisms::DotInitialisms;
use super::ellipsis_length::EllipsisLength;
use super::expand_time_shorthands::ExpandTimeShorthands;
use super::hedging::Hedging;
use super::hereby::Hereby;
use super::hop_hope::HopHope;
use super::hyphenate_number_day::HyphenateNumberDay;
use super::left_right_hand::LeftRightHand;
use super::lets_confusion::LetsConfusion;
use super::likewise::Likewise;
use super::linking_verbs::LinkingVerbs;
use super::long_sentences::LongSentences;
use super::matcher::Matcher;
use super::merge_words::MergeWords;
use super::modal_of::ModalOf;
use super::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::nobody::Nobody;
use super::number_suffix_capitalization::NumberSuffixCapitalization;
use super::out_of_date::OutOfDate;
use super::oxymorons::Oxymorons;
use super::pique_interest::PiqueInterest;
use super::plural_conjugate::PluralConjugate;
use super::possessive_your::PossessiveYour;
use super::pronoun_contraction::PronounContraction;
use super::proper_noun_capitalization_linters;
use super::repeated_words::RepeatedWords;
use super::sentence_capitalization::SentenceCapitalization;
use super::somewhat_something::SomewhatSomething;
use super::spaces::Spaces;
use super::spell_check::SpellCheck;
use super::spelled_numbers::SpelledNumbers;
use super::terminating_conjunctions::TerminatingConjunctions;
use super::that_which::ThatWhich;
use super::then_than::ThenThan;
use super::unclosed_quotes::UnclosedQuotes;
use super::use_genitive::UseGenitive;
use super::was_aloud::WasAloud;
use super::whereas::Whereas;
use super::wordpress_dotcom::WordPressDotcom;
use super::wrong_quotes::WrongQuotes;
use super::{CurrencyPlacement, Linter, NoOxfordComma, OxfordComma};
use crate::Document;
use crate::linting::{closed_compounds, phrase_corrections};
use crate::{Dictionary, MutableDictionary};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(transparent)]
pub struct LintGroupConfig {
    inner: HashMap<String, Option<bool>>,
}

#[cached]
fn curated_config() -> LintGroupConfig {
    // Dictionary does not matter, we're just after the config.
    let group = LintGroup::new_curated(MutableDictionary::new().into());
    group.config
}

impl LintGroupConfig {
    pub fn set_rule_enabled(&mut self, key: impl ToString, val: bool) {
        self.inner.insert(key.to_string(), Some(val));
    }

    /// Remove any configuration attached to a rule.
    /// This allows it to assume its default (curated) state.
    pub fn unset_rule_enabled(&mut self, key: impl AsRef<str>) {
        self.inner.remove_entry(key.as_ref());
    }

    pub fn set_rule_enabled_if_unset(&mut self, key: impl AsRef<str>, val: bool) {
        if self.inner.get(key.as_ref()).is_none() {
            self.set_rule_enabled(key.as_ref().to_string(), val);
        }
    }

    pub fn is_rule_enabled(&self, key: &str) -> bool {
        self.inner.get(key).cloned().flatten().unwrap_or(false)
    }

    /// Clear all config options.
    /// This will reset them all to disabled.
    pub fn clear(&mut self) {
        for val in self.inner.values_mut() {
            *val = None
        }
    }

    /// Merge the contents of another [`LintGroupConfig`] into this one.
    /// The other config will be left empty after this operation.
    ///
    /// Conflicting keys will be overridden by the value in the other group.
    pub fn merge_from(&mut self, other: &mut LintGroupConfig) {
        for (key, val) in other.inner.drain() {
            if val.is_none() {
                continue;
            }

            self.inner.insert(key, val);
        }
    }

    /// Fill the group with the values for the curated lint group.
    pub fn fill_with_curated(&mut self) {
        let mut temp = Self::new_curated();
        mem::swap(self, &mut temp);
        self.merge_from(&mut temp);
    }

    pub fn new_curated() -> Self {
        curated_config()
    }
}

#[derive(Default)]
pub struct LintGroup {
    pub config: LintGroupConfig,
    /// We use a binary map here so the ordering is stable.
    inner: BTreeMap<String, Box<dyn Linter>>,
}

impl LintGroup {
    pub fn empty() -> Self {
        Self {
            config: LintGroupConfig::default(),
            inner: BTreeMap::new(),
        }
    }

    /// Add a [`Linter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add(&mut self, name: impl AsRef<str>, linter: Box<dyn Linter>) -> bool {
        if self.inner.contains_key(name.as_ref()) {
            false
        } else {
            self.inner.insert(name.as_ref().to_string(), linter);
            true
        }
    }

    /// Merge the contents of another [`LintGroup`] into this one.
    /// The other lint group will be left empty after this operation.
    pub fn merge_from(&mut self, other: &mut LintGroup) {
        self.config.merge_from(&mut other.config);

        let other_map = std::mem::take(&mut other.inner);

        self.inner.extend(other_map);
    }

    /// Set all contained rules to a specific value.
    /// Passing `None` will unset that rule, allowing it to assume its default state.
    pub fn set_all_rules_to(&mut self, enabled: Option<bool>) {
        for key in self.inner.keys() {
            match enabled {
                Some(v) => self.config.set_rule_enabled(key, v),
                None => self.config.unset_rule_enabled(key),
            }
        }
    }

    pub fn all_descriptions(&self) -> HashMap<&str, &str> {
        self.inner
            .iter()
            .map(|(key, value)| (key.as_str(), value.description()))
            .collect()
    }

    /// Swap out [`Self::config`] with another [`LintGroupConfig`].
    pub fn with_lint_config(mut self, config: LintGroupConfig) -> Self {
        self.config = config;
        self
    }

    pub fn new_curated(dictionary: Arc<impl Dictionary + 'static>) -> Self {
        let mut out = Self::empty();

        macro_rules! insert_struct_rule {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), Box::new($rule::default()));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        out.merge_from(&mut phrase_corrections::lint_group());
        out.merge_from(&mut proper_noun_capitalization_linters::lint_group(
            dictionary.clone(),
        ));
        out.merge_from(&mut closed_compounds::lint_group());

        // Add all the more complex rules to the group.
        insert_struct_rule!(BackInTheDay, true);
        insert_struct_rule!(WordPressDotcom, true);
        insert_struct_rule!(OutOfDate, true);
        insert_struct_rule!(ThenThan, true);
        insert_struct_rule!(PiqueInterest, true);
        insert_struct_rule!(WasAloud, true);
        insert_struct_rule!(HyphenateNumberDay, true);
        insert_struct_rule!(LeftRightHand, true);
        insert_struct_rule!(HopHope, true);
        insert_struct_rule!(Hereby, true);
        insert_struct_rule!(Likewise, true);
        insert_struct_rule!(CompoundNouns, true);
        insert_struct_rule!(Nobody, true);
        insert_struct_rule!(Whereas, true);
        insert_struct_rule!(PossessiveYour, true);
        insert_struct_rule!(SpelledNumbers, false);
        insert_struct_rule!(AnA, true);
        insert_struct_rule!(SentenceCapitalization, true);
        insert_struct_rule!(UnclosedQuotes, true);
        insert_struct_rule!(WrongQuotes, false);
        insert_struct_rule!(LongSentences, true);
        insert_struct_rule!(RepeatedWords, true);
        insert_struct_rule!(Spaces, true);
        insert_struct_rule!(Matcher, true);
        insert_struct_rule!(CorrectNumberSuffix, true);
        insert_struct_rule!(NumberSuffixCapitalization, true);
        insert_struct_rule!(MultipleSequentialPronouns, true);
        insert_struct_rule!(LinkingVerbs, false);
        insert_struct_rule!(AvoidCurses, true);
        insert_struct_rule!(TerminatingConjunctions, true);
        insert_struct_rule!(EllipsisLength, true);
        insert_struct_rule!(DotInitialisms, true);
        insert_struct_rule!(BoringWords, false);
        insert_struct_rule!(UseGenitive, false);
        insert_struct_rule!(ThatWhich, true);
        insert_struct_rule!(CapitalizePersonalPronouns, true);
        insert_struct_rule!(MergeWords, true);
        insert_struct_rule!(PluralConjugate, false);
        insert_struct_rule!(OxfordComma, true);
        insert_struct_rule!(NoOxfordComma, false);
        insert_struct_rule!(PronounContraction, true);
        insert_struct_rule!(CurrencyPlacement, true);
        insert_struct_rule!(SomewhatSomething, true);
        insert_struct_rule!(LetsConfusion, true);
        insert_struct_rule!(DespiteOf, true);
        insert_struct_rule!(ChockFull, true);
        insert_struct_rule!(Confident, true);
        insert_struct_rule!(Oxymorons, true);
        insert_struct_rule!(Hedging, true);
        insert_struct_rule!(ExpandTimeShorthands, true);
        insert_struct_rule!(ModalOf, true);

        out.add("SpellCheck", Box::new(SpellCheck::new(dictionary)));
        out.config.set_rule_enabled("SpellCheck", true);

        out
    }

    /// Create a new curated group with all config values cleared out.
    pub fn new_curated_empty_config(dictionary: Arc<impl Dictionary + 'static>) -> Self {
        let mut group = Self::new_curated(dictionary);
        group.config.clear();
        group
    }
}

impl Linter for LintGroup {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut results = Vec::new();

        for (key, linter) in &mut self.inner {
            if self.config.is_rule_enabled(key) {
                results.extend(linter.lint(document));
            }
        }

        results
    }

    fn description(&self) -> &str {
        "A collection of linters that can be run as one."
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{Document, FstDictionary, MutableDictionary, linting::Linter};

    use super::LintGroup;

    #[test]
    fn can_get_all_descriptions() {
        let group = LintGroup::new_curated(Arc::new(MutableDictionary::default()));
        group.all_descriptions();
    }

    #[test]
    fn lint_descriptions_are_clean() {
        let mut group = LintGroup::new_curated(FstDictionary::curated());
        let pairs: Vec<_> = group
            .all_descriptions()
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        for (key, value) in pairs {
            let doc = Document::new_markdown_default_curated(&value);
            eprintln!("{key}: {value}");

            if !group.lint(&doc).is_empty() {
                dbg!(&group.lint(&doc));
                panic!();
            }
        }
    }
}
