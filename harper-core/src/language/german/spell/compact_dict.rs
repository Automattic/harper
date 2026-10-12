//! A memory-lean dictionary for the expanded German word list.
//!
//! Affix expansion turns 245 000 lines into about 700 000 word forms. Held in a
//! [`MutableDictionary`](crate::spell::MutableDictionary) that is one 168-byte
//! entry per form in a hash table that rounds up to a power of two, and
//! [`FstDictionary`](crate::spell::FstDictionary) then keeps that beside an FST:
//! about 240 MB resident and twice that while building. German has a few
//! thousand *distinct* metadata values and short words, so almost all of that
//! is repetition.
//!
//! This stores
//!
//! - every spelling in one `char` array, with offsets;
//! - the metadata once per distinct value, and a `u32` per word pointing at it;
//! - an open-addressed table from [`WordId`] to word, for the lookups;
//! - an FST from spelling to word, for fuzzy matching.
//!
//! It answers the [`Dictionary`] trait the way `FstDictionary` does; a test
//! compares every word and its metadata against the expansion it was built from.

use std::borrow::Cow;

use fst::{IntoStreamer, Map as FstMap};
use hashbrown::HashMap;

use crate::spell::fst_dictionary::{build_dfa, merge_best_distances, stream_distances_vec};
use crate::spell::rune::{AttributeList, word_list::AnnotatedWord};
use crate::spell::word_map::{WordMap, WordMapEntry};
use crate::spell::{Dictionary, FuzzyMatchResult, MutableDictionary, WordId};
use crate::{CharString, CharStringExt, DictWordMetadata};

const EMPTY: u32 = u32::MAX;

/// The open-addressed table of word indices, probed by id. Half full at most, so probes stay short.
fn build_slots(ids: &[u64]) -> Vec<u32> {
    let mut slots = vec![EMPTY; (2 * ids.len()).next_power_of_two().max(2)];
    let mask = slots.len() - 1;
    for (index, &id) in ids.iter().enumerate() {
        let mut slot = id as usize & mask;
        while slots[slot] != EMPTY {
            debug_assert_ne!(ids[slots[slot] as usize], id, "a word was added twice");
            slot = (slot + 1) & mask;
        }
        slots[slot] = index as u32;
    }
    slots
}

/// See the [module documentation](self).
pub struct CompactDictionary {
    /// All spellings, concatenated.
    chars: Vec<char>,
    /// `offsets[i]..offsets[i + 1]` is word `i` in `chars`. One longer than the
    /// number of words.
    offsets: Vec<u32>,
    /// The [`WordId`] of word `i`, as its `u64`.
    ids: Vec<u64>,
    /// Open-addressed table of word indices, probed by id. `EMPTY` is a free slot.
    slots: Vec<u32>,
    /// Index into `metas` for word `i`.
    meta_of: Vec<u32>,
    /// Each distinct metadata value once, without `derived_from`.
    metas: Vec<DictWordMetadata>,
    /// `derived_from` of word `i`. It is a hash of another word, so it is
    /// different for nearly every word and would stop the rest of the metadata
    /// from being shared.
    derived_from: Vec<Option<u64>>,
    /// Spelling to word index, for fuzzy matching. The FST wants its keys
    /// sorted; the arrays above are not, so this is the only place that is.
    fst: FstMap<Vec<u8>>,
}

impl CompactDictionary {
    fn spelling(&self, index: usize) -> &[char] {
        &self.chars[self.offsets[index] as usize..self.offsets[index + 1] as usize]
    }

