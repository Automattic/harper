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
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::a_part::APart;
use super::adjective_of_a::AdjectiveOfA;
use super::am_in_the_morning::AmInTheMorning;
use super::amounts_for::AmountsFor;
use super::an_a::AnA;
use super::another_thing_coming::AnotherThingComing;
use super::another_think_coming::AnotherThinkComing;
use super::ask_no_preposition::AskNoPreposition;
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
use super::discourse_markers::DiscourseMarkers;
use super::dot_initialisms::DotInitialisms;
use super::ellipsis_length::EllipsisLength;
use super::else_possessive::ElsePossessive;
use super::everyday::Everyday;
use super::expand_time_shorthands::ExpandTimeShorthands;
use super::expr_linter::run_on_chunk;
use super::few_units_of_time_ago::FewUnitsOfTimeAgo;
use super::first_aid_kit::FirstAidKit;
use super::for_noun::ForNoun;
use super::have_pronoun::HavePronoun;
use super::hedging::Hedging;
use super::hereby::Hereby;
use super::hop_hope::HopHope;
use super::how_to::HowTo;
use super::hyphenate_number_day::HyphenateNumberDay;
use super::in_on_the_cards::InOnTheCards;
use super::inflected_verb_after_to::InflectedVerbAfterTo;
use super::its_contraction::ItsContraction;
use super::left_right_hand::LeftRightHand;
use super::less_worse::LessWorse;
use super::lets_confusion::LetsConfusion;
use super::likewise::Likewise;
use super::long_sentences::LongSentences;
use super::merge_words::MergeWords;
use super::modal_of::ModalOf;
use super::most_number::MostNumber;
use super::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::nail_on_the_head::NailOnTheHead;
use super::no_match_for::NoMatchFor;
use super::nobody::Nobody;
use super::nominal_wants::NominalWants;
use super::noun_instead_of_verb::NounInsteadOfVerb;
use super::number_suffix_capitalization::NumberSuffixCapitalization;
use super::of_course::OfCourse;
use super::one_and_the_same::OneAndTheSame;
use super::open_the_light::OpenTheLight;
use super::out_of_date::OutOfDate;
use super::oxymorons::Oxymorons;
use super::phrasal_verb_as_compound_noun::PhrasalVerbAsCompoundNoun;
use super::pique_interest::PiqueInterest;
use super::possessive_noun::PossessiveNoun;
use super::possessive_your::PossessiveYour;
use super::pronoun_contraction::PronounContraction;
use super::pronoun_inflection_be::PronounInflectionBe;
use super::pronoun_knew::PronounKnew;
use super::proper_noun_capitalization_linters;
use super::redundant_additive_adverbs::RedundantAdditiveAdverbs;
use super::regionalisms::Regionalisms;
use super::repeated_words::RepeatedWords;
use super::save_to_safe::SaveToSafe;
use super::sentence_capitalization::SentenceCapitalization;
use super::shoot_oneself_in_the_foot::ShootOneselfInTheFoot;
use super::since_duration::SinceDuration;
use super::somewhat_something::SomewhatSomething;
use super::spaces::Spaces;
use super::spell_check::SpellCheck;
use super::spelled_numbers::SpelledNumbers;
use super::that_which::ThatWhich;
use super::the_how_why::TheHowWhy;
use super::the_my::TheMy;
use super::then_than::ThenThan;
use super::thing_think::ThingThink;
use super::throw_rubbish::ThrowRubbish;
use super::touristic::Touristic;
use super::unclosed_quotes::UnclosedQuotes;
use super::use_genitive::UseGenitive;
use super::was_aloud::WasAloud;
use super::way_too_adjective::WayTooAdjective;
use super::whereas::Whereas;
use super::widely_accepted::WidelyAccepted;
use super::win_prize::WinPrize;
use super::wordpress_dotcom::WordPressDotcom;
use super::{CurrencyPlacement, HtmlDescriptionLinter, Linter, NoOxfordComma, OxfordComma};
use super::{ExprLinter, Lint};
use crate::linting::dashes::Dashes;
use crate::linting::open_compounds::OpenCompounds;
use crate::linting::{closed_compounds, initialisms, phrase_corrections};
use crate::{CharString, Dialect, Document, TokenStringExt};
use crate::{Dictionary, MutableDictionary};

