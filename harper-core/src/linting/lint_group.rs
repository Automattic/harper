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

use super::ExprLinter;
use super::Lint;

use super::english::a_part::APart;
use super::english::a_while::AWhile;
use super::english::addicting::Addicting;
use super::english::adjective_double_degree::AdjectiveDoubleDegree;
use super::english::adjective_of_a::AdjectiveOfA;
use super::english::after_later::AfterLater;
use super::english::all_hell_break_loose::AllHellBreakLoose;
use super::english::all_intents_and_purposes::AllIntentsAndPurposes;
use super::english::allow_to::AllowTo;
use super::english::am_in_the_morning::AmInTheMorning;
use super::english::amounts_for::AmountsFor;
use super::english::an_a::AnA;
use super::english::and_in::AndIn;
use super::english::and_the_like::AndTheLike;
use super::english::another_thing_coming::AnotherThingComing;
use super::english::another_think_coming::AnotherThinkComing;
use super::english::apart_from::ApartFrom;
use super::english::ask_no_preposition::AskNoPreposition;
use super::english::aspire_to::AspireTo;
use super::english::avoid_curses::AvoidCurses;
use super::english::back_in_the_day::BackInTheDay;
use super::english::be_allowed::BeAllowed;
use super::english::be_worried::BeWorried;
use super::english::behind_the_scenes::BehindTheScenes;
use super::english::best_of_all_time::BestOfAllTime;
use super::english::boring_words::BoringWords;
use super::english::bought::Bought;
use super::english::brand_brandish::BrandBrandish;
use super::english::by_accident::ByAccident;
use super::english::cant::Cant;
use super::english::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::english::cautionary_tale::CautionaryTale;
use super::english::change_tack::ChangeTack;
use super::english::chock_full::ChockFull;
use super::english::comma_fixes::CommaFixes;
use super::english::compound_nouns::CompoundNouns;
use super::english::compound_subject_i::CompoundSubjectI;
use super::english::confident::Confident;
use super::english::correct_number_suffix::CorrectNumberSuffix;
use super::english::criteria_phenomena::CriteriaPhenomena;
use super::english::cure_for::CureFor;
use super::english::currency_placement::CurrencyPlacement;
use super::english::damages::Damages;
use super::english::day_and_age::DayAndAge;
use super::english::despite_it_is::DespiteItIs;
use super::english::despite_of::DespiteOf;
use super::english::did_past::DidPast;
use super::english::didnt::Didnt;
use super::english::discourse_markers::DiscourseMarkers;
use super::english::disjoint_prefixes::DisjointPrefixes;
use super::english::do_mistake::DoMistake;
use super::english::dot_initialisms::DotInitialisms;
use super::english::double_click::DoubleClick;
use super::english::double_modal::DoubleModal;
use super::english::ellipsis_length::EllipsisLength;
use super::english::else_possessive::ElsePossessive;
use super::english::ever_every::EverEvery;
use super::english::everyday::Everyday;
use super::english::except_of::ExceptOf;
use super::english::expand_memory_shorthands::ExpandMemoryShorthands;
use super::english::expand_time_shorthands::ExpandTimeShorthands;
use super::english::expr_linter::run_on_chunk;
use super::english::far_be_it::FarBeIt;
use super::english::fascinated_by::FascinatedBy;
use super::english::fed_up_with::FedUpWith;
use super::english::feel_fell::FeelFell;
use super::english::few_units_of_time_ago::FewUnitsOfTimeAgo;
use super::english::filler_words::FillerWords;
use super::english::find_fine::FindFine;
use super::english::first_aid_kit::FirstAidKit;
use super::english::flesh_out_vs_full_fledged::FleshOutVsFullFledged;
use super::english::for_noun::ForNoun;
use super::english::free_predicate::FreePredicate;
use super::english::friend_of_me::FriendOfMe;
use super::english::go_so_far_as_to::GoSoFarAsTo;
use super::english::go_to_war::GoToWar;
use super::english::good_at::GoodAt;
use super::english::handful::Handful;
use super::english::have_pronoun::HavePronoun;
use super::english::have_take_a_look::HaveTakeALook;
use super::english::hedging::Hedging;
use super::english::hello_greeting::HelloGreeting;
use super::english::hereby::Hereby;
use super::english::hop_hope::HopHope;
use super::english::how_to::HowTo;
use super::english::hyphenate_number_day::HyphenateNumberDay;
use super::english::i_am_agreement::IAmAgreement;
use super::english::if_wouldve::IfWouldve;
use super::english::in_on_the_cards::InOnTheCards;
use super::english::inflected_verb_after_to::InflectedVerbAfterTo;
use super::english::interested_in::InterestedIn;
use super::english::it_looks_like_that::ItLooksLikeThat;
use super::english::its_contraction::ItsContraction;
use super::english::its_possessive::ItsPossessive;
use super::english::jealous_of::JealousOf;
use super::english::johns_hopkins::JohnsHopkins;
use super::english::lead_rise_to::LeadRiseTo;
use super::english::left_right_hand::LeftRightHand;
use super::english::less_worse::LessWorse;
use super::english::let_to_do::LetToDo;
use super::english::lets_confusion::LetsConfusion;
use super::english::likewise::Likewise;
use super::english::long_sentences::LongSentences;
use super::english::look_down_ones_nose::LookDownOnesNose;
use super::english::looking_forward_to::LookingForwardTo;
use super::english::mass_nouns::MassNouns;
use super::english::means_a_lot_to::MeansALotTo;
use super::english::merge_words::MergeWords;
use super::english::missing_preposition::MissingPreposition;
use super::english::missing_to::MissingTo;
use super::english::misspell::Misspell;
use super::english::mixed_bag::MixedBag;
use super::english::modal_be_adjective::ModalBeAdjective;
use super::english::modal_of::ModalOf;
use super::english::modal_seem::ModalSeem;
use super::english::months::Months;
use super::english::more_adjective::MoreAdjective;
use super::english::more_better::MoreBetter;
use super::english::most_number::MostNumber;
use super::english::most_of_the_times::MostOfTheTimes;
use super::english::multiple_frequency_adverbs::MultipleFrequencyAdverbs;
use super::english::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::english::nail_on_the_head::NailOnTheHead;
use super::english::need_to_noun::NeedToNoun;
use super::english::no_french_spaces::NoFrenchSpaces;
use super::english::no_longer::NoLonger;
use super::english::no_match_for::NoMatchFor;
use super::english::no_oxford_comma::NoOxfordComma;
use super::english::nobody::Nobody;
use super::english::nominal_wants::NominalWants;
use super::english::nor_modal_pronoun::NorModalPronoun;
use super::english::not_only_inversion::NotOnlyInversion;
use super::english::noun_verb_confusion::NounVerbConfusion;
use super::english::number_suffix_capitalization::NumberSuffixCapitalization;
use super::english::obsess_preposition::ObsessPreposition;
use super::english::of_course::OfCourse;
use super::english::oldest_in_the_book::OldestInTheBook;
use super::english::on_floor::OnFloor;
use super::english::once_or_twice::OnceOrTwice;
use super::english::one_and_the_same::OneAndTheSame;
use super::english::one_of_the_singular::OneOfTheSingular;
use super::english::open_the_light::OpenTheLight;
use super::english::orthographic_consistency::OrthographicConsistency;
use super::english::ought_to_be::OughtToBe;
use super::english::out_of_date::OutOfDate;
use super::english::oxford_comma::OxfordComma;
use super::english::oxymorons::Oxymorons;
use super::english::phrasal_verb_as_compound_noun::PhrasalVerbAsCompoundNoun;
use super::english::pique_interest::PiqueInterest;
use super::english::plural_decades::PluralDecades;
use super::english::plural_wrong_word_of_phrase::PluralWrongWordOfPhrase;
use super::english::possessive_noun::PossessiveNoun;
use super::english::possessive_your::PossessiveYour;
use super::english::progressive_needs_be::ProgressiveNeedsBe;
use super::english::pronoun_are::PronounAre;
use super::english::pronoun_contraction::PronounContraction;
use super::english::pronoun_inflection_be::PronounInflectionBe;
use super::english::pronoun_knew::PronounKnew;
use super::english::pronoun_verb_agreement::PronounVerbAgreement;
use super::english::proper_noun_capitalization_linters;
use super::english::quantifier_needs_of::QuantifierNeedsOf;
use super::english::quantifier_numeral_conflict::QuantifierNumeralConflict;
use super::english::quite_quiet::QuiteQuiet;
use super::english::quote_spacing::QuoteSpacing;
use super::english::reason_for_doing::ReasonForDoing;
use super::english::redundant_acronyms::RedundantAcronyms;
use super::english::redundant_additive_adverbs::RedundantAdditiveAdverbs;
use super::english::redundant_progressive_comparative::RedundantProgressiveComparative;
use super::english::regionalisms::Regionalisms;
use super::english::regular_irregulars::RegularIrregulars;
use super::english::repeated_words::RepeatedWords;
use super::english::respond::Respond;
use super::english::right_click::RightClick;
use super::english::rise_the_ranks::RiseTheRanks;
use super::english::roller_skated::RollerSkated;
use super::english::safe_to_save::SafeToSave;
use super::english::save_to_safe::SaveToSafe;
use super::english::sentence_capitalization::SentenceCapitalization;
use super::english::shoot_oneself_in_the_foot::ShootOneselfInTheFoot;
use super::english::simple_past_to_past_participle::SimplePastToPastParticiple;
use super::english::since_duration::SinceDuration;
use super::english::single_be::SingleBe;
use super::english::some_without_article::SomeWithoutArticle;
use super::english::something_is::SomethingIs;
use super::english::somewhat_something::SomewhatSomething;
use super::english::soon_to_be::SoonToBe;
use super::english::sought_after::SoughtAfter;
use super::english::spaces::Spaces;
use super::english::spell_check::SpellCheck;
use super::english::spelled_numbers::SpelledNumbers;
use super::english::split_words::SplitWords;
use super::english::subject_pronoun::SubjectPronoun;
use super::english::take_a_look_to::TakeALookTo;
use super::english::take_medicine::TakeMedicine;
use super::english::that_than::ThatThan;
use super::english::that_which::ThatWhich;
use super::english::the_how_why::TheHowWhy;
use super::english::the_my::TheMy;
use super::english::the_point_for::ThePointFor;
use super::english::the_proper_noun_possessive::TheProperNounPossessive;
use super::english::then_than::ThenThan;
use super::english::theres::Theres;
use super::english::theses_these::ThesesThese;
use super::english::theyre_confusions::TheyreConfusions;
use super::english::thing_think::ThingThink;
use super::english::this_type_of_thing::ThisTypeOfThing;
use super::english::though_thought::ThoughThought;
use super::english::throw_away::ThrowAway;
use super::english::throw_rubbish::ThrowRubbish;
use super::english::to_adverb::ToAdverb;
use super::english::to_two_too::ToTwoToo;
use super::english::touristic::Touristic;
use super::english::transposed_space::TransposedSpace;
use super::english::try_ones_hand_at::TryOnesHandAt;
use super::english::unclosed_quotes::UnclosedQuotes;
use super::english::update_place_names::UpdatePlaceNames;
use super::english::use_title_case::UseTitleCase;
use super::english::verb_to_adjective::VerbToAdjective;
use super::english::very_unique::VeryUnique;
use super::english::vice_versa::ViceVersa;
use super::english::vicious_loop::ViciousCircle;
use super::english::vicious_loop::ViciousCircleOrCycle;
use super::english::vicious_loop::ViciousCycle;
use super::english::was_aloud::WasAloud;
use super::english::way_too_adjective::WayTooAdjective;
use super::english::well_educated::WellEducated;
use super::english::were_where::WereWhere;
use super::english::whereas::Whereas;
use super::english::whom_subject_of_verb::WhomSubjectOfVerb;
use super::english::widely_accepted::WidelyAccepted;
use super::english::win_prize::WinPrize;
use super::english::wish_could::WishCould;
use super::english::wordpress_dotcom::WordPressDotcom;
use super::english::worth_to_do::WorthToDo;
use super::english::would_never_have::WouldNeverHave;
use super::english::wrong_apostrophe::WrongApostrophe;
use super::english::{ExprLinter, Lint};
use super::english::{HtmlDescriptionLinter, Linter};
use crate::english::linting::dashes::Dashes;
use crate::english::linting::expr_linter::Chunk;
use crate::english::linting::open_compounds::OpenCompounds;
use crate::english::linting::{closed_compounds, initialisms, phrase_set_corrections, weir_rules};
use crate::english::spell::{Dictionary, MutableDictionary};
use crate::english::{CharString, Document, EnglishDialect, PortugueseDialect, TokenStringExt};

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
    let group = LintGroup::new_curated(
        MutableDictionary::new().into(),
        Language::English(EnglishDialect::American),
    );
    group.config
}