    fn metadata(&self, index: usize) -> Cow<'_, DictWordMetadata> {
        let shared = &self.metas[self.meta_of[index] as usize];
        match self.derived_from[index] {
            None => Cow::Borrowed(shared),
            Some(base) => {
                let mut metadata = shared.clone();
                metadata.derived_from = Some(WordId::from(base));
                Cow::Owned(metadata)
            }
        }
    }

    fn index_of_id(&self, id: u64) -> Option<usize> {
        let mask = self.slots.len() - 1;
        let mut slot = id as usize & mask;
        loop {
            match self.slots[slot] {
                EMPTY => return None,
                index if self.ids[index as usize] == id => return Some(index as usize),
                _ => slot = (slot + 1) & mask,
            }
        }
    }

    fn index_of(&self, word: &[char]) -> Option<usize> {
        self.index_of_id(WordId::from_word_chars(word).into())
    }

    /// Every word with its metadata.
    pub fn iter(&self) -> impl Iterator<Item = (&[char], Cow<'_, DictWordMetadata>)> {
        (0..self.len()).map(|i| (self.spelling(i), self.metadata(i)))
    }

    /// The number of words.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// The number of distinct metadata values the words share.
    pub fn distinct_metadata(&self) -> usize {
        self.metas.len()
    }
}

/// Collects finished words into a [`CompactDictionary`].
///
/// Expanding the whole word list into one [`WordMap`] needs 190 MB at once, and
/// the expansion cannot simply be cut into independent pieces: two headwords can
/// produce the same form, and which one wins `derived_from` or the spelling
/// depends on the code path that got there. [`expand_in_blocks`] therefore keeps
/// the one map the expansion needs, but hands each word over to this builder as
/// soon as no later block can touch it.
#[derive(Default)]
pub struct CompactBuilder {
    chars: Vec<char>,
    offsets: Vec<u32>,
    ids: Vec<u64>,
    meta_of: Vec<u32>,
    derived_from: Vec<Option<u64>>,
    metas: Vec<DictWordMetadata>,
    interned: HashMap<DictWordMetadata, u32>,
}

impl CompactBuilder {
    pub fn new() -> Self {
        Self {
            offsets: vec![0],
            ..Self::default()
        }
    }

    fn intern(&mut self, metadata: DictWordMetadata) -> u32 {
        if let Some(&index) = self.interned.get(&metadata) {
            return index;
        }
        let index = self.metas.len() as u32;
        self.metas.push(metadata.clone());
        self.interned.insert(metadata, index);
        index
    }

    /// Add a finished word. Every word must be added once only.
    pub fn push(&mut self, entry: WordMapEntry) {
        let id: u64 = WordId::from_word_chars(&entry.canonical_spelling).into();
        let mut metadata = entry.metadata;
        self.derived_from
            .push(metadata.derived_from.take().map(u64::from));
        let meta = self.intern(metadata);
        self.chars.extend_from_slice(&entry.canonical_spelling);
        self.offsets.push(self.chars.len() as u32);
        self.ids.push(id);
        self.meta_of.push(meta);
    }

    /// Build the lookup structures.
    pub fn finish(self) -> CompactDictionary {
        let Self {
            mut chars,
            mut offsets,
            mut ids,
            mut meta_of,
            metas,
            mut derived_from,
            ..
        } = self;
        chars.shrink_to_fit();
        offsets.shrink_to_fit();
        ids.shrink_to_fit();
        meta_of.shrink_to_fit();
        derived_from.shrink_to_fit();
        let n = ids.len();
        let spelling = |i: usize| &chars[offsets[i] as usize..offsets[i + 1] as usize];

        // The FST needs its keys in order. That order is all the sorting there is.
        let mut order: Vec<u32> = (0..n as u32).collect();
        order.sort_unstable_by(|&a, &b| spelling(a as usize).cmp(spelling(b as usize)));
        let mut fst_builder = fst::MapBuilder::memory();
        let mut key = String::new();
        for &index in &order {
            key.clear();
            key.extend(spelling(index as usize));
            fst_builder
                .insert(key.as_bytes(), index as u64)
                .expect("German words are inserted in sorted order, without repeats");
        }
        drop(order);

        let slots = build_slots(&ids);

        CompactDictionary {
            chars,
            offsets,
            ids,
            slots,
            meta_of,
            metas,
            derived_from,
            fst: FstMap::new(fst_builder.into_inner().expect("in-memory FST build"))
                .expect("Unable to build FST map."),
        }
    }
}

/// Why a pack could not be read. Any of these means "expand the word list instead".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackError(pub &'static str);

