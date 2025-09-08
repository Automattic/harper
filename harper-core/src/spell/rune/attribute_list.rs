use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use smallvec::ToSmallVec;

use super::super::word_map::{WordMap, WordMapEntry};
use super::Error;
use super::affix_replacement::AffixReplacement;
use super::expansion::Property;
use super::expansion::{
    AffixEntryKind,
    AffixEntryKind::{Prefix, Suffix},
    Expansion, HumanReadableExpansion,
};
use super::word_list::MarkedWord;
use crate::spell::WordId;
use crate::word_metadata_orthography::OrthFlags;
use crate::{CharString, CharStringExt, Span, WordMetadata};

#[derive(Debug, Clone)]
pub struct AttributeList {
    /// Key = Affix Flag
    affixes: HashMap<char, Expansion>,
    properties: HashMap<char, Property>,
}

impl AttributeList {
    fn into_human_readable(self) -> HumanReadableAttributeList {
        HumanReadableAttributeList {
            affixes: self
                .affixes
                .into_iter()
                .map(|(affix, exp)| (affix, exp.into_human_readable()))
                .collect(),
            properties: self.properties,
        }
    }

    pub fn parse(source: &str) -> Result<Self, Error> {
        let human_readable: Result<HumanReadableAttributeList, _> = serde_json::from_str(source);
        human_readable
            .map_err(Error::from)
            .and_then(|parsed| parsed.into_normal())
    }

    /// Expand [`MarkedWord`] into a list of full words, including itself.
    ///
    /// Will append to the given `dest`;
    ///
    /// In the future, I want to make this function cleaner and faster.
    pub fn expand_marked_word(&self, word: MarkedWord, dest: &mut WordMap) {
        dest.reserve(word.attributes.len() + 1);
        let mut gifted_metadata = WordMetadata::default();

        let orth_flags = check_orthography(&word);
        gifted_metadata.orth_info = orth_flags;

        let mut conditional_expansion_metadata = Vec::new();

        for attr in &word.attributes {
            let Some(property) = self.properties.get(attr) else {
                continue;
            };

            gifted_metadata.append(&property.metadata);
        }

        for attr in &word.attributes {
            let Some(expansion) = self.affixes.get(attr) else {
                continue;
            };

            gifted_metadata.append(&expansion.base_metadata);
            let mut new_words: HashMap<CharString, WordMetadata> = HashMap::new();

            for replacement in &expansion.replacements {
                if let Some(replaced) =
                    Self::apply_replacement(replacement, &word.letters, expansion.kind)
                {
                    let metadata = new_words.entry(replaced.clone()).or_default();
                    for target in &expansion.target {
                        if let Some(condition) = &target.if_base {
                            conditional_expansion_metadata.push((
                                replaced.clone(),
                                target.metadata.clone(),
                                condition.clone(),
                            ));
                        } else {
                            metadata.append(&target.metadata);
                        }
                    }
                }
            }

            if expansion.cross_product {
                let mut opp_attr = Vec::new();

                for attr in &word.attributes {
                    let Some(property) = self.properties.get(attr) else {
                        continue;
                    };
                    // This is the same logic as below, plus propagation
                    if expansion.kind == Prefix || property.propagate {
                        opp_attr.push(*attr);
                    }
                }

                for attr in &word.attributes {
                    let Some(attr_def) = self.affixes.get(attr) else {
                        continue;
                    };
                    // This looks wrong but matches the old logic: if attr_def.suffix != expansion.suffix
                    if (attr_def.kind != Prefix) != (expansion.kind != Prefix) {
                        opp_attr.push(*attr);
                    }
                }

                for (new_word, metadata) in new_words {
                    self.expand_marked_word(
                        MarkedWord {
                            letters: new_word.clone(),
                            attributes: opp_attr.clone(),
                        },
                        dest,
                    );
                    let t_metadata = dest.get_metadata_mut_chars(&new_word).unwrap();
                    t_metadata.append(&metadata);
                    t_metadata.derived_from = Some(WordId::from_word_chars(&word.letters))
                }
            } else {
                for (key, mut value) in new_words.into_iter() {
                    value.derived_from = Some(WordId::from_word_chars(&word.letters));

                    if let Some(val) = dest.get_metadata_mut_chars(&key) {
                        val.append(&value);
                    } else {
                        dest.insert(WordMapEntry {
                            canonical_spelling: key,
                            metadata: value,
                        });
                    }
                }
            }
        }

        let mut full_metadata = gifted_metadata;
        if let Some(prev_val) = dest.get_with_chars(&word.letters) {
            full_metadata.append(&prev_val.metadata);
        }

        dest.insert(WordMapEntry {
            metadata: full_metadata.clone(),
            canonical_spelling: word.letters,
        });

        for (letters, metadata, condition) in conditional_expansion_metadata {
            let condition_satisfied = full_metadata.or(&condition) == full_metadata;
            if !condition_satisfied {
                continue;
            }

            dest.get_metadata_mut_chars(&letters)
                .unwrap()
                .append(&metadata);
        }
    }