fn ser_ordered<S>(map: &HashMap<String, Option<bool>>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    ordered.serialize(ser)
}

fn de_hashbrown<'de, D>(de: D) -> Result<HashMap<String, Option<bool>>, D::Error>
where
    D: Deserializer<'de>,
{
    let ordered: BTreeMap<String, Option<bool>> = BTreeMap::deserialize(de)?;
    Ok(ordered.into_iter().collect())
}

/// The configuration for a [`LintGroup`].
/// Each child linter can be enabled, disabled, or set to a curated value.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct LintGroupConfig {
    /// We do this shenanigans with the [`BTreeMap`] to keep the serialized format consistent.
    #[serde(serialize_with = "ser_ordered", deserialize_with = "de_hashbrown")]
    inner: HashMap<String, Option<bool>>,
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
    /// This will reset them all to disable them.
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

/// A struct for collecting the output of a number of individual [Linter]s.
/// Each child can be toggled via the public, mutable [Self::config] object.
pub struct LintGroup {
    pub config: LintGroupConfig,
    /// We use a binary map here so the ordering is stable.
    linters: BTreeMap<String, Box<dyn Linter>>,
    /// We use a binary map here so the ordering is stable.
    expr_linters: BTreeMap<String, Box<dyn ExprLinter>>,
    /// Since [`ExprLinter`]s operate on a chunk-basis, we can store a
    /// mapping of `Chunk -> Lint` and only re-run the pattern linters
    /// when a chunk changes.
    ///
    /// Since the pattern linter results also depend on the config, we hash it and pass it as part
    /// of the key.
    chunk_expr_cache: LruCache<(CharString, u64), Vec<Lint>>,
    hasher_builder: RandomState,
}

impl LintGroup {
    pub fn empty() -> Self {
        Self {
            config: LintGroupConfig::default(),
            linters: BTreeMap::new(),
            expr_linters: BTreeMap::new(),
            chunk_expr_cache: LruCache::new(NonZero::new(10000).unwrap()),
            hasher_builder: RandomState::default(),
        }
    }

