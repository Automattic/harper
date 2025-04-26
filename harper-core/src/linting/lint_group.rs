use std::collections::BTreeMap;
use std::hash::Hash;
use std::hash::{BuildHasher, Hasher};
use std::mem;
use std::num::NonZero;
use std::sync::Arc;

use cached::proc_macro::cached;
use foldhash::quality::RandomState;
use hashbrown::HashMap;
use lru::LruCache;
use serde::{Deserialize, Serialize};

use super::adjective_of_a::AdjectiveOfA;
use super::an_a::AnA;
use super::avoid_curses::AvoidCurses;
use super::back_in_the_day::BackInTheDay;
use super::boring_words::BoringWords;
use super::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::chock_full::ChockFull;
use super::comma_fixes::CommaFixes;
use super::compound_nouns::CompoundNouns;
use super::confident::Confident;
use super::correct_number_suffix::CorrectNumberSuffix;
use super::despite_of::DespiteOf;
use super::dot_initialisms::DotInitialisms;
use super::ellipsis_length::EllipsisLength;
use super::expand_time_shorthands::ExpandTimeShorthands;
use super::first_aid_kit::FirstAidKit;
use super::for_noun::ForNoun;
use super::hedging::Hedging;
use super::hereby::Hereby;
use super::hop_hope::HopHope;
use super::hyphenate_number_day::HyphenateNumberDay;
use super::inflected_verb_after_to::InflectedVerbAfterTo;
use super::left_right_hand::LeftRightHand;
use super::lets_confusion::LetsConfusion;
use super::likewise::Likewise;
use super::linking_verbs::LinkingVerbs;
use super::long_sentences::LongSentences;
use super::merge_words::MergeWords;
use super::modal_of::ModalOf;
use super::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::nobody::Nobody;
use super::number_suffix_capitalization::NumberSuffixCapitalization;
use super::of_course::OfCourse;
use super::out_of_date::OutOfDate;
use super::oxymorons::Oxymorons;
use super::pattern_linter::run_on_chunk;
use super::pique_interest::PiqueInterest;
use super::possessive_your::PossessiveYour;
use super::pronoun_contraction::PronounContraction;
use super::pronoun_knew::PronounKnew;
use super::proper_noun_capitalization_linters;
use super::repeated_words::RepeatedWords;
use super::sentence_capitalization::SentenceCapitalization;
use super::somewhat_something::SomewhatSomething;
use super::spaces::Spaces;
use super::spell_check::SpellCheck;
use super::spelled_numbers::SpelledNumbers;
use super::that_which::ThatWhich;
use super::the_how_why::TheHowWhy;
use super::the_my::TheMy;
use super::then_than::ThenThan;
use super::unclosed_quotes::UnclosedQuotes;
use super::use_genitive::UseGenitive;
use super::was_aloud::WasAloud;
use super::whereas::Whereas;
use super::widely_accepted::WidelyAccepted;
use super::wordpress_dotcom::WordPressDotcom;
use super::{CurrencyPlacement, Linter, NoOxfordComma, OxfordComma};
use super::{Lint, PatternLinter};
use crate::linting::dashes::Dashes;
use crate::linting::{closed_compounds, phrase_corrections};
use crate::{CharString, Dialect, Document, TokenStringExt};
use crate::{Dictionary, MutableDictionary};

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct LintGroupConfig {
    /// A `BTreeMap` so that the config has a stable ordering when written to disk.
    inner: BTreeMap<String, Option<bool>>,
}

#[cached]
fn curated_config() -> LintGroupConfig {
    // The Dictionary and Dialect do not matter, we're just after the config.
    let group = LintGroup::new_curated(MutableDictionary::new().into(), Dialect::American);
    group.config
}

impl LintGroupConfig {
    pub fn set_rule_enabled(&mut self, key: impl ToString, val: bool) {
        self.inner.insert(key.to_string(), Some(val));
    }

    /// Remove any configuration attached to a rule.
    /// This allows it to assume its default (curated) state.
    pub fn unset_rule_enabled(&mut self, key: impl AsRef<str>) {
        self.inner.remove(key.as_ref());
    }

