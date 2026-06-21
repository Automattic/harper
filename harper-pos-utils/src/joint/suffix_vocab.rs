//! Suffix vocabulary: maps each word's lowercased last-k-character suffix to an
//! integer id for the morphology-aware encoder. Id 0 = UNK (unseen / padded).

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Reserved id for an unseen suffix or a padded token slot.
pub const SUFFIX_UNK: i32 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuffixVocab {
    k: usize,
    map: HashMap<String, usize>,
}

impl SuffixVocab {
    /// The lowercased last-`k`-char suffix of `word` (the whole word if shorter).
    pub fn suffix_of(word: &str, k: usize) -> String {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let start = chars.len().saturating_sub(k);
        chars[start..].iter().collect()
    }

    /// Build from `sentences`; keep the `cap` most frequent suffixes (ties
    /// broken alphabetically for determinism). Real ids start at 1 (0 = UNK).
    pub fn build(sentences: &[Vec<String>], k: usize, cap: usize) -> Self {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for sent in sentences {
            for tok in sent {
                *counts.entry(Self::suffix_of(tok, k)).or_insert(0) += 1;
            }
        }
        let mut candidates: Vec<(String, usize)> = counts.into_iter().collect();
        candidates.sort_unstable_by(|(sa, ca), (sb, cb)| cb.cmp(ca).then(sa.cmp(sb)));
        let mut map: HashMap<String, usize> = HashMap::new();
        for (i, (suf, _)) in candidates.into_iter().take(cap).enumerate() {
            map.insert(suf, 1 + i); // id 0 reserved for UNK
        }
        Self { k, map }
    }

    /// Total id space: 1 reserved (UNK) + distinct suffixes.
    pub fn len(&self) -> usize {
        self.map.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Encode a word to its suffix id (UNK = 0 for unseen suffixes).
    pub fn encode_word(&self, word: &str) -> i32 {
        let suf = Self::suffix_of(word, self.k);
        self.map.get(&suf).map(|&id| id as i32).unwrap_or(SUFFIX_UNK)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize suffix vocab")
    }

    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).expect("parse suffix vocab")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_of_takes_last_k_lowercased() {
        assert_eq!(SuffixVocab::suffix_of("Intricacy", 3), "acy");
        assert_eq!(SuffixVocab::suffix_of("to", 3), "to"); // shorter than k
        assert_eq!(SuffixVocab::suffix_of("ELUSIVE", 3), "ive");
    }

    #[test]
    fn build_assigns_ids_unseen_is_unk() {
        let sents = vec![
            vec!["accuracy".to_string(), "privacy".to_string()], // both -acy
            vec!["massive".to_string()],                          // -ive
        ];
        let v = SuffixVocab::build(&sents, 3, 100);
        // "-acy" (2x) and "-ive" (1x) are real ids >= 1; unseen -> 0.
        assert!(v.encode_word("intricacy") >= 1, "seen suffix -acy gets a real id");
        assert!(v.encode_word("elusive") >= 1, "seen suffix -ive gets a real id");
        assert_eq!(v.encode_word("xyzzq"), SUFFIX_UNK, "unseen suffix -> UNK");
        assert_eq!(v.len(), 3); // 1 reserved + {acy, ive}
    }

    #[test]
    fn json_round_trips() {
        let sents = vec![vec!["running".to_string(), "singing".to_string()]];
        let v = SuffixVocab::build(&sents, 3, 100);
        let v2 = SuffixVocab::from_json(&v.to_json());
        assert_eq!(v.encode_word("walking"), v2.encode_word("walking"));
        assert_eq!(v.len(), v2.len());
    }
}
