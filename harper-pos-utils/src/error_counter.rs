use hashbrown::HashMap;

use crate::UPOS;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ErrorKind {
    pub was_tagged: UPOS,
    pub correct_tag: UPOS,
}

#[derive(Debug, Default)]
pub struct ErrorCounter {
    pub error_counts: HashMap<ErrorKind, usize>,
    /// The number of times a word is associated with an error.
    pub word_counts: HashMap<String, usize>,
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
        self.word_counts
            .entry_ref(word)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    pub fn merge_from(&mut self, other: Self) {
        for (key, value) in other.error_counts {
            self.error_counts
                .entry(key)
                .and_modify(|counter| *counter += value)
                .or_insert(value);
        }

        for (key, value) in other.word_counts {
            self.word_counts
                .entry(key)
                .and_modify(|counter| *counter += value)
                .or_insert(value);
        }
    }

    pub fn total_errors(&self) -> usize {
        self.error_counts.values().sum()
    }

    /// Get an iterator over the most frequent words associated with errors.
    pub fn iter_top_n_words(&self, n: usize) -> impl Iterator<Item = &String> {
        let mut counts: Vec<(&String, &usize)> = self.word_counts.iter().collect();
        counts.sort_unstable_by(|a, b| b.1.cmp(a.1));
        counts.into_iter().take(n).map(|(a, _b)| a)
    }
}