impl std::fmt::Display for PackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unusable dictionary pack: {}", self.0)
    }
}

impl std::error::Error for PackError {}

const PACK_MAGIC: &[u8; 4] = b"HDPK";
const PACK_FORMAT: u32 = 1;
/// `derived_from` of a word that has none. A real id is a hash and never this.
const NO_BASE: u64 = u64::MAX;

/// Cuts a byte slice into the pieces a pack is made of, without ever reading past its end.
struct PackReader<'a>(&'a [u8]);

impl<'a> PackReader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], PackError> {
        if len > self.0.len() {
            return Err(PackError("the file is shorter than its header says"));
        }
        let (head, tail) = self.0.split_at(len);
        self.0 = tail;
        Ok(head)
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], PackError> {
        Ok(self.take(N)?.try_into().expect("take returns N bytes"))
    }

    fn length(&mut self) -> Result<usize, PackError> {
        usize::try_from(u64::from_le_bytes(self.take_array()?))
            .map_err(|_| PackError("a length does not fit in memory"))
    }

    fn records(
        &mut self,
        count: usize,
        size: usize,
    ) -> Result<std::slice::ChunksExact<'a, u8>, PackError> {
        let len = count
            .checked_mul(size)
            .ok_or(PackError("a length overflows"))?;
        Ok(self.take(len)?.chunks_exact(size))
    }
}

impl CompactDictionary {
    /// The dictionary as bytes that [`Self::from_pack`] reads back without expanding anything.
    ///
    /// `stamp` identifies what the dictionary was built from. A pack is only
    /// read back by a caller that presents the same stamp, so a pack that no
    /// longer matches the word list or the code is never used.
    pub fn to_pack(&self, stamp: u64) -> Vec<u8> {
        let metas = serde_json::to_vec(&self.metas).expect("metadata serializes");
        let text: String = self.chars.iter().collect();
        let fst = self.fst.as_fst().as_bytes();
        let mut out = Vec::with_capacity(text.len() + fst.len() + self.len() * 14 + metas.len());
        out.extend(PACK_MAGIC);
        out.extend(PACK_FORMAT.to_le_bytes());
        out.extend(stamp.to_le_bytes());
        for len in [
            self.len(),
            self.chars.len(),
            text.len(),
            fst.len(),
            metas.len(),
        ] {
            out.extend((len as u64).to_le_bytes());
        }
        out.extend(&metas);
        out.extend(fst);
        out.extend(self.offsets.iter().flat_map(|o| o.to_le_bytes()));
        for &meta in &self.meta_of {
            let meta = u16::try_from(meta).expect("fewer than 65 536 distinct metadata values");
            out.extend(meta.to_le_bytes());
        }
        for base in &self.derived_from {
            assert_ne!(
                *base,
                Some(NO_BASE),
                "an id collides with the 'no base' marker"
            );
            out.extend(base.unwrap_or(NO_BASE).to_le_bytes());
        }
        out.extend(text.as_bytes());
        out
    }

