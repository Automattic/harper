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
use super::a_while::AWhile;
use super::addicting::Addicting;
use super::adjective_double_degree::AdjectiveDoubleDegree;
use super::adjective_of_a::AdjectiveOfA;
use super::after_later::AfterLater;
use super::all_intents_and_purposes::AllIntentsAndPurposes;
use super::allow_to::AllowTo;
use super::am_in_the_morning::AmInTheMorning;
use super::amounts_for::AmountsFor;
use super::an_a::AnA;
use super::and_in::AndIn;
use super::and_the_like::AndTheLike;
use super::another_thing_coming::AnotherThingComing;
use super::another_think_coming::AnotherThinkComing;
use super::apart_from::ApartFrom;
use super::ask_no_preposition::AskNoPreposition;
use super::avoid_curses::AvoidCurses;
use super::back_in_the_day::BackInTheDay;
use super::be_allowed::BeAllowed;
use super::behind_the_scenes::BehindTheScenes;
use super::best_of_all_time::BestOfAllTime;
use super::boring_words::BoringWords;
use super::bought::Bought;
use super::brand_brandish::BrandBrandish;
use super::cant::Cant;
use super::capitalize_personal_pronouns::CapitalizePersonalPronouns;
use super::cautionary_tale::CautionaryTale;
use super::change_tack::ChangeTack;
use super::chock_full::ChockFull;
use super::comma_fixes::CommaFixes;
use super::compound_nouns::CompoundNouns;
use super::compound_subject_i::CompoundSubjectI;
use super::confident::Confident;
use super::correct_number_suffix::CorrectNumberSuffix;
use super::criteria_phenomena::CriteriaPhenomena;
use super::cure_for::CureFor;
use super::currency_placement::CurrencyPlacement;
use super::damages::Damages;
use super::day_and_age::DayAndAge;
use super::despite_it_is::DespiteItIs;
use super::despite_of::DespiteOf;
use super::did_past::DidPast;
use super::didnt::Didnt;
use super::discourse_markers::DiscourseMarkers;
use super::disjoint_prefixes::DisjointPrefixes;
use super::dot_initialisms::DotInitialisms;
use super::double_click::DoubleClick;
use super::double_modal::DoubleModal;
use super::ellipsis_length::EllipsisLength;
use super::else_possessive::ElsePossessive;
use super::ever_every::EverEvery;
use super::everyday::Everyday;
use super::expand_memory_shorthands::ExpandMemoryShorthands;
use super::expand_time_shorthands::ExpandTimeShorthands;
use super::expr_linter::run_on_chunk;
use super::far_be_it::FarBeIt;
use super::fascinated_by::FascinatedBy;
use super::feel_fell::FeelFell;
use super::few_units_of_time_ago::FewUnitsOfTimeAgo;
use super::filler_words::FillerWords;
use super::find_fine::FindFine;
use super::first_aid_kit::FirstAidKit;
use super::flesh_out_vs_full_fledged::FleshOutVsFullFledged;
use super::for_noun::ForNoun;
use super::free_predicate::FreePredicate;
use super::friend_of_me::FriendOfMe;
use super::go_so_far_as_to::GoSoFarAsTo;
use super::go_to_war::GoToWar;
use super::good_at::GoodAt;
use super::handful::Handful;
use super::have_pronoun::HavePronoun;
use super::have_take_a_look::HaveTakeALook;
use super::hedging::Hedging;
use super::hello_greeting::HelloGreeting;
use super::hereby::Hereby;
use super::hop_hope::HopHope;
use super::how_to::HowTo;
use super::hyphenate_number_day::HyphenateNumberDay;
use super::i_am_agreement::IAmAgreement;
use super::if_wouldve::IfWouldve;
use super::in_on_the_cards::InOnTheCards;
use super::inflected_verb_after_to::InflectedVerbAfterTo;
use super::interested_in::InterestedIn;
use super::it_looks_like_that::ItLooksLikeThat;
use super::its_contraction::ItsContraction;
use super::its_possessive::ItsPossessive;
use super::jealous_of::JealousOf;
use super::johns_hopkins::JohnsHopkins;
use super::lead_rise_to::LeadRiseTo;
use super::left_right_hand::LeftRightHand;
use super::less_worse::LessWorse;
use super::let_to_do::LetToDo;
use super::lets_confusion::LetsConfusion;
use super::likewise::Likewise;
use super::long_sentences::LongSentences;
use super::look_down_ones_nose::LookDownOnesNose;
use super::looking_forward_to::LookingForwardTo;
use super::mass_nouns::MassNouns;
use super::merge_words::MergeWords;
use super::missing_preposition::MissingPreposition;
use super::missing_to::MissingTo;
use super::misspell::Misspell;
use super::mixed_bag::MixedBag;
use super::modal_be_adjective::ModalBeAdjective;
use super::modal_of::ModalOf;
use super::modal_seem::ModalSeem;
use super::months::Months;
use super::more_adjective::MoreAdjective;
use super::more_better::MoreBetter;
use super::most_number::MostNumber;
use super::most_of_the_times::MostOfTheTimes;
use super::multiple_sequential_pronouns::MultipleSequentialPronouns;
use super::nail_on_the_head::NailOnTheHead;
use super::need_to_noun::NeedToNoun;
use super::no_french_spaces::NoFrenchSpaces;
use super::no_longer::NoLonger;
use super::no_match_for::NoMatchFor;
use super::no_oxford_comma::NoOxfordComma;
use super::nobody::Nobody;
use super::nominal_wants::NominalWants;
use super::nor_modal_pronoun::NorModalPronoun;
use super::noun_verb_confusion::NounVerbConfusion;
use super::number_suffix_capitalization::NumberSuffixCapitalization;
use super::obsess_preposition::ObsessPreposition;
use super::of_course::OfCourse;
use super::oldest_in_the_book::OldestInTheBook;
use super::on_floor::OnFloor;
use super::once_or_twice::OnceOrTwice;
use super::one_and_the_same::OneAndTheSame;
use super::one_of_the_singular::OneOfTheSingular;
use super::open_the_light::OpenTheLight;
use super::orthographic_consistency::OrthographicConsistency;
use super::ought_to_be::OughtToBe;
use super::out_of_date::OutOfDate;
use super::oxford_comma::OxfordComma;
use super::oxymorons::Oxymorons;
use super::phrasal_verb_as_compound_noun::PhrasalVerbAsCompoundNoun;
use super::pique_interest::PiqueInterest;
use super::plural_wrong_word_of_phrase::PluralWrongWordOfPhrase;
use super::possessive_noun::PossessiveNoun;
use super::possessive_your::PossessiveYour;
use super::progressive_needs_be::ProgressiveNeedsBe;
use super::pronoun_are::PronounAre;
use super::pronoun_contraction::PronounContraction;
use super::pronoun_inflection_be::PronounInflectionBe;
use super::pronoun_knew::PronounKnew;
use super::pronoun_verb_agreement::PronounVerbAgreement;
use super::proper_noun_capitalization_linters;
use super::quantifier_needs_of::QuantifierNeedsOf;
use super::quantifier_numeral_conflict::QuantifierNumeralConflict;
use super::quite_quiet::QuiteQuiet;
use super::quote_spacing::QuoteSpacing;
use super::redundant_acronyms::RedundantAcronyms;
use super::redundant_additive_adverbs::RedundantAdditiveAdverbs;
use super::regionalisms::Regionalisms;
use super::repeated_words::RepeatedWords;
use super::respond::Respond;
use super::right_click::RightClick;
use super::roller_skated::RollerSkated;
use super::safe_to_save::SafeToSave;
use super::save_to_safe::SaveToSafe;
use super::semicolon_apostrophe::SemicolonApostrophe;
use super::sentence_capitalization::SentenceCapitalization;
use super::shoot_oneself_in_the_foot::ShootOneselfInTheFoot;
use super::simple_past_to_past_participle::SimplePastToPastParticiple;
use super::since_duration::SinceDuration;
use super::single_be::SingleBe;
use super::some_without_article::SomeWithoutArticle;
use super::something_is::SomethingIs;
use super::somewhat_something::SomewhatSomething;
use super::soon_to_be::SoonToBe;
use super::sought_after::SoughtAfter;
use super::spaces::Spaces;
use super::spell_check::SpellCheck;
use super::spelled_numbers::SpelledNumbers;
use super::split_words::SplitWords;
use super::subject_pronoun::SubjectPronoun;
use super::take_a_look_to::TakeALookTo;
use super::take_medicine::TakeMedicine;
use super::that_than::ThatThan;
use super::that_which::ThatWhich;
use super::the_how_why::TheHowWhy;
use super::the_my::TheMy;
use super::the_proper_noun_possessive::TheProperNounPossessive;
use super::then_than::ThenThan;
use super::theres::Theres;
use super::theses_these::ThesesThese;
use super::thing_think::ThingThink;
use super::this_type_of_thing::ThisTypeOfThing;
use super::though_thought::ThoughThought;
use super::throw_away::ThrowAway;
use super::throw_rubbish::ThrowRubbish;
use super::to_adverb::ToAdverb;
use super::to_two_too::ToTwoToo;
use super::touristic::Touristic;
use super::transposed_space::TransposedSpace;
use super::try_ones_hand_at::TryOnesHandAt;
use super::unclosed_quotes::UnclosedQuotes;
use super::update_place_names::UpdatePlaceNames;
use super::use_title_case::UseTitleCase;
use super::verb_to_adjective::VerbToAdjective;
use super::very_unique::VeryUnique;
use super::vice_versa::ViceVersa;
use super::vicious_loop::ViciousCircle;
use super::vicious_loop::ViciousCircleOrCycle;
use super::vicious_loop::ViciousCycle;
use super::was_aloud::WasAloud;
use super::way_too_adjective::WayTooAdjective;
use super::well_educated::WellEducated;
use super::whereas::Whereas;
use super::whom_subject_of_verb::WhomSubjectOfVerb;
use super::widely_accepted::WidelyAccepted;
use super::win_prize::WinPrize;
use super::wish_could::WishCould;
use super::wordpress_dotcom::WordPressDotcom;
use super::worth_to_do::WorthToDo;
use super::would_never_have::WouldNeverHave;