    /// Check if the group already contains a linter with a given name.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.linters.contains_key(name.as_ref()) || self.expr_linters.contains_key(name.as_ref())
    }

    /// Add a [`Linter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add(&mut self, name: impl AsRef<str>, linter: impl Linter + 'static) -> bool {
        if self.contains_key(&name) {
            false
        } else {
            self.linters
                .insert(name.as_ref().to_string(), Box::new(linter));
            true
        }
    }

    /// Add a [`ExprLinter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    ///
    /// This function is not significantly different from [`Self::add`], but allows us to take
    /// advantage of some properties of [`ExprLinter`]s for cache optimization.
    pub fn add_expr_linter(
        &mut self,
        name: impl AsRef<str>,
        linter: impl ExprLinter + 'static,
    ) -> bool {
        if self.contains_key(&name) {
            false
        } else {
            self.expr_linters
                .insert(name.as_ref().to_string(), Box::new(linter));
            true
        }
    }

    /// Merge the contents of another [`LintGroup`] into this one.
    /// The other lint group will be left empty after this operation.
    pub fn merge_from(&mut self, other: &mut LintGroup) {
        self.config.merge_from(&mut other.config);

        let other_linters = std::mem::take(&mut other.linters);
        self.linters.extend(other_linters);

        let other_pattern_linters = std::mem::take(&mut other.expr_linters);
        self.expr_linters.extend(other_pattern_linters);
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.linters
            .keys()
            .chain(self.expr_linters.keys())
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

    /// Get map from each contained linter's name to its associated description.
    pub fn all_descriptions(&self) -> HashMap<&str, &str> {
        self.linters
            .iter()
            .map(|(key, value)| (key.as_str(), value.description()))
            .chain(
                self.expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), ExprLinter::description(value))),
            )
            .collect()
    }

    /// Get map from each contained linter's name to its associated description, rendered to HTML.
    pub fn all_descriptions_html(&self) -> HashMap<&str, String> {
        self.linters
            .iter()
            .map(|(key, value)| (key.as_str(), value.description_html()))
            .chain(
                self.expr_linters
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.description_html())),
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
                out.add(stringify!($rule), $rule::default());
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        macro_rules! insert_expr_rule {
            ($rule:ident, $default_config:expr) => {
                out.add_expr_linter(stringify!($rule), $rule::default());
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        out.merge_from(&mut phrase_corrections::lint_group());
        out.merge_from(&mut proper_noun_capitalization_linters::lint_group(
            dictionary.clone(),
        ));
        out.merge_from(&mut closed_compounds::lint_group());
        out.merge_from(&mut initialisms::lint_group());

        // Add all the more complex rules to the group.
        insert_expr_rule!(APart, true);
        insert_struct_rule!(AdjectiveOfA, true);
        insert_struct_rule!(AmInTheMorning, true);
        insert_expr_rule!(AmountsFor, true);
        insert_struct_rule!(AnA, true);
        insert_expr_rule!(AnotherThingComing, true);
        insert_expr_rule!(AnotherThinkComing, false);
        insert_expr_rule!(AskNoPreposition, true);
        insert_struct_rule!(AvoidCurses, true);
        insert_expr_rule!(BackInTheDay, true);
        insert_expr_rule!(BoringWords, false);
        insert_struct_rule!(CapitalizePersonalPronouns, true);
        insert_expr_rule!(ChockFull, true);
        insert_struct_rule!(DiscourseMarkers, true);
        insert_expr_rule!(WayTooAdjective, true);
        insert_expr_rule!(HavePronoun, true);
        insert_expr_rule!(PronounInflectionBe, true);
        insert_struct_rule!(CommaFixes, true);
        insert_struct_rule!(CompoundNouns, true);
        insert_expr_rule!(Confident, true);
        insert_struct_rule!(CorrectNumberSuffix, true);
        insert_struct_rule!(CurrencyPlacement, true);
        insert_expr_rule!(Dashes, true);
        insert_expr_rule!(DespiteOf, true);
        insert_expr_rule!(DotInitialisms, true);
        insert_struct_rule!(EllipsisLength, true);
        insert_struct_rule!(ElsePossessive, true);
        insert_struct_rule!(Everyday, true);
        insert_expr_rule!(ExpandTimeShorthands, true);
        insert_expr_rule!(FewUnitsOfTimeAgo, true);
        insert_struct_rule!(FirstAidKit, true);
        insert_struct_rule!(ForNoun, true);
        insert_expr_rule!(Hedging, true);
        insert_expr_rule!(Hereby, true);
        insert_expr_rule!(OpenCompounds, true);
        insert_struct_rule!(HopHope, true);
        insert_struct_rule!(HowTo, true);
        insert_expr_rule!(HyphenateNumberDay, true);
        insert_struct_rule!(ItsContraction, true);
        insert_expr_rule!(LeftRightHand, true);
        insert_expr_rule!(LessWorse, true);
        insert_struct_rule!(LetsConfusion, true);
        insert_expr_rule!(Likewise, true);
        insert_struct_rule!(LongSentences, true);
        insert_struct_rule!(MergeWords, true);
        insert_expr_rule!(ModalOf, true);
        insert_expr_rule!(MostNumber, true);
        insert_expr_rule!(MultipleSequentialPronouns, true);
        insert_expr_rule!(NoMatchFor, true);
        insert_struct_rule!(NailOnTheHead, true);
        insert_struct_rule!(NominalWants, true);
        insert_struct_rule!(NoOxfordComma, false);
        insert_expr_rule!(Nobody, true);
        insert_expr_rule!(NounInsteadOfVerb, true);
        insert_struct_rule!(NumberSuffixCapitalization, true);
        insert_struct_rule!(OfCourse, true);
        insert_expr_rule!(OneAndTheSame, true);
        insert_expr_rule!(OpenTheLight, true);
        insert_expr_rule!(OutOfDate, true);
        insert_struct_rule!(OxfordComma, true);
        insert_expr_rule!(Oxymorons, true);
        insert_struct_rule!(PhrasalVerbAsCompoundNoun, true);
        insert_expr_rule!(PiqueInterest, true);
        insert_expr_rule!(PossessiveYour, true);
        insert_struct_rule!(PronounContraction, true);
        insert_struct_rule!(PronounKnew, true);
        insert_expr_rule!(RedundantAdditiveAdverbs, true);
        insert_struct_rule!(RepeatedWords, true);
        insert_struct_rule!(SaveToSafe, true);
        insert_expr_rule!(SinceDuration, true);
        insert_expr_rule!(ShootOneselfInTheFoot, true);
        insert_expr_rule!(SomewhatSomething, true);
        insert_struct_rule!(Spaces, true);
        insert_struct_rule!(SpelledNumbers, false);
        insert_expr_rule!(ThatWhich, true);
        insert_expr_rule!(TheHowWhy, true);
        insert_struct_rule!(TheMy, true);
        insert_expr_rule!(ThenThan, true);
        insert_expr_rule!(ThingThink, true);
        insert_struct_rule!(ThrowRubbish, true);
        insert_expr_rule!(Touristic, true);
        insert_struct_rule!(UnclosedQuotes, true);
        insert_expr_rule!(UseGenitive, false);
        insert_expr_rule!(WasAloud, true);
        insert_expr_rule!(Whereas, true);
        insert_expr_rule!(WidelyAccepted, true);
        insert_struct_rule!(WidelyAccepted, true);
        insert_expr_rule!(WinPrize, true);
        insert_struct_rule!(WordPressDotcom, true);

        out.add("SpellCheck", SpellCheck::new(dictionary.clone(), dialect));
        out.config.set_rule_enabled("SpellCheck", true);

        out.add(
            "InflectedVerbAfterTo",
            InflectedVerbAfterTo::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("InflectedVerbAfterTo", true);

        out.add("InOnTheCards", InOnTheCards::new(dialect));
        out.config.set_rule_enabled("InOnTheCards", true);

        out.add(
            "SentenceCapitalization",
            SentenceCapitalization::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("SentenceCapitalization", true);

        out.add("PossessiveNoun", PossessiveNoun::new(dictionary.clone()));
        out.config.set_rule_enabled("PossessiveNoun", true);

        out.add("Regionalisms", Regionalisms::new(dialect));
        out.config.set_rule_enabled("Regionalisms", true);

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

            let mut chunk_results = if let Some(hit) = self.chunk_expr_cache.get(&key) {
                hit.clone()
            } else {
                let mut pattern_lints = Vec::new();

                for (key, linter) in &mut self.expr_linters {
                    if self.config.is_rule_enabled(key) {
                        pattern_lints.extend(run_on_chunk(linter, chunk, document.get_source()));
                    }
                }

                // Make the spans relative to the chunk start
                for lint in &mut pattern_lints {
                    lint.span.pull_by(chunk_span.start);
                }

                self.chunk_expr_cache.put(key, pattern_lints.clone());
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
    fn can_get_all_descriptions_as_html() {
        let group =
            LintGroup::new_curated(Arc::new(MutableDictionary::default()), Dialect::American);
        group.all_descriptions_html();
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