    /// Read what [`Self::to_pack`] wrote. The ids and the lookup table are
    /// rebuilt rather than stored; that takes about a tenth of a second.
    pub fn from_pack(bytes: &[u8], stamp: u64) -> Result<Self, PackError> {
        let mut reader = PackReader(bytes);
        if &reader.take_array::<4>()? != PACK_MAGIC {
            return Err(PackError("not a dictionary pack"));
        }
        if u32::from_le_bytes(reader.take_array()?) != PACK_FORMAT {
            return Err(PackError("written in another format"));
        }
        if u64::from_le_bytes(reader.take_array()?) != stamp {
            return Err(PackError("built from another word list or another version"));
        }
        let n = reader.length()?;
        let char_count = reader.length()?;
        let text_len = reader.length()?;
        let fst_len = reader.length()?;
        let metas_len = reader.length()?;

        let metas: Vec<DictWordMetadata> = serde_json::from_slice(reader.take(metas_len)?)
            .map_err(|_| PackError("the metadata does not parse"))?;
        let fst = FstMap::new(reader.take(fst_len)?.to_vec())
            .map_err(|_| PackError("the FST is damaged"))?;
        if fst.len() != n {
            return Err(PackError("the FST and the word count disagree"));
        }
        let offsets: Vec<u32> = reader
            .records(n + 1, 4)?
            .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")))
            .collect();
        let in_order = offsets.windows(2).all(|w| w[0] <= w[1]);
        if offsets.first() != Some(&0) || offsets.last() != Some(&(char_count as u32)) || !in_order
        {
            return Err(PackError("the offsets are inconsistent"));
        }
        let meta_of: Vec<u32> = reader
            .records(n, 2)?
            .map(|b| u32::from(u16::from_le_bytes(b.try_into().expect("2 bytes"))))
            .collect();
        if meta_of.iter().any(|&m| m as usize >= metas.len()) {
            return Err(PackError("a word points at metadata that is not there"));
        }
        let derived_from: Vec<Option<u64>> = reader
            .records(n, 8)?
            .map(|b| u64::from_le_bytes(b.try_into().expect("8 bytes")))
            .map(|base| (base != NO_BASE).then_some(base))
            .collect();
        let text = std::str::from_utf8(reader.take(text_len)?)
            .map_err(|_| PackError("the spellings are not UTF-8"))?;
        let chars: Vec<char> = text.chars().collect();
        if chars.len() != char_count {
            return Err(PackError("the spellings and the character count disagree"));
        }

        let ids: Vec<u64> = offsets
            .windows(2)
            .map(|w| WordId::from_word_chars(&chars[w[0] as usize..w[1] as usize]).into())
            .collect();
        let slots = build_slots(&ids);
        Ok(Self {
            chars,
            offsets,
            ids,
            slots,
            meta_of,
            metas,
            derived_from,
            fst,
        })
    }
}

/// How many headwords are expanded between two hand-overs to the builder.
const BLOCK: usize = 8192;

/// How many guesses [`expand_in_blocks`] makes at where a form sits in its sorted list.
const INTERPOLATION_STEPS: usize = 8;

