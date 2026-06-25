//! Character vocabulary: maps chars to integer ids for the char-CNN encoder.

use crate::joint::{CHAR_PAD, CHAR_UNK};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharVocab {
    map: HashMap<char, usize>,
}

impl CharVocab {
    pub fn build(sentences: &[Vec<String>]) -> Self {
        let mut map: HashMap<char, usize> = HashMap::new();
        // Reserve PAD/UNK by inserting sentinel chars that can never be real
        // input (`encode_word` only ever looks up real chars). We only need the
        // ids reserved, so seed `len()` past them.
        // Real chars start at id = next free slot (>= 2).
        let mut next = CHAR_UNK + 1; // first real id (2)
        for sent in sentences {
            for tok in sent {
                for ch in tok.chars() {
                    map.entry(ch).or_insert_with(|| {
                        let id = next;
                        next += 1;
                        id
                    });
                }
            }
        }
        Self { map }
    }

    /// Total id space: 2 reserved (PAD, UNK) + distinct chars.
    pub fn len(&self) -> usize {
        self.map.len() + 2
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn encode_word(&self, word: &str, max_word_len: usize) -> Vec<i32> {
        let mut out = Vec::with_capacity(max_word_len);
        for ch in word.chars().take(max_word_len) {
            let id = *self.map.get(&ch).unwrap_or(&CHAR_UNK);
            out.push(id as i32);
        }
        while out.len() < max_word_len {
            out.push(CHAR_PAD as i32);
        }
        out
    }

    pub fn to_json(&self) -> String {
        // Ordered (BTreeMap by char) + pretty-printed so the committed artifact
        // has a stable, diff-friendly layout. The ids are the values; key order
        // is cosmetic and does not affect decoding.
        let ordered: BTreeMap<char, usize> = self.map.iter().map(|(&k, &v)| (k, v)).collect();
        serde_json::to_string_pretty(&ordered).expect("serialize char vocab")
    }

    pub fn from_json(s: &str) -> Self {
        let map: HashMap<char, usize> = serde_json::from_str(s).expect("parse char vocab");
        Self { map }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_and_unk_reserved_then_chars_assigned() {
        let sents = vec![vec!["ab".to_string()], vec!["bc".to_string()]];
        let v = CharVocab::build(&sents);
        // 0/1 reserved; 'a','b','c' take 2,3,4 in first-sight order.
        assert_eq!(v.len(), 5);
        let enc = v.encode_word("abc", 4);
        // a,b,c then one pad
        assert_eq!(enc, vec![2, 3, 4, CHAR_PAD as i32]);
    }

    #[test]
    fn unknown_char_maps_to_unk_and_truncates() {
        let sents = vec![vec!["ab".to_string()]];
        let v = CharVocab::build(&sents);
        // 'z' unseen -> UNK(1); length truncated to 2.
        assert_eq!(v.encode_word("zb", 2), vec![CHAR_UNK as i32, 3]);
        assert_eq!(v.encode_word("abcd", 2), vec![2, 3]); // truncated
    }

    #[test]
    fn json_round_trips() {
        let sents = vec![vec!["hi".to_string()]];
        let v = CharVocab::build(&sents);
        let v2 = CharVocab::from_json(&v.to_json());
        assert_eq!(v.encode_word("hi", 2), v2.encode_word("hi", 2));
        assert_eq!(v.len(), v2.len());
    }
}