    /// Expand an iterator of marked words into strings.
    /// Note that this does __not__ guarantee that produced words will be
    /// unique.
    pub fn expand_marked_words(
        &self,
        words: impl IntoIterator<Item = MarkedWord>,
        dest: &mut WordMap,
    ) {
        for word in words {
            self.expand_marked_word(word, dest);
        }
    }

    fn apply_replacement(
        replacement: &AffixReplacement,
        letters: &[char],
        kind: AffixEntryKind,
    ) -> Option<CharString> {
        if replacement.condition.len() > letters.len() {
            return None;
        }

        let target_span = if kind == Suffix {
            Span::new(letters.len() - replacement.condition.len(), letters.len())
        } else {
            Span::new(0, replacement.condition.len())
        };

        let target_segment = target_span.get_content(letters);

        if replacement.condition.matches(target_segment) {
            let mut replaced_segment = letters.to_smallvec();
            let mut remove: CharString = replacement.remove.to_smallvec();

            if kind != Suffix {
                replaced_segment.reverse();
            } else {
                remove.reverse();
            }

            for c in &remove {
                let last = replaced_segment.last()?;

                if last == c {
                    replaced_segment.pop();
                } else {
                    return None;
                }
            }

            let mut to_add = replacement.add.to_vec();

            if kind != Suffix {
                to_add.reverse()
            }

            replaced_segment.extend(to_add);

            if kind != Suffix {
                replaced_segment.reverse();
            }

            return Some(replaced_segment);
        }

        None
    }
}

/// Gather metadata about the orthography of a word.
fn check_orthography(word: &MarkedWord) -> OrthFlags {
    use crate::char_ext::CharExt;
    use crate::word_metadata_orthography::OrthFlags;

    let mut ortho_flags = OrthFlags::default();
    let mut all_lower = true;
    let mut all_upper = true;
    let mut first_is_upper = false;
    let mut first_is_lower = false;
    let mut saw_upper_after_first = false;
    let mut saw_lower_after_first = false;
    let mut is_first_char = true;
    let mut upper_to_lower = false;
    let mut lower_to_upper = false;
    let letter_count = word
        .letters
        .iter()
        .filter(|c| c.is_english_lingual())
        .count();

    for &c in &word.letters {
        // Multiword: contains at least one space
        if c == ' ' {
            ortho_flags |= OrthFlags::MULTIWORD;
            continue;
        }
        // Hyphenated: contains at least one hyphen
        if c == '-' {
            ortho_flags |= OrthFlags::HYPHENATED;
            continue;
        }
        // Apostrophe: contains at least one apostrophe (straight or curly)
        if c == '\'' || c == '’' {
            ortho_flags |= OrthFlags::APOSTROPHE;
            continue;
        }
        // Only consider English letters for case flags
        if !c.is_english_lingual() {
            continue;
        }
        if c.is_lowercase() {
            all_upper = false;
            if is_first_char {
                first_is_lower = true;
            } else {
                saw_lower_after_first = true;
                if upper_to_lower {
                    lower_to_upper = true;
                }
                upper_to_lower = true;
            }
        } else if c.is_uppercase() {
            all_lower = false;
            if is_first_char {
                first_is_upper = true;
            } else {
                saw_upper_after_first = true;
                if lower_to_upper {
                    upper_to_lower = true;
                }
                lower_to_upper = true;
            }
        } else {
            // Non-cased char (e.g., numbers, symbols) - ignore for case flags
            // Reset case tracking after non-letter character
            first_is_upper = false;
            first_is_lower = false;
            upper_to_lower = false;
            lower_to_upper = false;
        }
        is_first_char = false;
    }

    // Set case-related orthography flags
    if letter_count > 0 {
        if all_lower {
            ortho_flags |= OrthFlags::LOWERCASE;
        }
        if all_upper {
            ortho_flags |= OrthFlags::ALLCAPS;
        }
        // Only mark as TITLECASE if more than one letter
        if letter_count > 1 && first_is_upper && !saw_upper_after_first {
            ortho_flags |= OrthFlags::TITLECASE;
        }
        // LowerCamel: first is lowercase and there's at least one uppercase character after it
        // Note: This must come after Titlecase check to avoid marking Titlecase words as LowerCamel
        // Example: "pH" is LowerCamel, but "Providence" is Titlecase
        if first_is_lower && saw_upper_after_first {
            ortho_flags |= OrthFlags::LOWER_CAMEL;
        }
        // UpperCamel: first is uppercase and there are both lowercase and uppercase characters after it
        // Note: This must come after Titlecase check to avoid marking Titlecase words as UpperCamel
        // Example: "CamelCase" is UpperCamel, but "Providence" is Titlecase
        if first_is_upper && saw_lower_after_first && saw_upper_after_first {
            ortho_flags |= OrthFlags::UPPER_CAMEL;
        }
    }

    if looks_like_roman_numerals(&word.letters)
        && is_really_roman_numerals(&word.letters.to_lower())
    {
        ortho_flags |= OrthFlags::ROMAN_NUMERALS;
    }

    ortho_flags
}

