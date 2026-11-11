use std::borrow::Cow;
#[cfg(any(test, feature = "training"))]
use std::path::Path;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

pub const PAD_IDX: usize = 0;
pub const UNK_IDX: usize = 1;
#[cfg(feature = "training")]
pub const SPECIAL_WORDS: [&str; 4] = ["<PAD>", "<UNK>", "<TARGET>", "<NUM>"];

/// The maximum number of words the ranker reads on each side of a misspelled word.
pub const CONTEXT_RADIUS: usize = 8;

pub(crate) const CONTEXT_SLOTS: usize = CONTEXT_RADIUS * 2;

// `table.bin` holds the quantization limit, the slot weights, the word weights, then the `i8` rows.
const SLOT_WEIGHTS_AT: usize = size_of::<f32>();
const WORD_WEIGHTS_AT: usize = SLOT_WEIGHTS_AT + CONTEXT_SLOTS * size_of::<f32>();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextRankerConfig {
    /// The number of values in the vector of each word.
    pub dimensions: usize,
    pub spelling_weight: f32,
}

pub(crate) fn is_known_word(id: u16) -> bool {
    id as usize != PAD_IDX && id as usize != UNK_IDX
}

/// The score that orders a candidate. Higher is better.
pub(crate) fn combined_score(fit: f32, spelling_score: i32, spelling_weight: f32) -> f32 {
    fit - spelling_weight * spelling_score as f32
}

/// Orders spelling candidates by how well they fit the words around a misspelled word.
pub struct ContextRanker {
    vocab: HashMap<String, u16>,
    table: Cow<'static, [u8]>,
    config: ContextRankerConfig,
}

impl ContextRanker {
    pub(crate) fn from_parts(
        words: &[String],
        table: Cow<'static, [u8]>,
        config: ContextRankerConfig,
    ) -> Self {
        assert_eq!(
            table.len(),
            WORD_WEIGHTS_AT + words.len() * (size_of::<f32>() + config.dimensions),
            "the table does not match the vocabulary and the dimensions",
        );

        Self {
            vocab: words
                .iter()
                .enumerate()
                .map(|(id, word)| (word.clone(), id as u16))
                .collect(),
            table,
            config,
        }
    }

    pub fn load_from_bytes(
        table: impl Into<Cow<'static, [u8]>>,
        vocab_bytes: impl AsRef<[u8]>,
        config_bytes: impl AsRef<[u8]>,
    ) -> Self {
        let words: Vec<String> = serde_json::from_slice(vocab_bytes.as_ref()).unwrap();
        let config: ContextRankerConfig = serde_json::from_slice(config_bytes.as_ref()).unwrap();

        Self::from_parts(&words, table.into(), config)
    }

    #[cfg(any(test, feature = "training"))]
    pub fn save_to(&self, dir: impl AsRef<Path>) {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir).unwrap();

        let mut words = vec![String::new(); self.vocab.len()];
        for (word, id) in &self.vocab {
            words[*id as usize].clone_from(word);
        }

        std::fs::write(dir.join("table.bin"), &self.table).unwrap();
        std::fs::write(dir.join("vocab.json"), serde_json::to_vec(&words).unwrap()).unwrap();
        std::fs::write(
            dir.join("config.json"),
            serde_json::to_vec_pretty(&self.config).unwrap(),
        )
        .unwrap();
    }

    #[cfg(test)]
    pub fn config(&self) -> &ContextRankerConfig {
        &self.config
    }

    #[cfg(feature = "training")]
    pub fn set_spelling_weight(&mut self, spelling_weight: f32) {
        self.config.spelling_weight = spelling_weight;
    }

    fn float_at(&self, offset: usize) -> f32 {
        f32::from_le_bytes(self.table[offset..offset + 4].try_into().unwrap())
    }

    fn row(&self, id: u16) -> impl Iterator<Item = f32> + '_ {
        let dimensions = self.config.dimensions;
        let start =
            WORD_WEIGHTS_AT + self.vocab.len() * size_of::<f32>() + id as usize * dimensions;

        self.table[start..start + dimensions]
            .iter()
            .map(|value| *value as i8 as f32)
    }

    fn word_id(&self, word: &str) -> u16 {
        self.vocab.get(word).copied().unwrap_or(UNK_IDX as u16)
    }

    /// The word ids by context slot, or `None` if the radius holds no known word.
    fn context_ids(&self, left: &[String], right: &[String]) -> Option<[u16; CONTEXT_SLOTS]> {
        let mut ids = [PAD_IDX as u16; CONTEXT_SLOTS];

        for (slot, word) in ids[..CONTEXT_RADIUS]
            .iter_mut()
            .rev()
            .zip(left.iter().rev())
        {
            *slot = self.word_id(word);
        }
        for (slot, word) in ids[CONTEXT_RADIUS..].iter_mut().zip(right) {
            *slot = self.word_id(word);
        }

        ids.iter().any(|id| is_known_word(*id)).then_some(ids)
    }

    /// How well each candidate fits the context. Higher is better.
    pub(crate) fn fits(
        &self,
        context: &[u16; CONTEXT_SLOTS],
        candidates: impl Iterator<Item = u16>,
    ) -> Vec<f32> {
        let scale = self.float_at(0) / i8::MAX as f32;
        let mut vector = vec![0.0; self.config.dimensions];

        for (slot, id) in context.iter().enumerate() {
            if *id as usize != PAD_IDX {
                let weight = self.float_at(SLOT_WEIGHTS_AT + slot * size_of::<f32>())
                    * self.float_at(WORD_WEIGHTS_AT + *id as usize * size_of::<f32>());
                for (total, value) in vector.iter_mut().zip(self.row(*id)) {
                    *total += weight * value;
                }
            }
        }

        candidates
            .map(|id| {
                vector
                    .iter()
                    .zip(self.row(id))
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
                    * scale
                    * scale
            })
            .collect()
    }

    /// The candidate positions, best first.
    ///
    /// Each candidate is a lowercase word and its spelling score. Lower spelling scores are better.
    /// The order stays the same when the radius holds no known word.
    pub fn rank(
        &self,
        left: &[String],
        right: &[String],
        candidates: &[(String, i32)],
    ) -> Vec<usize> {
        let mut order: Vec<usize> = (0..candidates.len()).collect();

        if candidates.len() < 2 {
            return order;
        }

        let Some(context) = self.context_ids(left, right) else {
            return order;
        };

        let scores: Vec<f32> = self
            .fits(
                &context,
                candidates.iter().map(|(word, _)| self.word_id(word)),
            )
            .iter()
            .zip(candidates)
            .map(|(fit, (_, spelling_score))| {
                combined_score(*fit, *spelling_score, self.config.spelling_weight)
            })
            .collect();

        order.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]));
        order
    }
}

