use hashbrown::{Equivalent, HashMap};
use strum::IntoEnumIterator;

use crate::upos::UPOS;

/// A mapping between words and the most common UPOS tag.
#[derive(Debug, Default)]
pub struct FreqDict {
    pub mapping: HashMap<String, UPOS>,
}

impl FreqDict {
    pub fn new() -> Self {
        Default::default()
    }
}

/// A mapping between words and the frequency of each UPOS.
/// If an element is missing from the map, it's count is assumed to be zero.
#[derive(Debug, Default)]
pub struct FreqDictBuilder {
    mapping: HashMap<FreqDictBuilderKey, usize>,
}

impl FreqDictBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn inc(&mut self, word: &str, tag: &UPOS) {
        let counter = self.mapping.get_mut(&(word, tag));

        if let Some(counter) = counter {
            *counter += 1;
        } else {
            self.mapping.insert(
                FreqDictBuilderKey {
                    word: word.to_string(),
                    pos: *tag,
                },
                1,
            );
        }
    }

    // Inefficient, but effective method that gets the most used POS for a word in the map.
    // Returns none if the word does not exist in the map.
    fn most_freq_pos(&self, word: &str) -> Option<UPOS> {
        let mut max_found: Option<(UPOS, usize)> = None;

        for pos in UPOS::iter() {
            if let Some(count) = self.mapping.get(&(word, &pos)) {
                if let Some((_, max_count)) = max_found {
                    if *count > max_count {
                        max_found = Some((pos, *count))
                    }
                } else {
                    max_found = Some((pos, *count))
                }
            }
        }

        max_found.map(|v| v.0)
    }

    pub fn build(self) -> FreqDict {
        let mut output = HashMap::new();

        for key in self.mapping.keys() {
            if output.contains_key(&key.word) {
                continue;
            }

            output.insert(key.word.to_string(), self.most_freq_pos(&key.word).unwrap());
        }

        FreqDict { mapping: output }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct FreqDictBuilderKey {
    word: String,
    pos: UPOS,
}

impl Equivalent<FreqDictBuilderKey> for (&str, &UPOS) {
    fn equivalent(&self, key: &FreqDictBuilderKey) -> bool {
        self.0 == key.word && *self.1 == key.pos
    }
}