impl LintGroupConfig {
    /// Check if a rule exists in the configuration.
    pub fn has_rule(&self, key: impl AsRef<str>) -> bool {
        self.inner.contains_key(key.as_ref())
    }

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
/// Each child can be toggled via the public, mutable `Self::config` object.
pub struct LintGroup {
    pub config: LintGroupConfig,
    /// We use a binary map here so the ordering is stable.
    linters: BTreeMap<String, Box<dyn Linter>>,
    /// We use a binary map here so the ordering is stable.
    chunk_expr_linters: BTreeMap<String, Box<dyn ExprLinter<Unit = Chunk>>>,
    /// Since [`ExprLinter`]s operate on a chunk-basis, we can store a
    /// mapping of `Chunk -> Lint` and only re-run the expr linters
    /// when a chunk changes.
    ///
    /// Since the expr linter results also depend on the config, we hash it and pass it as part
    /// of the key.
    chunk_expr_cache: LruCache<(CharString, u64), BTreeMap<String, Vec<Lint>>>,
    hasher_builder: RandomState,
    clashing_linter_names: Option<Vec<String>>,
}

impl LintGroup {
    // Constructor methods

    pub fn empty() -> Self {
        Self {
            config: LintGroupConfig::default(),
            linters: BTreeMap::new(),
            chunk_expr_linters: BTreeMap::new(),
            chunk_expr_cache: LruCache::new(NonZero::new(1000).unwrap()),
            hasher_builder: RandomState::default(),
            clashing_linter_names: None,
        }
    }