    pub fn set_rule_enabled_if_unset(&mut self, key: impl AsRef<str>, val: bool) {
        if !self.inner.contains_key(key.as_ref()) {
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
        for (key, val) in other.inner.iter() {
            if val.is_none() {
                continue;
            }

            self.inner.insert(key.to_string(), *val);
        }

        other.clear();
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

impl Hash for LintGroupConfig {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for (key, value) in &self.inner {
            hasher.write(key.as_bytes());
            if let Some(value) = value {
                hasher.write_u8(1);
                hasher.write_u8(*value as u8);
            } else {
                // Do it twice so we fill the same number of bytes as the other branch.
                hasher.write_u8(0);
                hasher.write_u8(0);
            }
        }
    }
}

pub struct LintGroup {
    pub config: LintGroupConfig,
    /// We use a binary map here so the ordering is stable.
    linters: BTreeMap<String, Box<dyn Linter>>,
    /// We use a binary map here so the ordering is stable.
    pattern_linters: BTreeMap<String, Box<dyn PatternLinter>>,
    /// Since [`PatternLinter`]s operate on a chunk-basis, we can store a
    /// mapping of `Chunk -> Lint` and only re-run the pattern linters
    /// when a chunk changes.
    ///
    /// Since the pattern linter results also depend on the config, we hash it and pass it as part
    /// of the key.
    chunk_pattern_cache: LruCache<(CharString, u64), Vec<Lint>>,
    hasher_builder: RandomState,
}

impl LintGroup {
    pub fn empty() -> Self {
        Self {
            config: LintGroupConfig::default(),
            linters: BTreeMap::new(),
            pattern_linters: BTreeMap::new(),
            chunk_pattern_cache: LruCache::new(NonZero::new(10000).unwrap()),
            hasher_builder: RandomState::default(),
        }
    }

    /// Check if the group already contains a linter with a given name.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.linters.contains_key(name.as_ref()) || self.pattern_linters.contains_key(name.as_ref())
    }

    /// Add a [`Linter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add(&mut self, name: impl AsRef<str>, linter: Box<dyn Linter>) -> bool {
        if self.contains_key(&name) {
            false
        } else {
            self.linters.insert(name.as_ref().to_string(), linter);
            true
        }
    }

    /// Add a [`PatternLinter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    ///
    /// This function is not significantly different from [`Self::add`], but allows us to take
    /// advantage of some properties of [`PatternLinter`]s for cache optimization.
    pub fn add_pattern_linter(
        &mut self,
        name: impl AsRef<str>,
        linter: Box<dyn PatternLinter>,
    ) -> bool {
        if self.contains_key(&name) {
            false
        } else {
            self.pattern_linters
                .insert(name.as_ref().to_string(), linter);
            true
        }
    }

    /// Merge the contents of another [`LintGroup`] into this one.
    /// The other lint group will be left empty after this operation.
    pub fn merge_from(&mut self, other: &mut LintGroup) {
        self.config.merge_from(&mut other.config);

        let other_linters = std::mem::take(&mut other.linters);
        self.linters.extend(other_linters);

        let other_pattern_linters = std::mem::take(&mut other.pattern_linters);
        self.pattern_linters.extend(other_pattern_linters);
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.linters
            .keys()
            .chain(self.pattern_linters.keys())
            .map(|v| v.as_str())
    }

    /// Set all contained rules to a specific value.
    /// Passing `None` will unset that rule, allowing it to assume its default state.
    pub fn set_all_rules_to(&mut self, enabled: Option<bool>) {
        let keys = self.iter_keys().map(|v| v.to_string()).collect::<Vec<_>>();

        for key in keys {
            match enabled {
                Some(v) => self.config.set_rule_enabled(key, v),
                None => self.config.unset_rule_enabled(key),
            }
        }
    }