fn looks_like_roman_numerals(word: &CharString) -> bool {
    let mut is_roman = false;
    let first_char_upper;

    if let Some((&first, rest)) = word.split_first()
        && "mdclxvi".contains(first.to_ascii_lowercase())
    {
        first_char_upper = first.is_uppercase();

        for &c in rest {
            if !"mdclxvi".contains(c.to_ascii_lowercase()) || c.is_uppercase() != first_char_upper {
                return false;
            }
        }
        is_roman = true;
    }
    is_roman
}

fn is_really_roman_numerals(word: &[char]) -> bool {
    let s: String = word.iter().collect();
    let mut chars = s.chars().peekable();

    let mut m_count = 0;
    while m_count < 4 && chars.peek() == Some(&'m') {
        chars.next();
        m_count += 1;
    }

    if !check_roman_group(&mut chars, 'c', 'd', 'm') {
        return false;
    }

    if !check_roman_group(&mut chars, 'x', 'l', 'c') {
        return false;
    }

    if !check_roman_group(&mut chars, 'i', 'v', 'x') {
        return false;
    }

    if chars.next().is_some() {
        return false;
    }

    true
}

fn check_roman_group<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    one: char,
    five: char,
    ten: char,
) -> bool {
    match chars.peek() {
        Some(&c) if c == one => {
            chars.next();
            match chars.peek() {
                Some(&next) if next == ten || next == five => {
                    chars.next();
                    true
                }
                _ => {
                    let mut count = 0;
                    while count < 2 && chars.peek() == Some(&one) {
                        chars.next();
                        count += 1;
                    }
                    true
                }
            }
        }
        Some(&c) if c == five => {
            chars.next();
            let mut count = 0;
            while count < 3 && chars.peek() == Some(&one) {
                chars.next();
                count += 1;
            }
            true
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_metadata_orthography::OrthFlags;

    fn check_orthography_str(s: &str) -> OrthFlags {
        let word = MarkedWord {
            letters: s.chars().collect(),
            attributes: Vec::new(),
        };
        check_orthography(&word)
    }

    #[test]
    fn test_lowercase() {
        let flags = check_orthography_str("hello");
        assert!(flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        // With non-letters
        let flags = check_orthography_str("hello123");
        assert!(flags.contains(OrthFlags::LOWERCASE));
    }

    #[test]
    fn test_titlecase() {
        let flags = check_orthography_str("Hello");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        // Examples that should be titlecase
        assert!(check_orthography_str("World").contains(OrthFlags::TITLECASE));
        assert!(check_orthography_str("Something").contains(OrthFlags::TITLECASE));

        // These examples should NOT be titlecase (they're UPPER_CAMEL)
        assert!(!check_orthography_str("McDonald").contains(OrthFlags::TITLECASE));
        assert!(!check_orthography_str("O'Reilly").contains(OrthFlags::TITLECASE));

        // Single character should not be titlecase
        assert!(!check_orthography_str("A").contains(OrthFlags::TITLECASE));
    }

    #[test]
    fn test_allcaps() {
        let flags = check_orthography_str("HELLO");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        // Examples from docs
        assert!(check_orthography_str("NASA").contains(OrthFlags::ALLCAPS));
        assert!(check_orthography_str("I").contains(OrthFlags::ALLCAPS));
    }

    #[test]
    fn test_lower_camel() {
        let flags = check_orthography_str("helloWorld");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(!flags.contains(OrthFlags::UPPER_CAMEL));

        // Examples from docs
        assert!(check_orthography_str("getHTTPResponse").contains(OrthFlags::LOWER_CAMEL));
        assert!(check_orthography_str("eBay").contains(OrthFlags::LOWER_CAMEL));

        // All lowercase should not be lower camel
        assert!(!check_orthography_str("hello").contains(OrthFlags::LOWER_CAMEL));

        // Starts with uppercase should not be lower camel
        assert!(!check_orthography_str("HelloWorld").contains(OrthFlags::LOWER_CAMEL));
    }

    #[test]
    fn test_upper_camel() {
        let flags = check_orthography_str("HelloWorld");
        assert!(!flags.contains(OrthFlags::LOWERCASE));
        assert!(!flags.contains(OrthFlags::TITLECASE));
        assert!(!flags.contains(OrthFlags::ALLCAPS));
        assert!(!flags.contains(OrthFlags::LOWER_CAMEL));
        assert!(flags.contains(OrthFlags::UPPER_CAMEL));

        // Examples from docs
        assert!(check_orthography_str("HttpRequest").contains(OrthFlags::UPPER_CAMEL));
        assert!(check_orthography_str("McDonald").contains(OrthFlags::UPPER_CAMEL));
        assert!(check_orthography_str("O'Reilly").contains(OrthFlags::UPPER_CAMEL));
        assert!(check_orthography_str("XMLHttpRequest").contains(OrthFlags::UPPER_CAMEL));

        // Titlecase should not be upper camel
        assert!(!check_orthography_str("Hello").contains(OrthFlags::UPPER_CAMEL));

        // All caps should not be upper camel
        assert!(!check_orthography_str("NASA").contains(OrthFlags::UPPER_CAMEL));

        // Needs at least 3 chars
        assert!(!check_orthography_str("Hi").contains(OrthFlags::UPPER_CAMEL));
    }

    #[test]
    fn test_roman_numerals() {
        assert!(check_orthography_str("MCMXCIV").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("mdccclxxi").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("MMXXI").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("mcmxciv").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("MCMXCIV").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("MMI").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(check_orthography_str("MMXXV").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn test_single_roman_numeral() {
        assert!(check_orthography_str("i").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn empty_string_is_not_roman_numeral() {
        assert!(!check_orthography_str("").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn dont_allow_mixed_case_roman_numerals() {
        assert!(!check_orthography_str("MCMlxxxVIII").contains(OrthFlags::ROMAN_NUMERALS));
    }

    #[test]
    fn dont_allow_looks_like_but_isnt_roman_numeral() {
        assert!(!check_orthography_str("mdxlivx").contains(OrthFlags::ROMAN_NUMERALS));
        assert!(!check_orthography_str("XIXIVV").contains(OrthFlags::ROMAN_NUMERALS));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableAttributeList {
    affixes: HashMap<char, HumanReadableExpansion>,
    properties: HashMap<char, Property>,
}

impl HumanReadableAttributeList {
    pub fn into_normal(self) -> Result<AttributeList, Error> {
        let mut affixes = HashMap::with_capacity(self.affixes.len());

        for (affix, expansion) in self.affixes.into_iter() {
            affixes.insert(affix, expansion.into_normal()?);
        }

        Ok(AttributeList {
            affixes,
            properties: self.properties,
        })
    }
}
