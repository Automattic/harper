use super::{Pattern, WordSet};

pub struct ModalVerb {
    inner: WordSet,
}

impl Default for ModalVerb {
    fn default() -> Self {
        let modals = [
            "can", "could", "may", "might", "must", "shall", "should", "will", "would", "ought",
            "dare",
        ];
        let mut words = WordSet::new(&modals);
        modals.iter().for_each(|word| {
            words.add(&format!("{word}n't"));
        });

        Self { inner: words }
    }
}

impl Pattern for ModalVerb {
    fn matches(&self, tokens: &[crate::Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}