    pub fn all_descriptions(&self) -> HashMap<&str, &str> {
        self.linters
            .iter()
            .map(|(key, value)| (key.as_str(), value.description()))
            .chain(
                self.pattern_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), PatternLinter::description(value))),
            )
            .collect()
    }

    /// Swap out [`Self::config`] with another [`LintGroupConfig`].
    pub fn with_lint_config(mut self, config: LintGroupConfig) -> Self {
        self.config = config;
        self
    }

    pub fn new_curated(dictionary: Arc<impl Dictionary + 'static>, dialect: Dialect) -> Self {
        let mut out = Self::empty();

        macro_rules! insert_struct_rule {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), Box::new($rule::default()));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        macro_rules! insert_pattern_rule {
            ($rule:ident, $default_config:expr) => {
                out.add_pattern_linter(stringify!($rule), Box::new($rule::default()));
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
        insert_struct_rule!(AdjectiveOfA, true);
        insert_pattern_rule!(BackInTheDay, true);
        insert_pattern_rule!(Dashes, true);
        insert_struct_rule!(WordPressDotcom, true);
        insert_pattern_rule!(OutOfDate, true);
        insert_struct_rule!(TheMy, true);
        insert_pattern_rule!(ThenThan, true);
        insert_pattern_rule!(PiqueInterest, true);
        insert_pattern_rule!(WasAloud, true);
        insert_pattern_rule!(HyphenateNumberDay, true);
        insert_pattern_rule!(LeftRightHand, true);
        insert_struct_rule!(HopHope, true);
        insert_pattern_rule!(Hereby, true);
        insert_pattern_rule!(Likewise, true);
        insert_struct_rule!(CompoundNouns, true);
        insert_pattern_rule!(Nobody, true);
        insert_pattern_rule!(Whereas, true);
        insert_pattern_rule!(PossessiveYour, true);
        insert_struct_rule!(SpelledNumbers, false);
        insert_struct_rule!(AnA, true);
        insert_struct_rule!(UnclosedQuotes, true);
        insert_struct_rule!(LongSentences, true);
        insert_struct_rule!(RepeatedWords, true);
        insert_struct_rule!(Spaces, true);
        insert_struct_rule!(CorrectNumberSuffix, true);
        insert_struct_rule!(NumberSuffixCapitalization, true);
        insert_pattern_rule!(MultipleSequentialPronouns, true);
        insert_struct_rule!(LinkingVerbs, false);
        insert_struct_rule!(AvoidCurses, true);
        insert_struct_rule!(EllipsisLength, true);
        insert_struct_rule!(CommaFixes, true);
        insert_pattern_rule!(DotInitialisms, true);
        insert_pattern_rule!(BoringWords, false);
        insert_pattern_rule!(UseGenitive, false);
        insert_pattern_rule!(ThatWhich, true);
        insert_struct_rule!(CapitalizePersonalPronouns, true);
        insert_struct_rule!(MergeWords, true);
        insert_struct_rule!(OxfordComma, true);
        insert_struct_rule!(NoOxfordComma, false);
        insert_struct_rule!(PronounContraction, true);
        insert_struct_rule!(CurrencyPlacement, true);
        insert_pattern_rule!(SomewhatSomething, true);
        insert_struct_rule!(LetsConfusion, true);
        insert_pattern_rule!(DespiteOf, true);
        insert_pattern_rule!(ChockFull, true);
        insert_struct_rule!(OfCourse, true);
        insert_struct_rule!(FirstAidKit, true);
        insert_struct_rule!(PronounKnew, true);
        insert_struct_rule!(TheHowWhy, true);
        insert_struct_rule!(WidelyAccepted, true);
        insert_pattern_rule!(Confident, true);
        insert_pattern_rule!(Oxymorons, true);
        insert_pattern_rule!(Hedging, true);
        insert_pattern_rule!(ExpandTimeShorthands, true);
        insert_pattern_rule!(ModalOf, true);
        insert_pattern_rule!(ForNoun, true);

        out.add(
            "SpellCheck",
            Box::new(SpellCheck::new(dictionary.clone(), dialect)),
        );
        out.config.set_rule_enabled("SpellCheck", true);

        out.add(
            "InflectedVerbAfterTo",
            Box::new(InflectedVerbAfterTo::new(dictionary.clone(), dialect)),
        );
        out.config.set_rule_enabled("InflectedVerbAfterTo", true);

        out.add(
            "SentenceCapitalization",
            Box::new(SentenceCapitalization::new(dictionary.clone(), dialect)),
        );
        out.config.set_rule_enabled("SentenceCapitalization", true);

        out
    }

    /// Create a new curated group with all config values cleared out.
    pub fn new_curated_empty_config(
        dictionary: Arc<impl Dictionary + 'static>,
        dialect: Dialect,
    ) -> Self {
        let mut group = Self::new_curated(dictionary, dialect);
        group.config.clear();
        group
    }
}

impl Default for LintGroup {
    fn default() -> Self {
        Self::empty()
    }
}

impl Linter for LintGroup {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut results = Vec::new();

        // Normal linters
        for (key, linter) in &mut self.linters {
            if self.config.is_rule_enabled(key) {
                results.extend(linter.lint(document));
            }
        }

        // Pattern linters
        for chunk in document.iter_chunks() {
            let Some(chunk_span) = chunk.span() else {
                continue;
            };

            let chunk_chars = document.get_span_content(&chunk_span);
            let config_hash = self.hasher_builder.hash_one(&self.config);
            let key = (chunk_chars.into(), config_hash);

            let mut chunk_results = if let Some(hit) = self.chunk_pattern_cache.get(&key) {
                hit.clone()
            } else {
                let mut pattern_lints = Vec::new();

                for (key, linter) in &mut self.pattern_linters {
                    if self.config.is_rule_enabled(key) {
                        pattern_lints.extend(run_on_chunk(linter, chunk, document.get_source()));
                    }
                }

                // Make the spans relative to the chunk start
                for lint in &mut pattern_lints {
                    lint.span.pull_by(chunk_span.start);
                }

                self.chunk_pattern_cache.put(key, pattern_lints.clone());
                pattern_lints
            };

            // Bring the spans back into document-space
            for lint in &mut chunk_results {
                lint.span.push_by(chunk_span.start);
            }

            results.append(&mut chunk_results);
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

    use crate::{Dialect, Document, FstDictionary, MutableDictionary, linting::Linter};

    use super::LintGroup;

    #[test]
    fn can_get_all_descriptions() {
        let group =
            LintGroup::new_curated(Arc::new(MutableDictionary::default()), Dialect::American);
        group.all_descriptions();
    }

    #[test]
    fn lint_descriptions_are_clean() {
        let mut group = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);
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