use super::{ExprLinter, Lint};
use super::{HtmlDescriptionLinter, Linter};
use crate::linting::dashes::Dashes;
use crate::linting::expr_linter::Chunk;
use crate::linting::open_compounds::OpenCompounds;
use crate::linting::{closed_compounds, initialisms, phrase_set_corrections, weir_rules};
use crate::spell::{Dictionary, MutableDictionary};
use crate::{CharString, Dialect, Document, TokenStringExt};

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
        }
    }

    /// Swap out [`Self::config`] with another [`LintGroupConfig`].
    pub fn with_lint_config(mut self, config: LintGroupConfig) -> Self {
        self.config = config;
        self
    }

    pub fn new_curated(dictionary: Arc<impl Dictionary + 'static>, dialect: Dialect) -> Self {
        let mut out = Self::empty();

        /// Add a `Linter` to the group, setting it to be enabled by default.
        macro_rules! insert_struct_rule {
            ($rule:ident, $default_config:expr) => {
                out.add(stringify!($rule), $rule::default());
                out.config
                    .set_rule_enabled(stringify!($rule), $default_config);
            };
        }

        /// Add a chunk-based `ExprLinter` to the group, setting it to be enabled by default.
        /// While you _can_ pass an `ExprLinter` to `insert_struct_rule`, using this macro instead
        /// will allow it to use more aggressive caching strategies.
        macro_rules! insert_expr_rule {
            ($rule:ident, $default_config:expr) => {
                out.add_chunk_expr_linter(stringify!($rule), $rule::default());
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
        insert_expr_rule!(AllIntentsAndPurposes, true);
        insert_expr_rule!(AllowTo, true);
        insert_expr_rule!(AmInTheMorning, true);
        insert_expr_rule!(AmountsFor, true);
        insert_expr_rule!(AndIn, true);
        insert_expr_rule!(AndTheLike, true);
        insert_expr_rule!(AnotherThingComing, true);
        insert_expr_rule!(AnotherThinkComing, false);
        insert_expr_rule!(ApartFrom, true);
        insert_expr_rule!(AskNoPreposition, true);
        insert_expr_rule!(AvoidCurses, true);
        insert_expr_rule!(BackInTheDay, true);
        insert_expr_rule!(BeAllowed, true);
        insert_expr_rule!(BehindTheScenes, true);
        insert_struct_rule!(BestOfAllTime, true);
        insert_expr_rule!(BoringWords, false);
        insert_expr_rule!(Bought, true);
        insert_expr_rule!(BrandBrandish, true);
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
        insert_expr_rule!(Didnt, true);
        insert_struct_rule!(DiscourseMarkers, true);
        insert_expr_rule!(DotInitialisms, true);
        insert_expr_rule!(DoubleClick, true);
        insert_expr_rule!(DoubleModal, true);
        insert_struct_rule!(EllipsisLength, true);
        insert_expr_rule!(ElsePossessive, true);
        insert_expr_rule!(EverEvery, true);
        insert_expr_rule!(Everyday, true);
        insert_expr_rule!(ExpandMemoryShorthands, true);
        insert_expr_rule!(ExpandTimeShorthands, true);
        insert_expr_rule!(FarBeIt, true);
        insert_expr_rule!(FascinatedBy, true);
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
        insert_expr_rule!(Hedging, true);
        insert_expr_rule!(HelloGreeting, true);
        insert_expr_rule!(Hereby, true);
        insert_struct_rule!(HopHope, true);
        insert_expr_rule!(HowTo, true);
        insert_expr_rule!(HyphenateNumberDay, true);
        insert_expr_rule!(IAmAgreement, true);
        insert_expr_rule!(IfWouldve, true);
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
        insert_struct_rule!(MergeWords, true);
        insert_expr_rule!(MissingPreposition, true);
        insert_expr_rule!(MissingTo, true);
        insert_expr_rule!(Misspell, true);
        insert_expr_rule!(MixedBag, true);
        insert_expr_rule!(ModalBeAdjective, true);
        insert_expr_rule!(ModalOf, true);
        insert_expr_rule!(ModalSeem, true);
        insert_expr_rule!(Months, true);
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
        insert_struct_rule!(NounVerbConfusion, true);
        insert_struct_rule!(NumberSuffixCapitalization, true);
        insert_expr_rule!(ObsessPreposition, true);
        insert_expr_rule!(OfCourse, true);
        insert_expr_rule!(OldestInTheBook, true);
        insert_expr_rule!(OnFloor, true);
        insert_expr_rule!(OnceOrTwice, true);
        insert_expr_rule!(OneAndTheSame, true);
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
        insert_expr_rule!(PossessiveYour, true);
        insert_expr_rule!(ProgressiveNeedsBe, true);
        insert_expr_rule!(PronounAre, true);
        insert_struct_rule!(PronounContraction, true);
        insert_expr_rule!(PronounInflectionBe, true);
        insert_expr_rule!(PronounKnew, true);
        insert_expr_rule!(QuantifierNeedsOf, true);
        insert_expr_rule!(QuantifierNumeralConflict, true);
        insert_expr_rule!(QuiteQuiet, true);
        insert_struct_rule!(QuoteSpacing, true);
        insert_expr_rule!(RedundantAcronyms, true);
        insert_expr_rule!(RedundantAdditiveAdverbs, true);
        insert_struct_rule!(RepeatedWords, true);
        insert_expr_rule!(Respond, true);
        insert_expr_rule!(RightClick, true);
        insert_expr_rule!(RollerSkated, true);
        insert_expr_rule!(SafeToSave, true);
        insert_expr_rule!(SaveToSafe, true);
        insert_expr_rule!(SemicolonApostrophe, true);
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
        insert_expr_rule!(TheProperNounPossessive, true);
        insert_expr_rule!(ThenThan, true);
        insert_expr_rule!(Theres, true);
        insert_expr_rule!(ThesesThese, true);
        insert_expr_rule!(ThingThink, true);
        insert_expr_rule!(ThisTypeOfThing, true);
        insert_expr_rule!(ThoughThought, true);
        insert_expr_rule!(ThrowAway, true);
        insert_struct_rule!(ThrowRubbish, true);
        insert_expr_rule!(ToAdverb, true);
        insert_struct_rule!(ToTwoToo, true);
        insert_expr_rule!(Touristic, true);
        insert_expr_rule!(TryOnesHandAt, true);
        insert_struct_rule!(UnclosedQuotes, true);
        insert_expr_rule!(UpdatePlaceNames, true);
        insert_expr_rule!(VerbToAdjective, true);
        insert_expr_rule!(VeryUnique, true);
        insert_expr_rule!(ViceVersa, true);
        insert_expr_rule!(ViciousCircle, true);
        insert_expr_rule!(ViciousCircleOrCycle, false);
        insert_expr_rule!(ViciousCycle, false);
        insert_expr_rule!(WasAloud, true);
        insert_expr_rule!(WayTooAdjective, true);
        insert_expr_rule!(WellEducated, true);
        insert_expr_rule!(Whereas, true);
        insert_expr_rule!(WhomSubjectOfVerb, true);
        insert_expr_rule!(WidelyAccepted, true);
        insert_expr_rule!(WinPrize, true);
        insert_expr_rule!(WishCould, true);
        insert_struct_rule!(WordPressDotcom, true);
        insert_expr_rule!(WouldNeverHave, true);

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
        out.config.set_rule_enabled("PossessiveNoun", false);

        out.add("Regionalisms", Regionalisms::new(dialect));
        out.config.set_rule_enabled("Regionalisms", true);

        out.add("HaveTakeALook", HaveTakeALook::new(dialect));
        out.config.set_rule_enabled("HaveTakeALook", true);

        out.add("MassNouns", MassNouns::new(dictionary.clone()));
        out.config.set_rule_enabled("MassNouns", true);

        out.add("UseTitleCase", UseTitleCase::new(dictionary.clone()));
        out.config.set_rule_enabled("UseTitleCase", true);

        out.add_chunk_expr_linter(
            "DisjointPrefixes",
            DisjointPrefixes::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("DisjointPrefixes", true);

        // add_chunk_expr_linter doesn't support the `Sentence` `Unit` and there is not yet any
        //  `add_sentence_expr_linter`
        out.add("Damages", Damages::default());
        out.config.set_rule_enabled("Damages", true);

        out.add(
            "PronounVerbAgreement",
            PronounVerbAgreement::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("PronounVerbAgreement", true);

        out.add_chunk_expr_linter("TransposedSpace", TransposedSpace::new(dictionary.clone()));
        out.config.set_rule_enabled("TransposedSpace", true);

        out.add_chunk_expr_linter(
            "OneOfTheSingular",
            OneOfTheSingular::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("OneOfTheSingular", true);

        out.add("AnA", AnA::new(dialect));
        out.config.set_rule_enabled("AnA", true);

        out.add("MoreAdjective", MoreAdjective::new(dictionary.clone()));
        out.config.set_rule_enabled("MoreAdjective", true);

        out.add("WorthToDo", WorthToDo::new(dictionary.clone()));
        out.config.set_rule_enabled("WorthToDo", true);

        out.add_chunk_expr_linter("DidPast", DidPast::new(dictionary.clone()));
        out.config.set_rule_enabled("DidPast", true);

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
        self.linters.extend(other_linters);

        let other_expr_linters = std::mem::take(&mut other.chunk_expr_linters);
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
    use crate::linting::LintKind;
    use crate::linting::tests::assert_no_lints;
    use crate::spell::{FstDictionary, MutableDictionary};
    use crate::{Dialect, Document, linting::Linter};

    fn test_group() -> LintGroup {
        LintGroup::new_curated(Arc::new(MutableDictionary::curated()), Dialect::American)
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
        let lints_to_check = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);

        let enforcer_config = LintGroupConfig::new_curated();
        let mut lints_to_enforce =
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American)
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
}
