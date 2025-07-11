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
use crate::{CharString, Span, WordMetadata};

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
        let human_readable: HumanReadableAttributeList =
            serde_json::from_str(source).map_err(|_| Error::MalformedJSON)?;

        human_readable.into_normal()
    }

    /// Expand [`MarkedWord`] into a list of full words, including itself.
    ///
    /// Will append to the given `dest`;
    ///
    /// In the future, I want to make this function cleaner and faster.
    pub fn expand_marked_word(&self, word: MarkedWord, dest: &mut WordMap) {
        dest.reserve(word.attributes.len() + 1);
        let mut gifted_metadata = WordMetadata::default();

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
