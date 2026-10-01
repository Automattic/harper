//! German compound word checker using lazy decomposition.
//!
//! This module provides runtime compound word checking for German, which avoids
//! the O(n²) memory explosion of pre-generating all possible compound combinations.
//! Instead, it stores only the base words with their compound flags and checks
//! at lookup time whether a word can be decomposed into valid compound parts.
//!
//! Any dictionary word of at least [`MIN_COMPOUND_PART_LEN`] characters — or a
//! shorter one carrying compound-formation flags — may act as a compound
//! element, and every standard German interfix (`""`, `s`, `n`, `en`, `er`,
//! `es`) is attempted at each boundary. Membership is resolved against the
//! base dictionary when one is injected via [`CompoundChecker::set_base_dictionary`],
//! otherwise against a casing-tolerant set built from the word list.
//!
//! The one restriction on top of that is
//! [`can_head_a_lowercase_compound`], which is what keeps `vieleicht` from
//! decomposing.
//!
//! `GermanSpellCheck` has a second, weaker decomposition of its own. It shares
//! these constants so the two cannot disagree about what an element is, but it
//! is only reached when this one has already declined the word.
//!
//! Subproblems are memoized by `(segment, depth)`, turning the previously
//! exponential re-decomposition of long or misspelled words into a
//! polynomial-time scan.

use hashbrown::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::CharString;
use crate::dict_word_metadata::{AdjectiveData, DictWordMetadata, NounData};
use crate::language::morphology::{Agreement, Morphology, MorphologyExt};
use crate::spell::rune::word_list::AnnotatedWord;
use crate::spell::{Dictionary, FstDictionary};

/// Compound word formation flags for German
const COMPOUND_FLAG_NO_INTERFIX: char = 'h';
const COMPOUND_FLAG_S_INTERFIX: char = 'i';
const COMPOUND_FLAG_N_INTERFIX: char = 'k';
const COMPOUND_FLAG_EN_INTERFIX: char = 'l';
const COMPOUND_FLAG_ER_INTERFIX: char = 'm';
const COMPOUND_FLAG_ES_INTERFIX: char = 'o';
const COMPOUND_ADJ_FLAG: char = 'q';

/// Marks an entry that is a bare stem rather than a word (see the `stem_only`
/// property in `annotations.json`). The affix expansion keeps such an entry out
/// of the word list, so it has to be collected here separately.
const STEM_ONLY_FLAG: char = '*';

/// The flags that mark a closed-class word: determiner, pronoun, conjunction.
///
/// These are the collision-free digit flags from `annotations.json`. Their
/// letter counterparts (`D`, `I`, `C`, `P`) share the namespace with affix
/// letters and a bulk import spread them over several hundred ordinary nouns
/// and verbs (`augur/~~P`, `abbinden/~~I`), so they cannot be trusted for this
/// question.
const FUNCTION_WORD_FLAGS: [char; 3] = ['4', '5', '6'];

/// The flags that give an entry a word class of its own.
///
/// An entry carrying one of these is a content word, whatever else it also
/// carries — that is what keeps the handful of genuinely ambiguous entries out
/// of the function-word set.
const CONTENT_WORD_FLAGS: [char; 8] = ['N', 'M', 'F', 'Z', 'z', 'V', 'J', 'A'];

/// The feminine derivational suffixes that force the `-s-` interfix.
///
/// A noun built with one of these takes `-s-` in front of whatever follows it,
/// without exception: `Bildungssystem`, `Gesundheitsamt`, `Möglichkeitsform`,
/// `Gesellschaftsordnung`, `Revolutionsführer`, `Universitätsklinik`. See
/// [`suffixed_element_set`].
const S_INTERFIX_SUFFIXES: [&str; 6] = ["ung", "heit", "keit", "schaft", "ion", "tät"];

/// Is `element` one of the derivational suffixes itself?
///
/// The suffix takes `-s-` after it and nothing at all in front of it: German
/// writes `Bereitschaft`, `Möglichkeit`, `Schönheit`, never `*Bereitsschaft`
/// or `*Eigensschaft`. That is the other half of [`suffixed_element_set`], and
/// it is what stops a doubled `s` from hiding in the seam.
pub(crate) fn is_derivational_suffix(element: &[char]) -> bool {
    let spelling: String = element.iter().collect::<String>().to_lowercase();
    S_INTERFIX_SUFFIXES.contains(&spelling.as_str())
}

/// The adjective-forming suffixes that attach to a noun with no interfix at
/// all: `Wissenschaft` + `lich`, `Konformation` + `ell`, `Nation` + `al`.
///
/// They are the one exception to [`suffixed_element_set`]'s rule that such an
/// element may only be followed by `-s-`. That rule is about a *compound*
/// seam; a derivation has no seam, and blocking it reported
/// `neurowissenschaftliche` and `konformationelle` as misspellings.
const DERIVATIONAL_ADJECTIVE_SUFFIXES: [&str; 11] = [
    "lich", "ell", "al", "isch", "iv", "ös", "haft", "bar", "los", "sam", "är",
];

/// Does `rest` begin a derivation rather than a second compound element?
///
/// Only the suffix itself is matched, not its ending, because the derived
/// adjective is declined: `lich` covers `liche`, `lichen` and `licher` alike.
fn opens_a_derivation(rest: &[char]) -> bool {
    let spelling: String = rest.iter().collect::<String>().to_lowercase();
    DERIVATIONAL_ADJECTIVE_SUFFIXES
        .iter()
        .any(|suffix| spelling.starts_with(suffix))
}

/// Marks an entry as a feminine noun.
const FEMININE_FLAG: char = 'F';

/// The genitive endings. A German feminine noun has none, so an entry that
/// carries both this and [`FEMININE_FLAG`] is mislabelled -- see
/// [`suffixed_element_set`].
const GENITIVE_FLAGS: [char; 2] = ['0', 'H'];

/// All standard German linking interfixes, tried at every compound boundary.
const STANDARD_INTERFIXES: [&str; 6] = ["", "s", "n", "en", "er", "es"];

/// Whether `interfix` may join `element` to whatever follows it.
///
/// A German linking interfix never repeats the letter it follows. There is no
/// `*Hauss-`, no `*Bahnn-`, no `*Kindeses-`: the interfix marks a seam, and a
/// seam that doubles the letter in front of it is not one a German writer ever
/// produces. Leaving this unchecked is what lets the decomposition absorb a
/// doubled-consonant typo into the seam — `Aussbildung` passes as `aus` + `s` +
/// `bildung`, `hinnaus` as `hin` + `n` + `aus`, `annerkannt` as `an` + `n` +
/// `erkannt` — so the misspelling draws no lint at all.
///
/// The empty interfix is exempt, and has to be: `Schifffahrt` really is
/// `Schiff` plus `Fahrt`, seam and all. The rule is about the linking letter,
/// not about two elements that happen to meet on the same consonant.
pub(crate) fn interfix_fits(element: &[char], interfix: &[char]) -> bool {
    match (element.last(), interfix.first()) {
        (Some(last), Some(first)) => !last.eq_ignore_ascii_case(first),
        _ => true,
    }
}