/// Expand `word_list` through `attr_list` and build the dictionary, with the
/// same result as expanding it into one [`WordMap`] and a fraction of the memory.
///
/// A first pass expands each block into a map of its own, on as many threads as
/// there are, only to learn the last block that produces each form. The second
/// pass expands the blocks in order into one working map, and after each block
/// moves every form that no later block produces into the builder. What stays in
/// the working map is exactly what a later block can still change.
pub fn expand_in_blocks(
    attr_list: &AttributeList,
    word_list: &[AnnotatedWord],
) -> CompactDictionary {
    let blocks: Vec<&[AnnotatedWord]> = word_list.chunks(BLOCK).collect();

    // Every form each block produces or looks up, with the block's number. A
    // bare stem is looked up but never stored, so the headwords count too.
    let forms_of_blocks = |range: std::ops::Range<usize>| {
        let mut seen = Vec::<(u64, u32)>::new();
        for index in range {
            let mut map = WordMap::default();
            attr_list.expand_annotated_words(blocks[index].iter().cloned(), &mut map);
            seen.extend(map.iter().map(|entry| {
                (
                    u64::from(WordId::from_word_chars(&entry.canonical_spelling)),
                    index as u32,
                )
            }));
            seen.extend(blocks[index].iter().map(|word| {
                (
                    u64::from(WordId::from_word_chars(&word.letters)),
                    index as u32,
                )
            }));
        }
        seen
    };
    let threads = if cfg!(target_arch = "wasm32") {
        1
    } else {
        std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(blocks.len().max(1))
    };
    let per_thread = blocks.len().div_ceil(threads.max(1)).max(1);
    let ranges: Vec<_> = (0..blocks.len())
        .step_by(per_thread)
        .map(|start| start..(start + per_thread).min(blocks.len()))
        .collect();
    let parts: Vec<Vec<(u64, u32)>> = if ranges.len() <= 1 {
        ranges.into_iter().map(forms_of_blocks).collect()
    } else {
        std::thread::scope(|scope| {
            let handles: Vec<_> = ranges
                .into_iter()
                .map(|range| scope.spawn(|| forms_of_blocks(range)))
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("German dictionary pre-pass panicked"))
                .collect()
        })
    };
    // Sorted by id, and by block within an id, so the last of each id is the
    // block that matters.
    let mut seen: Vec<(u64, u32)> = Vec::with_capacity(parts.iter().map(Vec::len).sum());
    for part in parts {
        seen.extend(part);
    }
    seen.sort_unstable();
    seen.dedup_by(|later, earlier| {
        // `dedup_by` calls this with the later element first, and drops it when
        // told to; the earlier one stays and takes over the later block number.
        if later.0 == earlier.0 {
            earlier.1 = later.1;
            true
        } else {
            false
        }
    });
    seen.shrink_to_fit();
    // Ids are hashes, so they are spread evenly over the `u64` range, and a
    // guess at the position by interpolation lands within a few entries of the
    // answer. A plain binary search over a million entries misses the cache at
    // every step, and this runs once per form.
    let last_block = |id: u64| {
        let (mut low, mut high) = (0, seen.len() - 1);
        for _ in 0..INTERPOLATION_STEPS {
            let (first, last) = (seen[low].0, seen[high].0);
            if first == last {
                break;
            }
            let guess = low
                + ((id - first) as u128 * (high - low) as u128 / (last - first) as u128) as usize;
            match seen[guess].0.cmp(&id) {
                std::cmp::Ordering::Equal => return seen[guess].1 as usize,
                std::cmp::Ordering::Less => low = guess + 1,
                std::cmp::Ordering::Greater => high = guess - 1,
            }
        }
        // Ids that are not spread evenly fall back to the search that always works.
        let at = low + seen[low..=high].partition_point(|&(other, _)| other < id);
        seen[at].1 as usize
    };

    let mut builder = CompactBuilder::new();
    let mut working = WordMap::default();
    for (index, block) in blocks.iter().enumerate() {
        attr_list.expand_annotated_words(block.iter().cloned(), &mut working);
        // The map iterates in an order that changes from run to run; the
        // dictionary should not inherit that.
        let mut finished = Vec::new();
        for entry in std::mem::take(&mut working) {
            let id: u64 = WordId::from_word_chars(&entry.canonical_spelling).into();
            if last_block(id) == index {
                finished.push((id, entry));
            } else {
                working.insert(entry);
            }
        }
        finished.sort_unstable_by_key(|&(id, _)| id);
        for (_, entry) in finished {
            builder.push(entry);
        }
    }
    builder.finish()
}

impl Dictionary for CompactDictionary {
    fn contains_word(&self, word: &[char]) -> bool {
        self.index_of(word).is_some()
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: CharString = word.chars().collect();
        self.contains_word(&chars)
    }

