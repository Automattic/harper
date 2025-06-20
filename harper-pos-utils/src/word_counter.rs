use hashbrown::HashMap;

#[derive(Debug, Default)]
pub struct WordCounter {
    /// The number of times a word is associated with an error.
    pub word_counts: HashMap<String, usize>,
}

impl WordCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the count for a particular word.
    pub fn inc(&mut self, word: &str) {
        self.word_counts
            .entry_ref(word)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    }

    /// Get an iterator over the most frequent words associated with errors.
    pub fn iter_top_n_words(&self, n: usize) -> impl Iterator<Item = &String> {
        let mut counts: Vec<(&String, &usize)> = self.word_counts.iter().collect();
        counts.sort_unstable_by(|a, b| b.1.cmp(a.1));
        counts.into_iter().take(n).map(|(a, _b)| a)
    }
}