/// The minimum length of a dictionary word that may act as a compound element.
///
/// Three is too permissive — it is what lets `Diskusion` through as `Diskus` +
/// `Ion` — but four is worse. Raising it costs several hundred false positives
/// on edited prose, because German builds just as freely on short *prefixes*
/// (`vor`, `aus`, `auf`, `neu`, `süd`) as on short nouns, and the decomposition
/// has no way to tell a prefix from a noun. Fixing this needs typed elements,
/// not a longer threshold; the frequent misspellings it lets through are caught
/// by Weir rules instead.
///
/// # Typing the elements from the word list does not work
///
/// Length is the whole gate, so `vieleicht` decomposes as `viel` + `eicht` —
/// `eicht` being the third person singular of *eichen*. Hunspell rejects the
/// same word without any blocklist entry, because its German dictionary marks
/// compound participation explicitly and positionally (`COMPOUNDBEGIN`,
/// `COMPOUNDMIDDLE`, `COMPOUNDEND`, `ONLYINCOMPOUND`) and only **3.0%** of its
/// 258 216 entries may take part at all. Here it is effectively every entry of
/// three characters or more.
///
/// The obvious repair is to ask what part of speech an element is. It was
/// measured three ways on 1.44M words of edited German prose, against the 1271
/// single-word entries of `Wikipedia:Liste von Tippfehlern`:
///
/// | element must … | extra typos caught | extra lints on correct prose |
/// |---|---|---|
/// | be a noun, adjective or adverb | — | **+14 006** |
/// | have any part of speech at all | +20 | **+1 576** |
/// | …or its lemma be nominal | +3 | **+148** |
///
/// All three lose, and for one reason: **9.4% of the 617 030 expanded forms
/// carry no part of speech at all**, and that bucket is not the verb forms. It
/// mixes `eicht`, `malt` and `agiert` with `zeuge`, `räume`, `garten` and
/// `folger`, which is how `Werkzeuge` and `Zeiträume` reach the dictionary at
/// all — `werkzeug/~~Nh` carries no plural flag and `säugetier/~~MhY` carries
/// the wrong one. The strongest rule fails earlier still: particles are
/// legitimate first elements and carry no nominal reading (`gegen`, `über`,
/// `vor`, `zusammen`), and ordinary nouns are tagged verb-only (`Tat`,
/// `arten`, `gen`, `erd`). Letter case carries nothing either, because
/// `WordId` lower-cases spellings.
///
/// What does work is [`can_head_a_lowercase_compound`], which asks a different
/// question in the one place the answer is decidable. Fixing the rest is a
/// dictionary question: the affix expansion has to give each generated form a
/// part of speech, and thousands of verbs are missing their conjugation flags
/// (`stattfinden/~~hc` cannot form `stattfindet`, and `denken` is not in the
/// word list at all).
pub(crate) const MIN_COMPOUND_PART_LEN: usize = 3;

/// The maximum nesting depth of a compound decomposition (mirrors the old
/// engine's `depth > 10` cap).
const MAX_COMPOUND_DEPTH: usize = 10;

/// Is the word written entirely in lower case?
pub(crate) fn lowercase(word: &[char]) -> bool {
    word.iter().all(|c| !c.is_uppercase())
}

/// May `tail` end the lower-case compound `whole`?
///
/// German compounds are right-headed: the last element decides what the whole
/// word is. A lower-case compound therefore ends in an adjective
/// (`umwelt|freundlich`), an adverb (`glücklicher|weise`) — or, when it is a
/// separable-prefix verb, in a finite verb form (`statt|findet`,
/// `zurück|geht`).
///
/// That last case is the problem. `viel` + `eicht` has exactly the shape of
/// `statt` + `findet`: a particle in front, a third person singular behind.
/// Nothing about the two *parts* tells them apart, and the word list does not
/// help — `eicht` and `findet` are both listed with no part of speech at all.
///
/// What tells them apart is whether the verb they would make exists.
/// `stattfinden` is a German verb and is in the dictionary; `vieleichen` is
/// not. So when the last element has no word class of its own, the compound is
/// accepted only if the prefix plus that element's **infinitive** is itself a
/// word — which is the question hunspell answers with per-entry compound
/// flags, asked the other way round.
pub(crate) fn can_head_a_lowercase_compound(
    dictionary: &impl Dictionary,
    whole: &[char],
    tail: &[char],
) -> bool {
    let Some(metadata) = dictionary.get_word_metadata(tail) else {
        return false;
    };

    // Any word class of its own is enough: an adjective or adverb head needs
    // no further argument, and a form the dictionary calls a verb was reached
    // through a verb's own paradigm.
    if metadata.noun.is_some()
        || metadata.adjective.is_some()
        || metadata.adverb.is_some()
        || metadata.verb.is_some()
        || metadata.pronoun.is_some()
        || metadata.conjunction.is_some()
        || metadata.determiner.is_some()
        || metadata.affix.is_some()
        || metadata.preposition
    {
        return true;
    }

    // No opinion at all. Reconstruct the verb and ask whether it exists.
    //
    // The recorded lemma is the best source, but roughly half of these forms
    // have none, so the infinitive is also rebuilt from the ending: German
    // makes it from the stem plus `-en`, or plus `-n` after an unstressed
    // `-e`. `schreitet` gives `schreiten`, `mittle` gives `mitteln`.
    let prefix = &whole[..whole.len() - tail.len()];
    let mut stem: Vec<char> = tail.to_vec();
    let without_t = tail.strip_suffix(&['t']).unwrap_or(tail).to_vec();

    let mut infinitives: Vec<Vec<char>> = Vec::with_capacity(3);
    if let Some(lemma) = metadata
        .derived_from
        .as_ref()
        .and_then(|id| dictionary.get_word_from_id(id))
    {
        infinitives.push(lemma.to_vec());
    }
    stem.push('n');
    infinitives.push(stem);
    infinitives.push([without_t.as_slice(), &['e', 'n']].concat());

    infinitives.into_iter().any(|infinitive| {
        let candidate: Vec<char> = [prefix, infinitive.as_slice()].concat();
        dictionary.contains_word(&candidate)
    })
}

/// Does the dictionary also read this string as a noun?
///
/// A string can be a function word *and* an inflected noun at the same time.
/// `falls` is the conjunction and the genitive of `der Fall`, and the word list
/// holds both (`falls/~~h6` and `fall/~~NhY0H`). [`function_word_set`] is built
/// from the base entries and cannot see the second reading, because that one
/// only exists after the affixes have been expanded. This asks the expanded
/// dictionary, which can, and so keeps `Einzelfalls` and `Rückfalls` words.
///
/// Only the noun reading counts. Admitting an adjective or verb reading as well
/// takes the exception far too wide — the declined determiners and possessives
/// all carry one — and it cost 42 caught misspellings (`alle`, `allen`,
/// `meine`, `deiner` back as compound heads) to save 18 lints.
pub(crate) fn has_content_reading(dictionary: &impl Dictionary, segment: &[char]) -> bool {
    dictionary
        .get_word_metadata(segment)
        .is_some_and(|metadata| metadata.noun.is_some())
}

/// Check if a character is a compound formation flag (case-insensitive)
fn is_compound_flag(c: char) -> bool {
    let lower_c = c.to_ascii_lowercase();
    matches!(
        lower_c,
        COMPOUND_FLAG_NO_INTERFIX
            | COMPOUND_FLAG_S_INTERFIX
            | COMPOUND_FLAG_N_INTERFIX
            | COMPOUND_FLAG_EN_INTERFIX
            | COMPOUND_FLAG_ER_INTERFIX
            | COMPOUND_FLAG_ES_INTERFIX
            | COMPOUND_ADJ_FLAG
    )
}