    fn get_word_metadata(&self, word: &[char]) -> Option<Cow<'_, DictWordMetadata>> {
        self.index_of(word).map(|index| self.metadata(index))
    }

    fn get_word_metadata_str(&self, word: &str) -> Option<Cow<'_, DictWordMetadata>> {
        let chars: CharString = word.chars().collect();
        self.get_word_metadata(&chars)
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        let misspelled_word_charslice = word.normalized();
        let misspelled_word_string = misspelled_word_charslice.to_string();
        let misspelled_lower = misspelled_word_string.to_lowercase();
        let is_already_lower = misspelled_lower == misspelled_word_string;

        let mut best_distances = HashMap::<u64, u8>::new();

        let dfa = build_dfa(max_distance, &misspelled_word_string);
        let mut stream = self.fst.search_with_state(&dfa).into_stream();
        merge_best_distances(&mut best_distances, stream_distances_vec(&mut stream, &dfa));

        if !is_already_lower {
            let dfa_lowercase = build_dfa(max_distance, &misspelled_lower);
            let mut stream = self.fst.search_with_state(&dfa_lowercase).into_stream();
            merge_best_distances(
                &mut best_distances,
                stream_distances_vec(&mut stream, &dfa_lowercase),
            );
        }

        let mut merged: Vec<_> = best_distances
            .into_iter()
            .filter(|&(_, edit_distance)| edit_distance > 0)
            .map(|(index, edit_distance)| FuzzyMatchResult {
                word: self.spelling(index as usize),
                edit_distance,
                metadata: self.metadata(index as usize),
            })
            .collect();

        merged.sort_unstable_by(|a, b| {
            a.edit_distance
                .cmp(&b.edit_distance)
                .then_with(|| a.word.cmp(b.word))
        });
        merged.truncate(max_results);

        merged
    }

    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<FuzzyMatchResult<'_>> {
        self.fuzzy_match(
            word.chars().collect::<Vec<_>>().as_slice(),
            max_distance,
            max_results,
        )
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new((0..self.len()).map(|i| self.spelling(i)))
    }

    fn word_count(&self) -> usize {
        self.len()
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        let normalized = word.normalized();

        self.index_of(normalized.as_ref())
            .is_some_and(|index| self.spelling(index) == normalized.as_ref())
    }

    fn contains_exact_word_str(&self, word: &str) -> bool {
        let word: CharString = word.chars().collect();
        self.contains_exact_word(word.as_ref())
    }

    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        self.index_of(word).map(|index| self.spelling(index))
    }

    fn get_word_from_id(&self, id: &WordId) -> Option<&[char]> {
        self.index_of_id((*id).into())
            .map(|index| self.spelling(index))
    }

    fn find_words_with_prefix(&self, prefix: &[char]) -> Vec<Cow<'_, [char]>> {
        self.words_iter()
            .filter(|word| word.starts_with(prefix))
            .map(Cow::Borrowed)
            .collect()
    }

    fn find_words_with_common_prefix(&self, word: &[char]) -> Vec<Cow<'_, [char]>> {
        self.words_iter()
            .filter(|item| word.starts_with(item))
            .map(Cow::Borrowed)
            .collect()
    }
}