#[cfg(test)]
mod tests {
    use super::{CONTEXT_RADIUS, CONTEXT_SLOTS, ContextRanker, ContextRankerConfig};

    const WORDS: [&str; 8] = [
        "<PAD>", "<UNK>", "<TARGET>", "<NUM>", "one", "two", "five", "file",
    ];

    fn ranker(spelling_weight: f32) -> ContextRanker {
        let rows: [[i8; 2]; 8] = [
            [0, 0],
            [0, 0],
            [0, 0],
            [0, 0],
            [127, 0],
            [127, 0],
            [127, 0],
            [0, 127],
        ];

        let mut table = 1f32.to_le_bytes().to_vec();
        table.extend(std::iter::repeat_n(1f32.to_le_bytes(), CONTEXT_SLOTS).flatten());
        table.extend(std::iter::repeat_n(1f32.to_le_bytes(), WORDS.len()).flatten());
        table.extend(rows.iter().flatten().map(|value| *value as u8));

        ContextRanker::from_parts(
            &WORDS.map(String::from),
            table.into(),
            ContextRankerConfig {
                dimensions: 2,
                spelling_weight,
            },
        )
    }

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    fn candidates() -> [(String, i32); 2] {
        [("file".to_string(), 0), ("five".to_string(), 0)]
    }

    #[test]
    fn the_context_picks_the_best_fit() {
        assert_eq!(
            ranker(0.0).rank(&words("one two"), &[], &candidates()),
            [1, 0]
        );
    }

    #[test]
    fn the_right_side_counts() {
        assert_eq!(ranker(0.0).rank(&[], &words("two"), &candidates()), [1, 0]);
    }

    #[test]
    fn a_large_spelling_penalty_beats_the_context() {
        let candidates = [("file".to_string(), 0), ("five".to_string(), 50)];

        assert_eq!(ranker(1.0).rank(&words("one"), &[], &candidates), [0, 1]);
    }

    #[test]
    fn no_known_word_keeps_the_order() {
        let ranker = ranker(0.0);

        assert_eq!(ranker.rank(&[], &[], &candidates()), [0, 1]);
        assert_eq!(ranker.rank(&words("qqq"), &[], &candidates()), [0, 1]);
    }

    #[test]
    fn words_outside_the_radius_do_not_count() {
        let mut left = words("one");
        left.extend(words("qqq").into_iter().cycle().take(CONTEXT_RADIUS));

        assert_eq!(ranker(0.0).rank(&left, &[], &candidates()), [0, 1]);
    }

    #[test]
    fn saved_files_load_again() {
        let dir = std::env::temp_dir().join(format!("context-ranker-{}", std::process::id()));
        ranker(0.25).save_to(&dir);

        let loaded = ContextRanker::load_from_bytes(
            std::fs::read(dir.join("table.bin")).unwrap(),
            std::fs::read(dir.join("vocab.json")).unwrap(),
            std::fs::read(dir.join("config.json")).unwrap(),
        );
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(loaded.config().spelling_weight, 0.25);
        assert_eq!(loaded.rank(&words("one"), &[], &candidates()), [1, 0]);
    }
}