/// Compound checker that can determine if a word is a valid German compound
pub struct CompoundChecker {
    /// Words that participate in compounds, mapped to their compound flags.
    ///
    /// This only contains words that carry compound-formation flags; it drives
    /// the metadata distinction (adjective-vs-noun via the `q` flag) and the
    /// short-part allowance.
    compound_words: HashMap<CharString, HashSet<char>>,
    /// Every word from the word list in the casings needed for case-insensitive
    /// membership lookups when no base dictionary has been injected.
    members: HashSet<CharString>,
    /// The bare stems, which are elements but not words.
    ///
    /// `absperr` is not a German word, and the affix expansion leaves it out of
    /// the word list for that reason. It is still a legitimate piece of
    /// `Absperrband`, as `schreib` is of `Schreibtisch` and `erd` of
    /// `Erdgeschichte`: German builds compounds on the bare verb stem. What a
    /// stem may not be is the *head* — the head says what the whole word is,
    /// and no word is an `absperr`. Membership here is therefore consulted only
    /// in front of a boundary; see [`CompoundChecker::opening_element_usable`].
    stems: HashSet<CharString>,
    /// The articles, pronouns and conjunctions, which may open a compound but
    /// never end one. See [`function_word_set`].
    function_words: HashSet<CharString>,
    /// The derived feminine nouns, which open a compound only with `-s-`.
    /// See [`suffixed_element_set`].
    suffixed_elements: HashSet<CharString>,
    /// The base dictionary to resolve element membership against, when set.
    ///
    /// When present, membership queries use this dictionary (whose lookup is
    /// case-insensitive) instead of the `members` set.
    base_dict: Option<Arc<FstDictionary>>,
    /// All compound flags for quick lookup
    compound_flags: HashSet<char>,
    /// Cache for compound check results to avoid repeated decomposition
    cache: Mutex<LruCache<CharString, bool>>,
    /// Maximum time allowed for a single compound check to prevent combinatorial explosion
    max_check_time: Duration,
}

impl std::fmt::Debug for CompoundChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompoundChecker")
            .field("flagged_words", &self.compound_words.len())
            .field("members", &self.members.len())
            .field("stems", &self.stems.len())
            .field("function_words", &self.function_words.len())
            .field("suffixed_elements", &self.suffixed_elements.len())
            .field("has_base_dict", &self.base_dict.is_some())
            .field("compound_flags", &self.compound_flags)
            .field("max_check_time", &self.max_check_time)
            .finish()
    }
}