impl From<MutableDictionary> for CompactDictionary {
    fn from(dict: MutableDictionary) -> Self {
        let mut map = WordMap::default();
        for (word, metadata) in dict.iter() {
            map.insert(WordMapEntry {
                canonical_spelling: word.clone(),
                metadata: metadata.clone(),
            });
        }
        let mut builder = CompactBuilder::new();
        for entry in map {
            builder.push(entry);
        }
        builder.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::german_dict::{german_word_list, mutable_german_dictionary};
    use super::*;
    use crate::spell::FstDictionary;

    /// The word list expanded into a single map, which is what the dictionary
    /// was built from before it was built a block at a time.
    fn one_shot_expansion() -> WordMap {
        let attr_list = AttributeList::parse(include_str!("../annotations.json")).unwrap();
        let mut map = WordMap::default();
        attr_list.expand_annotated_words(german_word_list().into_iter(), &mut map);
        map
    }

    #[test]
    fn blockwise_build_matches_one_shot_expansion() {
        let expected = one_shot_expansion();
        let compact = mutable_german_dictionary();

        assert_eq!(compact.len(), expected.len());
        let mut differing = Vec::new();
        for entry in expected.iter() {
            let id = WordId::from_word_chars(&entry.canonical_spelling);
            let Some(spelling) = compact.get_word_from_id(&id) else {
                differing.push(format!("missing {:?}", entry.canonical_spelling));
                continue;
            };
            let metadata = compact.get_word_metadata(spelling).unwrap();
            // The expansion itself is not deterministic about the capitalisation
            // of a form that two headwords spell differently (`-in` against
            // `-In`): two runs of it differ in about 4000 of them. Anything but
            // the case has to agree.
            if spelling.to_lower() != entry.canonical_spelling.to_lower() {
                differing.push(format!(
                    "SPELL {:?}/{:?}",
                    spelling.iter().collect::<String>(),
                    entry.canonical_spelling.iter().collect::<String>()
                ));
            } else if *metadata != entry.metadata {
                let (a, b) = (format!("{:?}", *metadata), format!("{:?}", entry.metadata));
                let at = a
                    .chars()
                    .zip(b.chars())
                    .position(|(x, y)| x != y)
                    .unwrap_or(0);
                differing.push(format!(
                    "META {} ... got {:?} / want {:?}",
                    spelling.iter().collect::<String>(),
                    &a[at.saturating_sub(40)..(at + 60).min(a.len())],
                    &b[at.saturating_sub(40)..(at + 60).min(b.len())]
                ));
            }
        }
        assert!(
            differing.is_empty(),
            "{} of {} words differ, e.g. {:?}",
            differing.len(),
            expected.len(),
            &differing[..differing.len().min(8)]
        );
    }

    #[test]
    fn fuzzy_matching_agrees_with_the_fst_dictionary() {
        let expected = FstDictionary::new(
            one_shot_expansion()
                .into_iter()
                .map(|entry| (entry.canonical_spelling, entry.metadata))
                .collect(),
        );
        let compact = mutable_german_dictionary();

        for word in [
            "Haus",
            "haus",
            "Hauss",
            "Strase",
            "gehn",
            "Ärtze",
            "Frühstük",
            "Maedchen",
            "unmoeglich",
            "Bundesregirung",
            "schön",
            "Zeitung",
            "kaufte",
            "gelaufen",
            "daß",
            "Fahrradweg",
            "x",
            "Qwertz",
        ] {
            let summary = |dict: &dyn Dictionary| {
                dict.fuzzy_match_str(word, 2, 12)
                    .into_iter()
                    .map(|m| (m.word.iter().collect::<String>(), m.edit_distance))
                    .collect::<Vec<_>>()
            };
            assert_eq!(summary(compact.as_ref()), summary(&expected), "{word}");
            assert_eq!(
                compact.contains_word_str(word),
                expected.contains_word_str(word),
                "{word}"
            );
            assert_eq!(
                compact.get_word_metadata_str(word),
                expected.get_word_metadata_str(word),
                "{word}"
            );
        }
        assert_eq!(compact.word_count(), expected.word_count());
    }

    #[test]
    fn a_pack_reads_back_as_the_dictionary_it_was_written_from() {
        let dict = mutable_german_dictionary();
        let pack = dict.to_pack(7);
        let back = CompactDictionary::from_pack(&pack, 7).ok().unwrap();

        assert_eq!(back.len(), dict.len());
        for index in 0..dict.len() {
            assert_eq!(back.spelling(index), dict.spelling(index));
            assert_eq!(back.ids[index], dict.ids[index]);
            assert_eq!(back.metadata(index), dict.metadata(index));
        }
        for word in [
            "Haus",
            "Häuser",
            "lernen",
            "gelernt",
            "Mitteleuropa",
            "xyzzy",
        ] {
            assert_eq!(back.contains_word_str(word), dict.contains_word_str(word));
            assert_eq!(
                back.get_word_metadata_str(word),
                dict.get_word_metadata_str(word)
            );
        }
        let (a, b) = (
            back.fuzzy_match_str("Hauss", 2, 20),
            dict.fuzzy_match_str("Hauss", 2, 20),
        );
        assert_eq!(a.len(), b.len());
    }

    #[test]
    fn a_pack_that_does_not_fit_is_refused_and_never_read_past_its_end() {
        let pack = mutable_german_dictionary().to_pack(7);
        let refused = |bytes: &[u8], stamp| CompactDictionary::from_pack(bytes, stamp).is_err();

        assert!(refused(&pack, 8), "another stamp");
        assert!(refused(b"", 7));
        assert!(refused(b"not a pack at all", 7));
        let mut other_format = pack.clone();
        other_format[4] ^= 1;
        assert!(refused(&other_format, 7));
        for cut in [3, 20, 60, 5_000, pack.len() / 2, pack.len() - 1] {
            assert!(refused(&pack[..cut], 7), "cut at {cut}");
        }
    }

    #[test]
    fn metadata_is_shared_between_words() {
        let compact = mutable_german_dictionary();
        assert!(
            compact.distinct_metadata() * 20 < compact.len(),
            "{} distinct values for {} words",
            compact.distinct_metadata(),
            compact.len()
        );
    }
}
