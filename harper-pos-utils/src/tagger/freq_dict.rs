use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::Tagger;
use crate::upos::UPOS;

/// A mapping between words (normalized to lowercase) and their most common UPOS tag.
/// Can be used as a minimally accurate [`Tagger`].
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct FreqDict {
    pub mapping: HashMap<String, UPOS>,
}

impl FreqDict {
    pub fn get(&self, word: &str) -> Option<UPOS> {
        let word_lower = word.to_lowercase();
        self.mapping.get(word_lower.as_str()).copied()
    }
}

impl Tagger for FreqDict {
    fn tag_sentence(&self, sentence: &[String]) -> Vec<Option<UPOS>> {
        let mut tags = Vec::new();

        for word in sentence {
            let tag = self.get(word);
            tags.push(tag);
        }

        tags
    }
}
