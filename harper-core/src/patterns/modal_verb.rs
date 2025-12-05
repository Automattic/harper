use super::{Pattern, WordSet};

pub struct ModalVerb {
    inner: WordSet,
    include_common_errors: bool,
}

impl Default for ModalVerb {
    fn default() -> Self {
        let (words, include_common_errors) = Self::init(false);
        Self {
            inner: words,
            include_common_errors,
        }
    }
}

impl ModalVerb {
    fn init(include_common_errors: bool) -> (WordSet, bool) {
        let modals = [
            "can", "can't", "could", "may", "might", "must", "shall", "shan't", "should", "will",
            "won't", "would", "ought", "dare",
        ];

        let mut words = WordSet::new(&modals);
        modals.iter().for_each(|word| {
            words.add(&format!("{word}n't"));
            if include_common_errors {
                words.add(&format!("{word}nt"));
            }
        });
        words.add("cannot");
        (words, include_common_errors)
    }

    pub fn with_common_errors() -> Self {
        let (words, _) = Self::init(true);
        Self {
            inner: words,
            include_common_errors: true,
        }
    }
}

impl Pattern for ModalVerb {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
