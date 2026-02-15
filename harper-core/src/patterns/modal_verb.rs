use std::sync::LazyLock;

use super::{Pattern, WordSet};

pub struct ModalVerb {
    inner: &'static WordSet,
}

impl Default for ModalVerb {
    fn default() -> Self {
        let words = Self::init(false);
        Self { inner: words }
    }
}

impl ModalVerb {
    fn init(include_common_errors: bool) -> &'static WordSet {
        const MODALS: [&str; 14] = [
            "can", "can't", "could", "may", "might", "must", "shall", "shan't", "should", "will",
            "won't", "would", "ought", "dare",
        ];

        static CACHED_WITHOUT_COMMON_ERRORS: LazyLock<WordSet> = LazyLock::new(|| {
            let mut words = WordSet::new(&MODALS);
            MODALS.iter().for_each(|word| {
                words.add(&format!("{word}n't"));
            });
            words.add("cannot");
            words
        });

        static CACHED_WITH_COMMON_ERRORS: LazyLock<WordSet> = LazyLock::new(|| {
            let mut words = WordSet::new(&MODALS);
            MODALS.iter().for_each(|word| {
                words.add(&format!("{word}n't"));
                words.add(&format!("{word}nt"));
            });
            words.add("cannot");
            words
        });

        if include_common_errors {
            &CACHED_WITH_COMMON_ERRORS
        } else {
            &CACHED_WITHOUT_COMMON_ERRORS
        }
    }

    pub fn with_common_errors() -> Self {
        let words = Self::init(true);
        Self { inner: words }
    }
}

impl Pattern for ModalVerb {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
