use std::collections::HashMap;

use crate::UPOS;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ErrorKind {
    pub was_tagged: UPOS,
    pub correct_tag: UPOS,
}

#[derive(Debug, Default)]
pub struct ErrorCounter {
    pub error_counts: HashMap<ErrorKind, usize>,
}

impl ErrorCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the count for a particular lint kind.
    pub fn inc(&mut self, kind: ErrorKind) {
        self.error_counts
            .entry(kind)
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
    }

    pub fn total_errors(&self) -> usize {
        self.error_counts.values().sum()
    }
}
