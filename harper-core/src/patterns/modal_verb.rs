use std::sync::LazyLock;

use super::{Pattern, WordSet};

static MODALS: [&str; 14] = [
    "can", "can't", "could", "may", "might", "must", "shall", "shan't", "should", "will", "won't",
    "would", "ought", "dare",
];

pub struct ModalVerb {
    inner: &'static WordSet,
}

impl Default for ModalVerb {
    fn default() -> Self {
        Self::without_common_errors()
    }
}

impl ModalVerb {
    pub fn without_common_errors() -> Self {
        static CACHED_WITHOUT_COMMON_ERRORS: LazyLock<WordSet> = LazyLock::new(|| {
            let mut words = WordSet::new(&MODALS);
            MODALS.iter().for_each(|word| {
                words.add(&format!("{word}n't"));
            });
            words.add("cannot");
            words
        });

        Self {
            inner: &CACHED_WITHOUT_COMMON_ERRORS,
        }
    }

    pub fn with_common_errors() -> Self {
        static CACHED_WITH_COMMON_ERRORS: LazyLock<WordSet> = LazyLock::new(|| {
            let mut words = WordSet::new(&MODALS);
            MODALS.iter().for_each(|word| {
                words.add(&format!("{word}n't"));
                words.add(&format!("{word}nt"));
            });
            words.add("cannot");
            words
        });

        Self {
            inner: &CACHED_WITH_COMMON_ERRORS,
        }
    }
}

impl Pattern for ModalVerb {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