    // Non-constructor methods

    /// Check if the group already contains a linter with a given name.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.linters.contains_key(name.as_ref())
            || self.chunk_expr_linters.contains_key(name.as_ref())
    }

    /// Add a [`Linter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    pub fn add(&mut self, name: impl AsRef<str>, linter: impl Linter + 'static) -> bool {
        if self.contains_key(&name) {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![name.as_ref().to_string()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(name.as_ref().to_string());
            }
            false
        } else {
            self.linters
                .insert(name.as_ref().to_string(), Box::new(linter));
            true
        }
    }

    /// Add a chunk-based [`ExprLinter`] to the group, returning whether the operation was successful.
    /// If it returns `false`, it is because a linter with that key already existed in the group.
    ///
    /// This function is not significantly different from [`Self::add`], but allows us to take
    /// advantage of some properties of chunk-based [`ExprLinter`]s for cache optimization.
    pub fn add_chunk_expr_linter(
        &mut self,
        name: impl AsRef<str>,
        // linter: impl ExprLinter + 'static,
        linter: impl ExprLinter<Unit = Chunk> + 'static,
    ) -> bool {
        if self.contains_key(&name) {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![name.as_ref().to_string()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(name.as_ref().to_string());
            }
            false
        } else {
            self.chunk_expr_linters
                .insert(name.as_ref().to_string(), Box::new(linter) as _);
            true
        }
    }

    /// Merge the contents of another [`LintGroup`] into this one.
    /// The other lint group will be left empty after this operation.
    pub fn merge_from(&mut self, other: &mut LintGroup) {
        self.config.merge_from(&mut other.config);

        let other_linters = std::mem::take(&mut other.linters);
        if let Some((conflicting_key, _)) = other_linters.iter().find(|(k, _)| self.contains_key(k))
        {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![conflicting_key.clone()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(conflicting_key.clone());
            }
        }
        self.linters.extend(other_linters);

        let other_expr_linters = std::mem::take(&mut other.chunk_expr_linters);
        if let Some((conflicting_key, _)) = other_expr_linters
            .iter()
            .find(|(k, _)| self.contains_key(k))
        {
            if self.clashing_linter_names.is_none() {
                self.clashing_linter_names = Some(vec![conflicting_key.clone()]);
            } else if let Some(clashing_names) = &mut self.clashing_linter_names {
                clashing_names.push(conflicting_key.clone());
            }
        }
        self.chunk_expr_linters.extend(other_expr_linters);
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &str> {
        self.linters
            .keys()
            .chain(self.chunk_expr_linters.keys())
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
                self.chunk_expr_linters
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
                self.chunk_expr_linters
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

    pub fn new_curated(dictionary: Arc<impl Dictionary + 'static>, language: Language) -> Self {
        match language {
            Language::English(english_dialect) => {
                Self::new_curated_english(dictionary, english_dialect)
            }
            Language::Portuguese(portuguese_dialect) => {
                Self::new_curated_portuguese(dictionary, portuguese_dialect)
            }
        }
    }
    pub fn new_curated_english(
        dictionary: Arc<impl Dictionary + 'static>,
        dialect: EnglishDialect,
    ) -> Self {
        let mut out = Self::empty();

        /// Add a `Linter` to the group, setting it to be enabled or disabled.
        macro_rules! insert_struct_rule {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), $rule::default());
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a `Linter` that requires a `Dictionary` to the group, setting it to be enabled or disabled.
        macro_rules! insert_struct_rule_with_dict {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), $rule::new(dictionary.clone()));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a `Linter` that requires a `Dialect` to the group, setting it to be enabled or disabled.
        macro_rules! insert_struct_rule_with_dialect {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), $rule::new(dialect));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a chunk-based `ExprLinter` to the group, setting it to be enabled or disabled.
        /// While you _can_ pass an `ExprLinter` to `insert_struct_rule`, using this macro instead
        /// will allow it to use more aggressive caching strategies.
        macro_rules! insert_expr_rule {
            ($rule:ident, $default_config:expr) => {
                out.add_chunk_expr_linter(stringify!($rule), $rule::default());
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a chunk-based `ExprLinter` that requires a `Dictionary` to the group, setting it to be enabled or disabled.
        macro_rules! insert_expr_rule_with_dict {
            ($rule:ident, $default_config:expr) => {
                out.add_chunk_expr_linter(stringify!($rule), $rule::new(dictionary.clone()));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a chunk-based `ExprLinter` that requires a `Dialect` to the group, setting it to be enabled or disabled.
        macro_rules! insert_expr_rule_with_dialect {
            ($rule:ident, $default_config:expr) => {
                out.add_chunk_expr_linter(stringify!($rule), $rule::new(dialect));
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        out.merge_from(&mut weir_rules::lint_group());
        out.merge_from(&mut phrase_set_corrections::lint_group());
        out.merge_from(&mut proper_noun_capitalization_linters::lint_group(
            dictionary.clone(),
        ));
        out.merge_from(&mut closed_compounds::lint_group());
        out.merge_from(&mut initialisms::lint_group());

        // Add all the more complex rules to the group.
        // Please maintain alphabetical order.
        // On *nix you can maintain sort order with `sort -t'(' -k2`
        insert_expr_rule!(APart, true);
        insert_expr_rule!(AWhile, true);
        insert_expr_rule!(Addicting, true);
        insert_expr_rule!(AdjectiveDoubleDegree, true);
        insert_struct_rule!(AdjectiveOfA, true);
        insert_expr_rule!(AfterLater, true);
        insert_expr_rule!(AllHellBreakLoose, true);
        insert_expr_rule!(AllIntentsAndPurposes, true);
        insert_expr_rule!(AllowTo, true);
        insert_expr_rule!(AmInTheMorning, true);
        insert_expr_rule!(AmountsFor, true);
        insert_struct_rule_with_dialect!(AnA, true);
        insert_expr_rule!(AndIn, true);
        insert_expr_rule!(AndTheLike, true);
        insert_expr_rule!(AnotherThingComing, true);
        insert_expr_rule!(AnotherThinkComing, false);
        insert_expr_rule!(ApartFrom, true);
        insert_expr_rule!(AskNoPreposition, true);
        insert_expr_rule!(AvoidCurses, true);
        insert_expr_rule!(BackInTheDay, true);
        insert_expr_rule!(BeAllowed, true);
        insert_expr_rule!(BeWorried, true);
        insert_expr_rule!(BehindTheScenes, true);
        insert_struct_rule!(BestOfAllTime, true);
        insert_expr_rule!(BoringWords, false);
        insert_expr_rule!(Bought, true);
        insert_expr_rule!(BrandBrandish, true);
        insert_expr_rule!(ByAccident, true);
        insert_expr_rule!(Cant, true);
        insert_struct_rule!(CapitalizePersonalPronouns, true);
        insert_expr_rule!(CautionaryTale, true);
        insert_expr_rule!(ChangeTack, true);
        insert_expr_rule!(ChockFull, true);
        insert_struct_rule!(CommaFixes, true);
        insert_struct_rule!(CompoundNouns, true);
        insert_expr_rule!(CompoundSubjectI, true);
        insert_expr_rule!(Confident, true);
        insert_struct_rule!(CorrectNumberSuffix, true);
        insert_expr_rule!(CriteriaPhenomena, true);
        insert_expr_rule!(CureFor, true);
        insert_struct_rule!(CurrencyPlacement, true);
        insert_expr_rule!(Dashes, true);
        insert_expr_rule!(DayAndAge, true);
        insert_expr_rule!(DespiteItIs, true);
        insert_expr_rule!(DespiteOf, true);
        insert_expr_rule_with_dict!(DidPast, true);
        insert_expr_rule!(Didnt, true);
        insert_struct_rule!(DiscourseMarkers, true);
        insert_expr_rule_with_dict!(DisjointPrefixes, true);
        insert_expr_rule!(DoMistake, true);
        insert_expr_rule!(DotInitialisms, true);
        insert_expr_rule!(DoubleClick, true);
        insert_expr_rule!(DoubleModal, true);
        insert_struct_rule!(EllipsisLength, true);
        insert_expr_rule!(ElsePossessive, true);
        insert_expr_rule!(EverEvery, true);
        insert_expr_rule!(Everyday, true);
        insert_expr_rule!(ExceptOf, true);
        insert_expr_rule!(ExpandMemoryShorthands, true);
        insert_expr_rule!(ExpandTimeShorthands, true);
        insert_expr_rule!(FarBeIt, true);
        insert_expr_rule!(FascinatedBy, true);
        insert_expr_rule_with_dialect!(FedUpWith, true);
        insert_expr_rule!(FeelFell, true);
        insert_expr_rule!(FewUnitsOfTimeAgo, true);
        insert_expr_rule!(FillerWords, true);
        insert_struct_rule!(FindFine, true);
        insert_expr_rule!(FirstAidKit, true);
        insert_expr_rule!(FleshOutVsFullFledged, true);
        insert_expr_rule!(ForNoun, true);
        insert_expr_rule!(FreePredicate, true);
        insert_expr_rule!(FriendOfMe, true);
        insert_expr_rule!(GoSoFarAsTo, true);
        insert_expr_rule!(GoToWar, true);
        insert_expr_rule!(GoodAt, true);
        insert_expr_rule!(Handful, true);
        insert_expr_rule!(HavePronoun, true);
        insert_struct_rule_with_dialect!(HaveTakeALook, true);
        insert_expr_rule!(Hedging, true);
        insert_expr_rule!(HelloGreeting, true);
        insert_expr_rule!(Hereby, true);
        insert_struct_rule!(HopHope, true);
        insert_expr_rule!(HowTo, true);
        insert_expr_rule!(HyphenateNumberDay, true);
        insert_expr_rule!(IAmAgreement, true);
        insert_expr_rule!(IfWouldve, true);
        insert_struct_rule_with_dialect!(InOnTheCards, true);
        insert_struct_rule_with_dict!(InflectedVerbAfterTo, true);
        insert_expr_rule!(InterestedIn, true);
        insert_expr_rule!(ItLooksLikeThat, true);
        insert_struct_rule!(ItsContraction, true);
        insert_expr_rule!(ItsPossessive, true);
        insert_expr_rule!(JealousOf, true);
        insert_expr_rule!(JohnsHopkins, true);
        insert_expr_rule!(LeadRiseTo, true);
        insert_expr_rule!(LeftRightHand, true);
        insert_expr_rule!(LessWorse, true);
        insert_expr_rule!(LetToDo, true);
        insert_struct_rule!(LetsConfusion, true);
        insert_expr_rule!(Likewise, true);
        insert_struct_rule!(LongSentences, true);
        insert_expr_rule!(LookDownOnesNose, true);
        insert_expr_rule!(LookingForwardTo, true);
        insert_struct_rule_with_dict!(MassNouns, true);
        insert_expr_rule!(MeansALotTo, true);
        insert_struct_rule!(MergeWords, true);
        insert_expr_rule!(MissingPreposition, true);
        insert_expr_rule!(MissingTo, true);
        insert_expr_rule!(Misspell, true);
        insert_expr_rule!(MixedBag, true);
        insert_expr_rule!(ModalBeAdjective, true);
        insert_expr_rule!(ModalOf, true);
        insert_expr_rule!(ModalSeem, true);
        insert_expr_rule!(Months, true);
        insert_expr_rule_with_dict!(MoreAdjective, true);
        insert_expr_rule!(MoreBetter, true);
        insert_expr_rule!(MostNumber, true);
        insert_expr_rule!(MostOfTheTimes, true);
        insert_expr_rule!(MultipleSequentialPronouns, true);
        insert_expr_rule!(NailOnTheHead, true);
        insert_expr_rule!(NeedToNoun, true);
        insert_struct_rule!(NoFrenchSpaces, true);
        insert_expr_rule!(NoLonger, true);
        insert_expr_rule!(NoMatchFor, true);
        insert_struct_rule!(NoOxfordComma, false);
        insert_expr_rule!(Nobody, true);
        insert_expr_rule!(NominalWants, true);
        insert_expr_rule!(NorModalPronoun, true);
        insert_expr_rule!(NotOnlyInversion, true);
        insert_struct_rule!(NounVerbConfusion, true);
        insert_struct_rule!(NumberSuffixCapitalization, true);
        insert_expr_rule!(ObsessPreposition, true);
        insert_expr_rule!(OfCourse, true);
        insert_expr_rule!(OldestInTheBook, true);
        insert_expr_rule!(OnFloor, true);
        insert_expr_rule!(OnceOrTwice, true);
        insert_expr_rule!(OneAndTheSame, true);
        insert_expr_rule_with_dict!(OneOfTheSingular, true);
        insert_expr_rule!(OpenCompounds, true);
        insert_expr_rule!(OpenTheLight, true);
        insert_expr_rule!(OrthographicConsistency, true);
        insert_expr_rule!(OughtToBe, true);
        insert_expr_rule!(OutOfDate, true);
        insert_struct_rule!(OxfordComma, true);
        insert_expr_rule!(Oxymorons, true);
        insert_struct_rule!(PhrasalVerbAsCompoundNoun, true);
        insert_expr_rule!(PiqueInterest, true);
        insert_expr_rule!(PluralWrongWordOfPhrase, true);
        insert_struct_rule_with_dict!(PossessiveNoun, false);
        insert_expr_rule!(PossessiveYour, true);
        insert_expr_rule!(ProgressiveNeedsBe, true);
        insert_expr_rule!(PronounAre, true);
        insert_struct_rule!(PronounContraction, true);
        insert_expr_rule!(PronounInflectionBe, true);
        insert_expr_rule!(PronounKnew, true);
        insert_expr_rule_with_dict!(PronounVerbAgreement, true);
        insert_expr_rule!(QuantifierNeedsOf, true);
        insert_expr_rule!(QuantifierNumeralConflict, true);
        insert_expr_rule!(QuiteQuiet, true);
        insert_struct_rule!(QuoteSpacing, true);
        insert_expr_rule!(ReasonForDoing, true);
        insert_expr_rule!(RedundantAcronyms, true);
        insert_expr_rule!(RedundantAdditiveAdverbs, true);
        insert_expr_rule!(RedundantProgressiveComparative, true);
        insert_struct_rule_with_dialect!(Regionalisms, true);
        insert_expr_rule_with_dict!(RegularIrregulars, true);
        insert_struct_rule!(RepeatedWords, true);
        insert_expr_rule!(Respond, true);
        insert_expr_rule!(RightClick, true);
        insert_expr_rule!(RiseTheRanks, true);
        insert_expr_rule!(RollerSkated, true);
        insert_expr_rule!(SafeToSave, true);
        insert_expr_rule!(SaveToSafe, true);
        insert_struct_rule_with_dict!(SentenceCapitalization, true);
        insert_expr_rule!(ShootOneselfInTheFoot, true);
        insert_expr_rule!(SimplePastToPastParticiple, true);
        insert_expr_rule!(SinceDuration, true);
        insert_expr_rule!(SingleBe, true);
        insert_expr_rule!(SomeWithoutArticle, true);
        insert_expr_rule!(SomethingIs, true);
        insert_expr_rule!(SomewhatSomething, true);
        insert_expr_rule!(SoonToBe, true);
        insert_expr_rule!(SoughtAfter, true);
        insert_struct_rule!(Spaces, true);
        insert_struct_rule!(SpelledNumbers, false);
        insert_expr_rule!(SplitWords, true);
        insert_struct_rule!(SubjectPronoun, true);
        insert_expr_rule!(TakeALookTo, true);
        insert_expr_rule!(TakeMedicine, true);
        insert_expr_rule!(ThatThan, true);
        insert_expr_rule!(ThatWhich, true);
        insert_expr_rule!(TheHowWhy, true);
        insert_expr_rule!(TheMy, true);
        insert_expr_rule!(ThePointFor, true);
        insert_expr_rule!(TheProperNounPossessive, true);
        insert_expr_rule!(ThenThan, true);
        insert_expr_rule!(Theres, true);
        insert_expr_rule!(ThesesThese, true);
        insert_struct_rule!(TheyreConfusions, true);
        insert_expr_rule!(ThingThink, true);
        insert_expr_rule!(ThisTypeOfThing, true);
        insert_expr_rule!(ThoughThought, true);
        insert_expr_rule!(ThrowAway, true);
        insert_struct_rule!(ThrowRubbish, true);
        insert_expr_rule!(ToAdverb, true);
        insert_struct_rule!(ToTwoToo, true);
        insert_expr_rule!(Touristic, true);
        insert_expr_rule_with_dict!(TransposedSpace, true);
        insert_expr_rule!(TryOnesHandAt, true);
        insert_struct_rule!(UnclosedQuotes, true);
        insert_expr_rule!(UpdatePlaceNames, true);
        insert_struct_rule_with_dict!(UseTitleCase, true);
        insert_expr_rule!(VerbToAdjective, true);
        insert_expr_rule!(VeryUnique, true);
        insert_expr_rule!(ViceVersa, true);
        insert_expr_rule!(ViciousCircle, true);
        insert_expr_rule!(ViciousCircleOrCycle, false);
        insert_expr_rule!(ViciousCycle, false);
        insert_expr_rule!(WasAloud, true);
        insert_expr_rule!(WayTooAdjective, true);
        insert_expr_rule!(WellEducated, true);
        insert_expr_rule!(WereWhere, true);
        insert_expr_rule!(Whereas, true);
        insert_expr_rule!(WhomSubjectOfVerb, true);
        insert_expr_rule!(WidelyAccepted, true);
        insert_expr_rule!(WinPrize, true);
        insert_expr_rule!(WishCould, true);
        insert_struct_rule!(WordPressDotcom, true);
        insert_expr_rule_with_dict!(WorthToDo, true);
        insert_expr_rule!(WouldNeverHave, true);
        insert_expr_rule!(WrongApostrophe, true);

        // Uses Sentence rather than Chunk
        out.add("AspireTo", AspireTo::default());
        out.config.set_rule_enabled("AspireTo", true);

        // Uses Sentence rather than Chunk
        out.add("Damages", Damages::default());
        out.config.set_rule_enabled("Damages", true);

        // Uses Sentence rather than Chunk
        out.add(
            "MultipleFrequencyAdverbs",
            MultipleFrequencyAdverbs::default(),
        );
        out.config
            .set_rule_enabled("MultipleFrequencyAdverbs", true);

        // Uses Sentence rather than Chunk
        out.add("PluralDecades", PluralDecades::default());
        out.config.set_rule_enabled("PluralDecades", true);

        // Uses Dictionary and Dialect
        out.add(
            "SpellCheck",
            SpellCheck::new(dictionary.clone(), dialect.into()),
        );
        out.config.set_rule_enabled("SpellCheck", true);

        out
    }

    /// Create a new curated group with all config values cleared out.
    pub fn new_curated_empty_config(
        dictionary: Arc<impl Dictionary + 'static>,
        language: Language,
    ) -> Self {
        let mut group = Self::new_curated(dictionary, language);
        group.config.clear();
        group
    }

    pub fn organized_lints(&mut self, document: &Document) -> BTreeMap<String, Vec<Lint>> {
        let mut results = BTreeMap::new();

        // Normal linters
        for (key, linter) in &mut self.linters {
            if self.config.is_rule_enabled(key) {
                results.insert(key.clone(), linter.lint(document));
            }
        }

        // Expr linters
        for chunk in document.iter_chunks() {
            let Some(chunk_span) = chunk.span() else {
                continue;
            };

            let chunk_chars = document.get_span_content(&chunk_span);
            let config_hash = self.hasher_builder.hash_one(&self.config);
            let cache_key = (chunk_chars.into(), config_hash);

            let mut chunk_results = if let Some(hit) = self.chunk_expr_cache.get(&cache_key) {
                hit.clone()
            } else {
                let mut pattern_lints = BTreeMap::new();

                for (key, linter) in &mut self.chunk_expr_linters {
                    if self.config.is_rule_enabled(key) {
                        let lints =
                            run_on_chunk(linter, chunk, document.get_source()).map(|mut l| {
                                l.span.pull_by(chunk_span.start);
                                l
                            });

                        pattern_lints.insert(key.clone(), lints.collect());
                    }
                }

                self.chunk_expr_cache.put(cache_key, pattern_lints.clone());
                pattern_lints
            };

            // Bring the spans back into document-space
            for value in chunk_results.values_mut() {
                for lint in value {
                    lint.span.push_by(chunk_span.start);
                }
            }

            for (key, mut vec) in chunk_results {
                results.entry(key).or_default().append(&mut vec);
            }
        }

        results
    }
}

impl Default for LintGroup {
    fn default() -> Self {
        Self::empty()
    }
}

impl Linter for LintGroup {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.organized_lints(document)
            .into_values()
            .flatten()
            .collect()
    }

    fn description(&self) -> &str {
        "A collection of linters that can be run as one."
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{LintGroup, LintGroupConfig};
    use crate::languages::Language;
    use crate::linting::LintKind;
    use crate::linting::tests::assert_no_lints;
    use crate::spell::{FstDictionary, MutableDictionary};
    use crate::{Dialect, Document, linting::Linter};

    fn test_group() -> LintGroup {
        LintGroup::new_curated(
            Arc::new(MutableDictionary::curated()),
            Language::English(EnglishDialect::American),
        )
    }

    #[test]
    fn clean_interjection() {
        assert_no_lints(
            "Although I only saw the need to interject once, I still saw it.",
            test_group(),
        );
    }

    #[test]
    fn clean_consensus() {
        assert_no_lints("But there is less consensus on this.", test_group());
    }

    #[test]
    fn can_get_all_descriptions() {
        let group = LintGroup::new_curated(
            Arc::new(MutableDictionary::default()),
            Language::English(EnglishDialect::American),
        );
        group.all_descriptions();
    }

    #[test]
    fn can_get_all_descriptions_as_html() {
        let group = LintGroup::new_curated(
            Arc::new(MutableDictionary::default()),
            Language::English(EnglishDialect::American),
        );
        group.all_descriptions_html();
    }

    #[test]
    fn dont_flag_low_hanging_fruit_msg() {
        assert_no_lints(
            "The standard form is low-hanging fruit with a hyphen and singular form.",
            test_group(),
        );
    }

    #[test]
    fn dont_flag_low_hanging_fruit_desc() {
        assert_no_lints(
            "Corrects nonstandard variants of low-hanging fruit.",
            test_group(),
        );
    }

    /// Tests that no linters' descriptions contain errors handled by other linters.
    ///
    /// This test verifies that the description of each linter (which is written in natural language)
    /// doesn't trigger any other linter's rules, with the exception of certain linters that
    /// suggest mere alternatives rather than flagging actual errors.
    ///
    /// For example, we disable the "MoreAdjective" linter since some comparative and superlative
    /// adjectives can be more awkward than their two-word counterparts, even if technically correct.
    ///
    /// If this test fails, it means either:
    /// 1. A linter's description contains an actual error that should be fixed, or
    /// 2. A linter is being too aggressive in flagging text that is actually correct English
    ///    in the context of another linter's description.
    #[test]
    fn lint_descriptions_are_clean() {
        let lints_to_check = LintGroup::new_curated(
            FstDictionary::curated(crate::languages::LanguageFamily::English),
            Language::English(EnglishDialect::American),
        );

        let enforcer_config = LintGroupConfig::new_curated();
        let mut lints_to_enforce = LintGroup::new_curated(
            FstDictionary::curated(crate::languages::LanguageFamily::English),
            Language::English(EnglishDialect::American),
        )
        .with_lint_config(enforcer_config);

        let name_description_pairs: Vec<_> = lints_to_check
            .all_descriptions()
            .into_iter()
            .map(|(n, d)| (n.to_string(), d.to_string()))
            .collect();

        for (lint_name, description) in name_description_pairs {
            let doc = Document::new_markdown_default_curated(&description);
            eprintln!("{lint_name}: {description}");

            let mut lints = lints_to_enforce.lint(&doc);

            // Remove ones related to style
            lints.retain(|l| l.lint_kind != LintKind::Style);

            if !lints.is_empty() {
                dbg!(lints);
                panic!();
            }
        }
    }

    #[test]
    fn no_linter_names_clash() {
        let group =
            LintGroup::new_curated(Arc::new(MutableDictionary::default()), Dialect::American);

        if let Some(names) = &group.clashing_linter_names {
            if !names.is_empty() {
                panic!(
                    "⚠️ Found {} clashing linter names: {}",
                    names.len(),
                    names.join(", ")
                );
            }
        }
    }
}
