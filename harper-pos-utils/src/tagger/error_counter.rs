use hashbrown::HashMap;

use crate::{UPOS, word_counter::WordCounter};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ErrorKind {
    pub was_tagged: UPOS,
    pub correct_tag: UPOS,
}

#[derive(Debug, Default)]
pub struct ErrorCounter {
    pub error_counts: HashMap<ErrorKind, usize>,
    /// The number of times a word is associated with an error.
    pub word_counts: WordCounter,
}

impl ErrorCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the count for a particular lint kind.
    pub fn inc(&mut self, kind: ErrorKind, word: &str) {
        self.error_counts
            .entry(kind)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
        self.word_counts.inc(word)
    }

    pub fn merge_from(&mut self, other: Self) {
        for (key, value) in other.error_counts {
            self.error_counts
                .entry(key)
                .and_modify(|counter| *counter += value)
                .or_insert(value);
        }

        for (key, value) in other.word_counts.word_counts {
            self.word_counts
                .word_counts
                .entry(key)
                .and_modify(|counter| *counter += value)
                .or_insert(value);
        }
    }

    pub fn total_errors(&self) -> usize {
        self.error_counts.values().sum()
    }
}