impl Clone for CompoundChecker {
    fn clone(&self) -> Self {
        Self {
            compound_words: self.compound_words.clone(),
            members: self.members.clone(),
            stems: self.stems.clone(),
            function_words: self.function_words.clone(),
            suffixed_elements: self.suffixed_elements.clone(),
            base_dict: self.base_dict.clone(),
            compound_flags: self.compound_flags.clone(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: self.max_check_time,
        }
    }
}

/// Insert `letters` (plus the first-letter-lowercased and first-letter-capitalized
/// variants) into `members` so case-insensitive lookups always hit.
fn insert_member_casings(members: &mut HashSet<CharString>, letters: &[char]) {
    if letters.is_empty() {
        return;
    }

    let exact: CharString = letters.iter().copied().collect();
    members.insert(exact.clone());

    let mut lowercased = exact.clone();
    if let Some(first_char) = lowercased.first_mut() {
        *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
    }
    members.insert(lowercased);

    let mut capitalized = exact.clone();
    if let Some(first_char) = capitalized.first_mut() {
        *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
    }
    members.insert(capitalized);
}

/// The bare stems of a word list: compound elements that are not words.
///
/// `GermanSpellCheck` has a decomposition of its own and only a [`Dictionary`]
/// to consult, so it reads the same set from here rather than keeping a second
/// idea of what a stem is.
pub(crate) fn stem_set(word_list: &[AnnotatedWord]) -> HashSet<CharString> {
    let mut stems = HashSet::new();
    for word in word_list {
        if word.annotations.contains(&STEM_ONLY_FLAG) {
            insert_member_casings(&mut stems, &word.letters);
        }
    }
    stems
}

/// The function words of a word list: articles, pronouns and conjunctions.
///
/// German builds compounds out of content words, and the last element is the
/// one that says what the whole word is. A function word says nothing of the
/// kind, so it never ends a compound: there is no word that ends in the
/// article `den`, in `der`, in `alle` or in the pronoun `er`. Leaving this
/// unchecked is what lets a doubled-letter typo split off a function word and
/// escape unnoticed — `Badden` passes as `bad` + `den`, `Bildder` as `bild` +
/// `der`, `Bewusstseinns` as `bewusst` + `sein` + `ns`.
///
/// The opening position stays open, and has to: `Ausbildung`, `Anfang` and
/// `Nachteil` all begin with one.
///
/// Like [`stem_set`], this is read by `GermanSpellCheck` too, so the two
/// decompositions cannot disagree about what a function word is.
pub(crate) fn function_word_set(word_list: &[AnnotatedWord]) -> HashSet<CharString> {
    let mut function_words = HashSet::new();
    for word in word_list {
        let is_function = word
            .annotations
            .iter()
            .any(|flag| FUNCTION_WORD_FLAGS.contains(flag));
        let is_content = word
            .annotations
            .iter()
            .any(|flag| CONTENT_WORD_FLAGS.contains(flag));
        if is_function && !is_content {
            insert_member_casings(&mut function_words, &word.letters);
        }
    }
    function_words
}

/// The elements that may only be followed by the `-s-` interfix.
///
/// German derives feminine nouns with `-ung`, `-heit`, `-keit`, `-schaft`,
/// `-ion` and `-tät`, and every one of them takes `-s-` when it opens a
/// compound. Trying the other interfixes there is what lets a doubled-letter
/// typo through: `Abbildunggen` decomposes as `abbildung` + `gen`,
/// `Bereitsschaft` as `bereit` + `-s-` + `schaft`, `Eigensschaft` as `ei` +
/// `gen` + `-s-` + `schaft`.
///
/// **The ending alone is not enough.** `Sprung`, `Schwung`, `Ursprung`, `Ion`
/// and `Schaft` end the same way without being derivations, and they take no
/// interfix at all (`Sprungbrett`, `Ursprungsland` is the exception that
/// proves nothing). The word list marks them feminine anyway
/// (`sprung/~~Fh0H`), so the flag on its own would catch them too.
///
/// What separates them is a second flag on the same entry: a German feminine
/// noun has no genitive ending, so an entry that is marked feminine *and*
/// carries a genitive is not feminine. That test takes 81 mislabelled entries
/// out of 8768 and needs no list of exceptions.
pub(crate) fn suffixed_element_set(word_list: &[AnnotatedWord]) -> HashSet<CharString> {
    let mut suffixed = HashSet::new();
    for word in word_list {
        let spelling: String = word.letters.iter().collect::<String>().to_lowercase();
        if !S_INTERFIX_SUFFIXES
            .iter()
            .any(|suffix| spelling.ends_with(suffix))
        {
            continue;
        }
        if !word.annotations.contains(&FEMININE_FLAG)
            || word
                .annotations
                .iter()
                .any(|flag| GENITIVE_FLAGS.contains(flag))
        {
            continue;
        }
        insert_member_casings(&mut suffixed, &word.letters);
    }
    suffixed
}

impl CompoundChecker {
    /// Create a new CompoundChecker from a list of annotated words
    pub fn new(word_list: &[AnnotatedWord]) -> Self {
        // Build compound words map from words that have compound flags
        let mut compound_words = HashMap::new();
        let mut members = HashSet::new();
        let stems = stem_set(word_list);
        let function_words = function_word_set(word_list);
        let suffixed_elements = suffixed_element_set(word_list);

        for word in word_list {
            let flags: HashSet<char> = word
                .annotations
                .iter()
                .filter(|&&c| is_compound_flag(c))
                .map(|&c| c.to_ascii_lowercase())
                .collect();

            // Every dictionary word may act as a compound element, regardless of
            // whether it carries compound-formation flags. Record the casings
            // needed for case-insensitive membership lookups.
            insert_member_casings(&mut members, &word.letters);

            if !flags.is_empty() {
                // Insert the original word
                compound_words.insert(word.letters.clone(), flags.clone());

                // Also insert a version with the first letter lowercased for case-insensitive lookup
                if !word.letters.is_empty() {
                    let mut lowercased_first = word.letters.clone();
                    if let Some(first_char) = lowercased_first.first_mut() {
                        // Use proper Unicode lowercasing for German characters like Ä, Ö, Ü
                        *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
                    }
                    compound_words.insert(lowercased_first, flags);
                }
            }
        }

        Self {
            compound_words,
            members,
            stems,
            function_words,
            suffixed_elements,
            base_dict: None,
            compound_flags: ['h', 'i', 'k', 'l', 'm', 'o', 'q']
                .iter()
                .copied()
                .collect(),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(10000).unwrap())),
            max_check_time: Duration::from_millis(5000), // 5 second timeout
        }
    }

    /// Inject the base dictionary whose word membership drives compound
    /// decomposition. When set, element membership is resolved through
    /// `base.contains_word` (a case-insensitive lookup) instead of the word
    /// list collected by [`CompoundChecker::new`].
    pub fn set_base_dictionary(&mut self, base: Arc<FstDictionary>) {
        self.base_dict = Some(base);
        // Membership queries now go through the base dictionary, so the
        // casing-tolerant word-list set is no longer needed.
        self.members.clear();
        // Membership semantics changed, so cached decomposition results are stale.
        self.cache.lock().unwrap().clear();
    }

    /// Whether `word` is a member of the underlying dictionary.
    fn member_of(&self, word: &[char]) -> bool {
        match &self.base_dict {
            Some(base_dict) => base_dict.contains_word(word),
            None => self.members.contains(word),
        }
    }

    /// See the free function of the same name. Without a base dictionary there
    /// is no metadata to consult; that path is only used by tests.
    fn can_head_a_lowercase_compound(&self, whole: &[char], tail: &[char]) -> bool {
        match self.base_dict.as_ref() {
            Some(base) => can_head_a_lowercase_compound(base.as_ref(), whole, tail),
            None => true,
        }
    }

    /// Whether `word` carries compound-formation flags in the word list.
    fn has_compound_flags(&self, word: &[char]) -> bool {
        self.get_compound_flags(word).is_some()
    }

    /// Whether a segment can participate in a compound as an element.
    ///
    /// A segment is usable iff it is a dictionary word AND it is either at
    /// least [`MIN_COMPOUND_PART_LEN`] characters long or one of the handful of
    /// two-letter words German really does build on.
    fn element_usable(&self, segment: &[char]) -> bool {
        self.member_of(segment)
            && (segment.len() >= MIN_COMPOUND_PART_LEN || self.short_element_usable(segment))
    }

    /// May a word shorter than [`MIN_COMPOUND_PART_LEN`] be a compound element?
    ///
    /// Only if it is a noun or a preposition. German does build on two-letter
    /// words, but on a closed handful of them: the nouns `Ei` and `Öl`
    /// (`Eigelb`, `Ölgemälde`, `Rohöl`) and the particles `ab`, `an`, `um`,
    /// `zu`, `im` (`Abbau`, `Umbau`). Everything else two letters long in the
    /// word list is an interjection, an abbreviation or a fragment — `ah`,
    /// `kt`, `mm`, `hl`, `äh` — and admitting those is how `Abau`, `Adahm` and
    /// `Absiecht` pass as compounds.
    ///
    /// This used to key on the compound flags instead, which was no gate at
    /// all: `h` is the bookkeeping marker that sits on nearly every entry. On
    /// 1.44M words of prose, asking for a noun or a preposition catches 235
    /// more misspellings and newly reports 670 words, of which 488 are ones
    /// hunspell rejects as well (proper names and quoted English) and the rest
    /// are almost all names too.
    ///
    /// Without a base dictionary there is no metadata to ask, and the flags are
    /// all that is left; that path is only used by tests.
    fn short_element_usable(&self, segment: &[char]) -> bool {
        match self.base_dict.as_ref() {
            Some(base) => base
                .get_word_metadata(segment)
                .is_some_and(|metadata| metadata.noun.is_some() || metadata.preposition),
            None => self.has_compound_flags(segment),
        }
    }

    /// Whether a segment can stand *in front of* a compound boundary.
    ///
    /// Wider than [`CompoundChecker::element_usable`] by the bare stems, which
    /// are compound elements without being words. `Erdgeschichte`,
    /// `Schreibtisch` and `Absperrband` all open on one. They are excluded from
    /// the head position, where the word class of the whole compound is
    /// decided, because a stem has none.
    fn opening_element_usable(&self, segment: &[char]) -> bool {
        self.element_usable(segment)
            || (segment.len() >= MIN_COMPOUND_PART_LEN && self.stems.contains(segment))
    }

    /// Whether a segment can stand *at the end of* a compound.
    ///
    /// The mirror image of [`CompoundChecker::opening_element_usable`]: a stem
    /// is excluded from the head position because it has no word class, and a
    /// function word because the class it has is not one a compound can be.
    /// See [`function_word_set`].
    fn closing_element_usable(&self, segment: &[char]) -> bool {
        self.element_usable(segment) && !self.is_function_word_only(segment)
    }

    /// Is this string a function word and nothing else? See
    /// [`has_content_reading`], which is what rescues the strings that are
    /// both.
    fn is_function_word_only(&self, segment: &[char]) -> bool {
        self.function_words.contains(segment)
            && !self
                .base_dict
                .as_ref()
                .is_some_and(|base| has_content_reading(base.as_ref(), segment))
    }

    /// Helper to get compound flags for a word, trying lowercase first if not found
    fn get_compound_flags(&self, word: &[char]) -> Option<&HashSet<char>> {
        // First try exact match
        if let Some(flags) = self.compound_words.get(word) {
            return Some(flags);
        }

        // For German, try with first letter lowercased (nouns are capitalized in text but stored lowercase in dict)
        if !word.is_empty() {
            let mut lowercased: CharString = word.iter().copied().collect();
            if let Some(first_char) = lowercased.first_mut() {
                // Use proper Unicode lowercasing for German characters like Ä, Ö, Ü
                *first_char = first_char.to_lowercase().next().unwrap_or(*first_char);
            }
            self.compound_words.get(&lowercased)
        } else {
            None
        }
    }

    /// Check if a word is a valid German compound word
    pub fn is_compound_word(&self, word: &[char]) -> bool {
        let word_chars = CharString::from(word);

        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(&result) = cache.get(&word_chars) {
                return result;
            }
        }

        // A word that is itself a plain dictionary element is not a compound.
        let result = if word.is_empty() || self.element_usable(word) {
            false
        } else {
            let start = Instant::now();
            let mut memo = HashMap::new();
            self.is_valid_segment(word, 0, &start, &mut memo, lowercase(word).then_some(word))
        };

        // Cache result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(word_chars, result);
        }

        result
    }

    /// Recursively check whether `segment` decomposes into usable compound
    /// elements joined by standard German interfixes.
    ///
    /// The whole `segment` is accepted as an element only below the top level
    /// (`depth > 0`); at the top level a plain dictionary word must first be
    /// rejected by the `element_usable` guard in [`CompoundChecker::is_compound_word`].
    /// Results are memoized by `(segment, depth)` so that overlapping
    /// subproblems are evaluated at most once per top-level check.
    fn is_valid_segment(
        &self,
        segment: &[char],
        depth: usize,
        start: &Instant,
        memo: &mut HashMap<(Vec<char>, usize), bool>,
        lowercase_whole: Option<&[char]>,
    ) -> bool {
        if start.elapsed() > self.max_check_time {
            return false; // Timeout exceeded
        }

        if depth > MAX_COMPOUND_DEPTH {
            return false;
        }

        // Below the top level, the whole sub-segment may itself be an element.
        // This precedes the minimum-length guard so that short elements carrying
        // compound flags (e.g. `ei`, `öl`) are accepted at any position.
        //
        // Reaching here means the whole remaining tail is one element, so this
        // is the compound's last element — the one that decides what the word
        // is. In a lower-case compound that decision is constrained.
        if depth > 0 && self.closing_element_usable(segment) {
            return match lowercase_whole {
                Some(whole) => self.can_head_a_lowercase_compound(whole, segment),
                None => true,
            };
        }

        if segment.len() < MIN_COMPOUND_PART_LEN {
            return false;
        }

        if let Some(&cached) = memo.get(&(segment.to_vec(), depth)) {
            return cached;
        }

        let mut valid = false;
        for split_pos in 1..segment.len() {
            let (first, rest) = segment.split_at(split_pos);

            // The left part must itself be a usable element. A bare stem
            // counts here and nowhere else.
            if !self.opening_element_usable(first) {
                continue;
            }

            // Try every standard interfix at this boundary.
            let only_s = self.suffixed_elements.contains(first) && !opens_a_derivation(rest);
            for interfix in STANDARD_INTERFIXES {
                if only_s && interfix != "s" {
                    continue;
                }
                let interfix_chars: Vec<char> = interfix.chars().collect();
                if !interfix_fits(first, &interfix_chars) {
                    continue;
                }
                let Some(after) = rest.strip_prefix(interfix_chars.as_slice()) else {
                    continue;
                };
                if !interfix.is_empty() && is_derivational_suffix(after) {
                    continue;
                }

                if self.is_valid_segment(after, depth + 1, start, memo, lowercase_whole) {
                    valid = true;
                    break;
                }
            }

            if valid {
                break;
            }
        }

        memo.insert((segment.to_vec(), depth), valid);
        valid
    }

    /// Get metadata for a compound word (for use in dictionary lookups)
    ///
    /// A German *Determinativkompositum* is **right-headed**: the last element
    /// decides the word class, and everything in front of it only modifies.
    /// `Stickstoff` + `tolerant` is an adjective, `Haus` + `Tür` a noun.
    pub fn get_compound_metadata(&self, word: &[char]) -> Option<DictWordMetadata> {
        if !self.is_compound_word(word) {
            return None;
        }

        // The head decides. This used to ask whether the *first* element was an
        // adjective, which is the opposite question: "stickstofftolerant" and
        // "galleresistent" came back nouns, and `GermanNounCapitalization` saw
        // an unambiguous noun reading and reported them as miscapitalized.
        if self.head_is_adjective(word) || self.opens_with_adjective(word) {
            return Some(DictWordMetadata {
                adjective: Some(AdjectiveData::default()),
                ..Default::default()
            });
        }

        // Default to noun metadata for other compounds (most German compounds are nouns)
        //
        // The head decides the features too, not just the word class: a
        // *Hausschlüssel* is masculine because a *Schlüssel* is, and a *Haustür*
        // feminine because a *Tür* is. Taking the head's agreement here gives a
        // gender to compounds that have no dictionary entry at all, which is
        // most of them — German builds compounds faster than any word list can
        // record them.
        let agreement = self.head_noun_agreement(word);
        Some(DictWordMetadata {
            noun: Some(NounData::default()),
            morphology: (!agreement.is_unknown()).then(|| Morphology {
                noun: Some(agreement),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// The agreement features of the compound's head.
    ///
    /// Finds the head the same way [`CompoundChecker::head_is_adjective`] does,
    /// so the two cannot disagree about where the word splits: walk the split
    /// points from the left, take the first tail that is a usable element's
    /// remainder and a known noun. Empty when there is no such split, or when
    /// the head itself carries no features.
    ///
    /// The **number** is deliberately dropped. *Schlüssel* is a singular, but
    /// *Hausschlüsseln* is the dative plural of the compound and the head looks
    /// exactly the same; only the compound's own ending says which, and that is
    /// not what this reads. Gender survives because it is a property of the
    /// lexeme rather than of the form.
    fn head_noun_agreement(&self, word: &[char]) -> Agreement {
        let Some(base) = self.base_dict.as_ref() else {
            return Agreement::default();
        };

        for split_pos in 1..word.len() {
            let (first, rest) = word.split_at(split_pos);

            if rest.len() < MIN_COMPOUND_PART_LEN {
                break;
            }
            if !self.element_usable(first) {
                continue;
            }

            if let Some(metadata) = base.get_word_metadata(rest)
                && metadata.noun.is_some()
            {
                return Agreement {
                    gender: metadata.noun_agreement().gender,
                    ..Default::default()
                };
            }
        }

        Agreement::default()
    }

    /// Is the compound's head — its **last** element — an adjective?
    ///
    /// Walks the split points from the left, so the first hit is the longest
    /// tail that is both a known adjective and preceded by a usable compound
    /// element. The element test is the one [`CompoundChecker::collect_parts`]
    /// uses, so a head found here belongs to a decomposition the checker would
    /// actually accept.
    ///
    /// A word that is *also* a noun is left alone: `-mann`, `-teil`, `-recht`
    /// and friends carry an adjective reading in this dictionary, and treating
    /// `Bürgerrecht` as an adjective would cost far more than the compound
    /// adjectives gain.
    fn head_is_adjective(&self, word: &[char]) -> bool {
        let Some(base) = self.base_dict.as_ref() else {
            return false;
        };

        for split_pos in 1..word.len() {
            let (first, rest) = word.split_at(split_pos);

            if rest.len() < MIN_COMPOUND_PART_LEN {
                break;
            }
            if !self.element_usable(first) {
                continue;
            }

            if let Some(metadata) = base.get_word_metadata(rest)
                && metadata.adjective.is_some()
                && metadata.noun.is_none()
            {
                return true;
            }
        }

        false
    }

    /// Does the compound start with an adjective?
    ///
    /// The head decides the word class, but when the head is a noun/adjective
    /// homograph — `braun`, `recht`, `mal` — an adjective in front of it settles
    /// which reading is meant: `purpur` + `braun` is a colour adjective,
    /// `Bürger` + `recht` a noun. Kept as the second question precisely because
    /// on its own it answers the wrong one.
    fn opens_with_adjective(&self, word: &[char]) -> bool {
        for split_pos in 1..word.len() {
            let (first, _rest) = word.split_at(split_pos);

            if let Some(first_flags) = self.get_compound_flags(first)
                && first_flags.contains(&COMPOUND_ADJ_FLAG)
            {
                return true;
            }
        }

        false
    }

    /// Check if a word can be decomposed and return the decomposition parts
    pub fn get_decomposition(&self, word: &[char]) -> Option<Vec<String>> {
        if word.is_empty() {
            return None;
        }

        let start = Instant::now();
        let mut memo = HashMap::new();
        self.collect_parts(word, 0, &start, &mut memo)
    }

    /// Collect the parts of the first successful decomposition of `segment`.
    ///
    /// Uses the same `element_usable` predicate and interfix probing as
    /// [`CompoundChecker::is_valid_segment`]. The interfix string is emitted as
    /// its own part (e.g. `arbeitsgeber` yields `["arbeit", "s", "geber"]`).
    /// Failed suffixes are memoized to avoid re-exploration.
    fn collect_parts(
        &self,
        segment: &[char],
        depth: usize,
        start: &Instant,
        memo: &mut HashMap<(Vec<char>, usize), Option<Vec<String>>>,
    ) -> Option<Vec<String>> {
        if start.elapsed() > self.max_check_time {
            return None;
        }

        if depth > MAX_COMPOUND_DEPTH {
            return None;
        }

        // Mirror is_valid_segment: accept short flagged elements below the top
        // level before applying the minimum-length guard.
        if depth > 0 && self.element_usable(segment) {
            return Some(vec![segment.iter().collect()]);
        }

        if segment.len() < MIN_COMPOUND_PART_LEN {
            return None;
        }

        if let Some(cached) = memo.get(&(segment.to_vec(), depth)) {
            return cached.clone();
        }

        let mut result = None;
        'search: for split_pos in 1..segment.len() {
            let (first, rest) = segment.split_at(split_pos);

            if !self.element_usable(first) {
                continue;
            }

            let only_s = self.suffixed_elements.contains(first) && !opens_a_derivation(rest);
            for interfix in STANDARD_INTERFIXES {
                if only_s && interfix != "s" {
                    continue;
                }
                let interfix_chars: Vec<char> = interfix.chars().collect();
                if !interfix_fits(first, &interfix_chars) {
                    continue;
                }
                let Some(after) = rest.strip_prefix(interfix_chars.as_slice()) else {
                    continue;
                };
                if !interfix.is_empty() && is_derivational_suffix(after) {
                    continue;
                }

                if let Some(mut tail) = self.collect_parts(after, depth + 1, start, memo) {
                    let mut parts = vec![first.iter().collect()];
                    if !interfix.is_empty() {
                        parts.push(interfix.to_string());
                    }
                    parts.append(&mut tail);
                    result = Some(parts);
                    break 'search;
                }
            }
        }

        memo.insert((segment.to_vec(), depth), result.clone());
        result
    }

    /// Get the number of compound-eligible words
    pub fn compound_word_count(&self) -> usize {
        self.compound_words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spell::rune::word_list::AnnotatedWord;

    fn create_test_checker() -> CompoundChecker {
        let words = vec![
            // Basic words with compound flags
            AnnotatedWord {
                letters: "schuh".chars().collect(),
                annotations: vec!['N', 'X', 'h'],
            },
            AnnotatedWord {
                letters: "hersteller".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "bildung".chars().collect(),
                annotations: vec!['N', 'i'], // s interfix (corrected from 'l' which is for "en" interfix)
            },
            AnnotatedWord {
                letters: "ministerium".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            // Adjective compound word
            AnnotatedWord {
                letters: "rot".chars().collect(),
                annotations: vec!['A', 'q'], // adjective with q flag
            },
            AnnotatedWord {
                letters: "haar".chars().collect(),
                annotations: vec!['N', 'q'], // noun with q flag for adjective compounds
            },
        ];

        CompoundChecker::new(&words)
    }

    #[test]
    fn test_simple_noun_compound_no_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_noun_compound_with_s_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_noun_compound_with_en_interfix() {
        let checker = create_test_checker();
        assert!(checker.is_compound_word(&"bildungsministerium".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_adjective_compound() {
        let checker = create_test_checker();
        // rothaar (red-haired) - adjective + noun with q flag
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_non_compound_word() {
        let checker = create_test_checker();
        // "xyzabc" should not be recognized as a compound
        assert!(!checker.is_compound_word(&"xyzabc".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_single_word_not_compound() {
        let checker = create_test_checker();
        // Single words from the dictionary should not be compounds
        assert!(!checker.is_compound_word(&"schuh".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"arbeit".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_empty_word() {
        let checker = create_test_checker();
        assert!(!checker.is_compound_word(&[]));
    }

    #[test]
    fn test_decomposition_parts() {
        let checker = create_test_checker();
        let parts = checker.get_decomposition(&"schuhhersteller".chars().collect::<Vec<_>>());
        assert!(parts.is_some());
        let parts = parts.unwrap();
        // With recursive compounding, we might get more granular parts
        // The key is that schuh and hersteller should be present (either as parts or combined)
        let parts_str: String = parts.join("");
        assert!(parts_str.contains("schuh"));
        assert!(parts_str.contains("hersteller"));
    }

    #[test]
    fn test_decomposition_with_interfix() {
        let checker = create_test_checker();
        let parts = checker.get_decomposition(&"arbeitsgeber".chars().collect::<Vec<_>>());
        assert!(parts.is_some());
        let parts = parts.unwrap();
        assert!(parts.contains(&"arbeit".to_string()));
        assert!(parts.contains(&"s".to_string()));
        assert!(parts.contains(&"geber".to_string()));
    }

    #[test]
    fn test_compound_word_count() {
        let checker = create_test_checker();
        assert!(checker.compound_word_count() > 0);
    }

    #[test]
    fn test_cache_works() {
        let checker = create_test_checker();
        let word: Vec<char> = "schuhhersteller".chars().collect();

        // First check should populate cache
        let result1 = checker.is_compound_word(&word);

        // Second check should use cache
        let result2 = checker.is_compound_word(&word);

        // Results should be the same
        assert_eq!(result1, result2);
        assert!(result1);
    }

    #[test]
    fn test_get_compound_metadata() {
        let checker = create_test_checker();
        let word: Vec<char> = "schuhhersteller".chars().collect();

        let metadata = checker.get_compound_metadata(&word);
        assert!(metadata.is_some());
        let metadata = metadata.unwrap();
        assert!(metadata.noun.is_some());
    }

    // ==================== RECURSIVE COMPOUNDING TESTS ====================

    #[test]
    fn test_recursive_compounding_basic() {
        // Test the classic example: dampfschiff -> donaudampfschiff
        let words = vec![
            AnnotatedWord {
                letters: "donau".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "dampf".chars().collect(),
                annotations: vec!['M', 'h'],
            },
            AnnotatedWord {
                letters: "schiff".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // dampfschiff should be recognized as compound
        assert!(checker.is_compound_word(&"dampfschiff".chars().collect::<Vec<_>>()));

        // donaudampfschiff should be recognized as compound (recursive)
        assert!(checker.is_compound_word(&"donaudampfschiff".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_recursive_compounding_deep() {
        // Realistic deep-nesting fixture: every element is a full-length German-
        // looking word (no single-character elements), and a 4-5 level compound
        // must decompose recursively.
        let words = vec![
            AnnotatedWord {
                letters: "dampf".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "schiff".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "fahrt".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "kapitän".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "gesellschaft".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // 2-element compound
        assert!(checker.is_compound_word(&"dampfschiff".chars().collect::<Vec<_>>()));

        // 3-element compound
        assert!(checker.is_compound_word(&"dampfschifffahrt".chars().collect::<Vec<_>>()));

        // 4-element compound with an s-interfix
        assert!(checker.is_compound_word(&"dampfschifffahrtskapitän".chars().collect::<Vec<_>>()));

        // 5-element compound with an s-interfix
        assert!(
            checker.is_compound_word(&"dampfschifffahrtsgesellschaft".chars().collect::<Vec<_>>())
        );

        // Single fixture words are elements, not compounds.
        for word in ["dampf", "schiff", "fahrt", "kapitän", "gesellschaft"] {
            assert!(
                !checker.is_compound_word(&word.chars().collect::<Vec<_>>()),
                "{word}"
            );
        }
    }

    #[test]
    fn test_recursive_compounding_with_interfix() {
        let words = vec![
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['F', 'i'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['M', 'h'],
            },
            AnnotatedWord {
                letters: "fach".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // arbeitsgeber should be recognized (with -s interfix)
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));

        // arbeitsgeberfach should be recognized (recursive)
        assert!(checker.is_compound_word(&"arbeitsgeberfach".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_recursive_compounding_with_adjective() {
        let words = vec![
            AnnotatedWord {
                letters: "rot".chars().collect(),
                annotations: vec!['A', 'q'],
            },
            AnnotatedWord {
                letters: "haar".chars().collect(),
                annotations: vec!['N', 'q'],
            },
            AnnotatedWord {
                letters: "farbe".chars().collect(),
                annotations: vec!['F', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // rothaar should work (adjective compound)
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));

        // rothaarfarbe should work recursively
        assert!(checker.is_compound_word(&"rothaarfarbe".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_existing_compounds_still_work() {
        let checker = create_test_checker();

        // All existing test cases should still pass
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"bildungsministerium".chars().collect::<Vec<_>>()));
        assert!(checker.is_compound_word(&"rothaar".chars().collect::<Vec<_>>()));

        // Non-compounds should still fail
        assert!(!checker.is_compound_word(&"xyzabc".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"schuh".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_timeout_protection() {
        // Fixture where every single letter is a usable (flagged) element, so a
        // word like "aaaa..." is decomposable in principle. The depth cap (and
        // the requirement that the final element be at least MIN_COMPOUND_PART_LEN
        // characters) keeps it from ever succeeding. With subproblem memoization
        // the check completes quickly instead of re-decomposing exponentially;
        // the 1ms timeout guard is never the deciding factor, but must not
        // change the outcome either.
        let words: Vec<AnnotatedWord> = (b'a'..=b'z')
            .map(|c| AnnotatedWord {
                letters: vec![c as char].into(),
                annotations: vec!['N', 'h'],
            })
            .collect();

        let mut checker = CompoundChecker::new(&words);
        checker.max_check_time = Duration::from_millis(1); // 1ms timeout

        // Very long word that would cause combinatorial explosion without memoization
        let long_word: Vec<char> = "a".repeat(100).chars().collect();

        // Should return false quickly, not hang
        let start = Instant::now();
        let result = checker.is_compound_word(&long_word);
        let elapsed = start.elapsed();

        assert!(!result);
        assert!(elapsed < Duration::from_millis(100)); // Should complete quickly
    }

    // ==================== PRODUCTIVITY TESTS ====================

    #[test]
    fn test_head_without_compound_flags() {
        // Core fix: a compound is accepted even when its head (final) element
        // carries no compound-formation flags, as long as it is a dictionary word.
        let words = vec![
            AnnotatedWord {
                letters: "haus".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "tor".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // "haustor" is haus + tor, and "tor" has no compound flags at all.
        assert!(checker.is_compound_word(&"haustor".chars().collect::<Vec<_>>()));

        // Plain words are elements, not compounds.
        assert!(!checker.is_compound_word(&"haus".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"tor".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_compound_with_no_flags_anywhere() {
        // All parts are in the word list but none carries compound flags.
        let words = vec![
            AnnotatedWord {
                letters: "see".chars().collect(),
                annotations: vec!['N'],
            },
            AnnotatedWord {
                letters: "karte".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"seekarte".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"see".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"karte".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_s_interfix_without_i_flag() {
        // The old engine only allowed an s-interfix when the first element
        // carried the 'i' flag. Any first element may now combine with an
        // s-interfix if the result decomposes into dictionary words.
        let words = vec![
            AnnotatedWord {
                letters: "arbeit".chars().collect(),
                annotations: vec!['N'],
            },
            AnnotatedWord {
                letters: "geber".chars().collect(),
                annotations: vec!['N'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"arbeitsgeber".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_short_flagged_element() {
        // Two-character real words (like "ei") remain usable compound elements
        // because they carry compound-formation flags.
        let words = vec![
            AnnotatedWord {
                letters: "ei".chars().collect(),
                annotations: vec!['N', 'h', 'i'],
            },
            AnnotatedWord {
                letters: "schnee".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        assert!(checker.is_compound_word(&"eischnee".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"ei".chars().collect::<Vec<_>>()));
        assert!(!checker.is_compound_word(&"schnee".chars().collect::<Vec<_>>()));
    }

    #[test]
    fn test_memoization_determinism_and_junk() {
        let words = vec![
            AnnotatedWord {
                letters: "see".chars().collect(),
                annotations: vec!['N', 'h'],
            },
            AnnotatedWord {
                letters: "karte".chars().collect(),
                annotations: vec!['N', 'h'],
            },
        ];

        let checker = CompoundChecker::new(&words);

        // Repeated checks must return the same result (top-level cache + memo).
        let valid: Vec<char> = "seekarte".chars().collect();
        assert_eq!(
            checker.is_compound_word(&valid),
            checker.is_compound_word(&valid)
        );
        assert!(checker.is_compound_word(&valid));

        // A concatenation of two real words with a spurious inserted letter is
        // not decomposable into exactly the dictionary elements and must stay
        // rejected, deterministically.
        let junk: Vec<char> = "seeekarte".chars().collect();
        assert_eq!(
            checker.is_compound_word(&junk),
            checker.is_compound_word(&junk)
        );
        assert!(!checker.is_compound_word(&junk));
    }

    #[test]
    fn test_base_dictionary_membership() {
        use crate::spell::MutableDictionary;

        // The word list only knows "schuh" (flagged). "hersteller" is provided
        // through the injected base dictionary, as happens with the real German
        // base dictionary.
        let words = vec![AnnotatedWord {
            letters: "schuh".chars().collect(),
            annotations: vec!['N', 'h'],
        }];

        let mut checker = CompoundChecker::new(&words);

        // Without a base dictionary, "hersteller" is not a member and the
        // compound cannot be decomposed.
        assert!(!checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));

        // Both are nouns, and the base dictionary has to say so: a lower-case
        // compound may only end in a word that has a word class.
        let noun = DictWordMetadata {
            noun: Some(Default::default()),
            ..Default::default()
        };
        let mut base = MutableDictionary::new();
        base.append_word("schuh".chars().collect::<CharString>(), noun.clone());
        base.append_word("hersteller".chars().collect::<CharString>(), noun);
        checker.set_base_dictionary(Arc::new(base.into()));

        // With the base dictionary injected, "schuh" + "hersteller" resolves.
        assert!(checker.is_compound_word(&"schuhhersteller".chars().collect::<Vec<_>>()));
    }

    /// The element gate cannot be typed from the word list, and this is the
    /// measurement that says so — see [`MIN_COMPOUND_PART_LEN`].
    ///
    /// `eicht` is a finite verb form and has no business inside a compound;
    /// `räume` is a noun plural and belongs in every second one. The
    /// dictionary describes both identically: present, three or more
    /// characters, and carrying no part of speech at all. As long as that
    /// holds, `vieleicht` cannot be rejected by decomposition without taking
    /// `Zeiträume` with it.
    ///
    /// `zeuge` used to be on this list and is not any more:
    /// `fix_german_noun_forms.py` gave `Zeug` its plural, so `zeuge` is now a
    /// noun and `Werkzeuge` no longer needs the gate to stay open. The ones
    /// left are the umlaut plurals, which no affix class can build — there is
    /// no rule in `annotations.json` that turns `Raum` into `Räume` — and the
    /// verb forms.
    ///
    /// When the affix expansion starts assigning a part of speech to those
    /// too, this test fails, and that is the moment to try the typed element
    /// gate again.
    #[test]
    fn expanded_forms_carry_no_part_of_speech_to_type_elements_with() {
        use crate::language::german::spell::base_german_dictionary_fst;

        let dictionary = base_german_dictionary_fst();

        let word_class_of = |word: &str| {
            let letters: Vec<char> = word.chars().collect();
            let metadata = dictionary
                .get_word_metadata(&letters)
                .unwrap_or_else(|| panic!("{word} should be in the dictionary"));

            metadata.noun.is_some()
                || metadata.adjective.is_some()
                || metadata.adverb.is_some()
                || metadata.verb.is_some()
                || metadata.pronoun.is_some()
                || metadata.conjunction.is_some()
                || metadata.determiner.is_some()
                || metadata.affix.is_some()
                || metadata.preposition
        };

        for (word, what) in [
            ("eicht", "third person singular of 'eichen'"),
            ("malt", "third person singular of 'malen'"),
            ("agiert", "third person singular of 'agieren'"),
            ("räume", "umlaut plural of 'Raum'"),
            ("garten", "lower-case 'Garten'"),
        ] {
            assert!(
                !word_class_of(word),
                "{word} ({what}) now carries a part of speech; \
                 re-read MIN_COMPOUND_PART_LEN and try typing the element gate"
            );
        }
    }

    /// A bare verb stem is a piece of a compound, never a word.
    ///
    /// `absperr` exists in the word list so the conjugation affixes have
    /// something to attach to. Writing it on its own is a spelling mistake,
    /// and it used to pass — along with `geg`, `bes`, `erd` and two thousand
    /// others. Building on it is ordinary German.
    #[test]
    fn a_verb_stem_is_an_element_but_not_a_word() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        // Stems whose own pieces are not words either, so the decomposition
        // cannot put them back: `abspreiz` still passes as `ab` + `spreiz`.
        for stem in ["geg", "erd", "ahm", "bes"] {
            assert!(!accepts(stem), "{stem} is a stem, not a word");
        }

        for compound in [
            "Absperrband",
            "Erdgeschichte",
            "Schreibtisch",
            "Abspielgerät",
        ] {
            assert!(
                accepts(compound),
                "{compound} is built on a stem and must stay a word"
            );
        }

        // Still a word: hunspell reads these as imperatives, so
        // `mark_german_stems.py` leaves them alone.
        for imperative in ["hab", "werd", "soll", "quer", "vier", "paar"] {
            assert!(
                accepts(imperative),
                "{imperative} is a word in its own right"
            );
        }
    }

    /// Two-letter elements are nouns and particles, not leftovers.
    ///
    /// German does build compounds on two-letter words, but only on `Ei`, `Öl`
    /// and the particles. The word list also holds `ah`, `kt`, `mm`, `hl` and
    /// `äh`, and while those counted as elements, `Abau`, `Adahm` and `Adehl`
    /// all decomposed.
    #[test]
    fn a_two_letter_element_must_be_a_noun_or_a_preposition() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        for word in ["Adahm", "Adehl"] {
            assert!(!accepts(word), "{word} must not decompose");
        }

        for word in ["Ölgemälde", "Rohöl", "Eigelb", "Abbau", "Umbau", "Eiweiß"] {
            assert!(accepts(word), "{word} must stay a word");
        }
    }

    /// A linking interfix never repeats the letter it follows.
    ///
    /// Without that rule the seam swallows a doubled-consonant typo: the
    /// decomposition simply reads the extra letter as the interfix, and the
    /// misspelling draws no lint. The words below are the four shapes it
    /// produced most often on 1.44M words of prose.
    #[test]
    fn an_interfix_may_not_repeat_the_letter_before_it() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        for word in [
            "Aussbildung",
            "hinnaus",
            "annerkannt",
            "Annfang",
            "Anneignung",
            "allennfalls",
            "abhänngig",
        ] {
            assert!(!accepts(word), "{word} must not decompose");
        }

        // The empty interfix is exempt, and has to be: these really do meet on
        // the same consonant, with no linking letter between them.
        for word in [
            "Schifffahrt",
            "Schlusssatz",
            "Nussschale",
            "Balletttänzer",
            // And the genuine interfixes, which never double.
            "Arbeitsgeber",
            "Sonnenschein",
            "Kindergarten",
            "Bundesland",
        ] {
            assert!(accepts(word), "{word} must stay a word");
        }
    }

    /// A compound is made of content words, so a function word cannot end one.
    ///
    /// Every rejected word here is a doubled-letter typo that used to split its
    /// doubled letter off as an article or a pronoun.
    #[test]
    fn a_function_word_may_not_end_a_compound() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        for word in [
            "Badden",      // bad + den
            "Bildder",     // bild + der
            "Lassalle",    // lass + alle
            "anzuwendden", // an + zu + wend + den
            "Herder",      // herd + er
        ] {
            assert!(!accepts(word), "{word} must not decompose");
        }

        // A function word in the *opening* position is ordinary German, and so
        // is any compound that merely ends in the same letters.
        for word in [
            "Ausbildung",
            "Anfang",
            "Nachteil",
            "Bundestagswahl",
            "Arbeitsweg",
            "Außenminister",
            // Fixed entries, not decompositions -- they must not regress.
            "hinaus",
            "deswegen",
            "trotzdem",
            "infolgedessen",
        ] {
            assert!(accepts(word), "{word} must stay a word");
        }
    }

    /// A derived feminine noun opens a compound only with `-s-`, and the
    /// suffix itself takes nothing in front of it.
    #[test]
    fn a_derivational_suffix_fixes_the_interfix() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        for word in [
            "Abbildunggen", // abbildung + gen, with no -s- between them
            "Abhandlunggen",
            "Ablagerunggen",
        ] {
            assert!(!accepts(word), "{word} must not decompose");
        }

        // The `-s-` compounds themselves, and the words whose ending only
        // looks like a suffix: `Sprung`, `Schwung`, `Ursprung`, `Ion` and
        // `Schaft` are not derivations and take no interfix.
        for word in [
            "Bildungssystem",
            "Gesundheitsamt",
            "Gesellschaftsordnung",
            "Revolutionsführer",
            "Universitätsklinik",
            "Sprungbrett",
            "Ursprungsland",
            "Ionenaustausch",
            "Stadionbesuch",
            "Schaftfräser",
            // And the suffix attached directly, which is the normal case.
            "Bereitschaft",
            "Möglichkeit",
            "Freundschaftsdienst",
            "Arbeitsgemeinschaft",
        ] {
            assert!(accepts(word), "{word} must stay a word");
        }
    }

    /// `vieleicht` is rejected, and every compound that has the same shape is
    /// not.
    ///
    /// `viel` + `eicht` looks exactly like `statt` + `findet`: a particle, then
    /// a third person singular. The parts cannot tell them apart — `eicht` and
    /// `findet` are both listed with no part of speech. What tells them apart
    /// is that `stattfinden` is a verb and `vieleichen` is not.
    #[test]
    fn a_lowercase_compound_must_make_a_word_that_exists() {
        use crate::language::german::spell::combined_german_dictionary;

        let dictionary = combined_german_dictionary();
        let accepts = |word: &str| dictionary.contains_word(&word.chars().collect::<Vec<_>>());

        for word in [
            "vieleicht",
            "tischeicht",
            "baumeicht",
            "gartenmalt",
            "wandmalt",
            "vielagiert",
        ] {
            assert!(!accepts(word), "{word} must not decompose");
        }

        for word in [
            // The same shape, but the verb exists.
            "stattfindet",
            "verbleibt",
            "zurückgeht",
            "teilnimmt",
            // Ordinary lower-case compounds, whose head is an adjective or adverb.
            "umweltfreundlich",
            "wissenschaftlich",
            "vielsagend",
            "gleichzeitig",
            // Capitalized compounds are not touched by the rule at all.
            "Werkzeuge",
            "Zeiträume",
            "Säugetiere",
            "Donaudampfschifffahrtsgesellschaft",
        ] {
            assert!(accepts(word), "{word} must stay a word");
        }
    }
}
