//! Batch construction: tokenises sentences into char-id tensors with padding masks.

use crate::joint::char_vocab::CharVocab;
use crate::joint::suffix_vocab::SuffixVocab;
use crate::joint::{CHAR_PAD, TAG_PAD_CLASS};
use crate::UPOS;
use burn::tensor::{Int, Tensor, TensorData, backend::Backend};

pub struct JointBatch {
    pub char_buf: Vec<i32>,  // [batch * max_sent * max_word]
    pub suffix_buf: Vec<i32>, // [batch * max_sent]
    pub tag_buf: Vec<i32>,   // [batch * max_sent]
    pub np_buf: Vec<f32>,    // [batch * max_sent]
    pub mask_buf: Vec<f32>,  // [batch * max_sent]
    pub batch: usize,
    pub max_sent: usize,
    pub max_word: usize,
    pub real_tokens: usize,
}

impl JointBatch {
    pub fn build(
        sents: &[Vec<String>],
        tags: &[Vec<Option<UPOS>>],
        np: &[Vec<bool>],
        vocab: &CharVocab,
        suffix_vocab: &SuffixVocab,
        max_word: usize,
    ) -> Self {
        let batch = sents.len();
        let max_sent = sents.iter().map(Vec::len).max().unwrap_or(0).max(1);

        let mut char_buf = vec![CHAR_PAD as i32; batch * max_sent * max_word];
        let mut suffix_buf = vec![crate::joint::suffix_vocab::SUFFIX_UNK; batch * max_sent];
        let mut tag_buf = vec![TAG_PAD_CLASS as i32; batch * max_sent];
        let mut np_buf = vec![0f32; batch * max_sent];
        let mut mask_buf = vec![0f32; batch * max_sent];
        let mut real_tokens = 0usize;

        for (si, sent) in sents.iter().enumerate() {
            for (wi, word) in sent.iter().enumerate() {
                let tok = si * max_sent + wi;
                let enc = vocab.encode_word(word, max_word);
                char_buf[tok * max_word..(tok + 1) * max_word].copy_from_slice(&enc);
                suffix_buf[tok] = suffix_vocab.encode_word(word);
                if let Some(Some(t)) = tags[si].get(wi) {
                    tag_buf[tok] = *t as i32; // 0-based class (ADJ..VERB == 0..15)
                }
                np_buf[tok] = if np[si].get(wi).copied().unwrap_or(false) { 1.0 } else { 0.0 };
                mask_buf[tok] = 1.0;
                real_tokens += 1;
            }
        }

        Self { char_buf, suffix_buf, tag_buf, np_buf, mask_buf, batch, max_sent, max_word, real_tokens }
    }

    pub fn char_ids<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2, Int> {
        Tensor::<B, 1, Int>::from_data(TensorData::from(self.char_buf.as_slice()), device)
            .reshape([self.batch * self.max_sent, self.max_word])
    }

    pub fn suffix_ids<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2, Int> {
        Tensor::<B, 1, Int>::from_data(TensorData::from(self.suffix_buf.as_slice()), device)
            .reshape([self.batch, self.max_sent])
    }

    pub fn flat_tags<B: Backend>(&self, device: &B::Device) -> Tensor<B, 1, Int> {
        Tensor::<B, 1, Int>::from_data(TensorData::from(self.tag_buf.as_slice()), device)
    }

    pub fn np<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2> {
        Tensor::<B, 1>::from_data(TensorData::from(self.np_buf.as_slice()), device)
            .reshape([self.batch, self.max_sent])
    }

    pub fn mask<B: Backend>(&self, device: &B::Device) -> Tensor<B, 2> {
        Tensor::<B, 1>::from_data(TensorData::from(self.mask_buf.as_slice()), device)
            .reshape([self.batch, self.max_sent])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint::{CHAR_PAD, TAG_PAD_CLASS};
    use crate::joint::char_vocab::CharVocab;
    use crate::joint::suffix_vocab::SuffixVocab;
    use crate::UPOS;

    #[test]
    fn pads_words_and_tokens_with_sentinels() {
        let sents = vec![
            vec!["a".to_string(), "bb".to_string()], // len 2
            vec!["c".to_string()],                   // len 1 -> padded to 2
        ];
        let tags = vec![
            vec![Some(UPOS::DET), Some(UPOS::NOUN)],
            vec![Some(UPOS::VERB)],
        ];
        let np = vec![vec![true, true], vec![false]];
        let vocab = CharVocab::build(&sents);
        let suffix_vocab = SuffixVocab::build(&sents, 3, 100);
        let b = JointBatch::build(&sents, &tags, &np, &vocab, &suffix_vocab, 3);

        assert_eq!(b.batch, 2);
        assert_eq!(b.max_sent, 2);
        assert_eq!(b.max_word, 3);
        assert_eq!(b.real_tokens, 3); // 2 + 1

        // tag_buf laid out [batch * max_sent]; sentence 2's padded slot = PAD class.
        // sentence 0: DET, NOUN ; sentence 1: VERB, PAD
        assert_eq!(
            b.tag_buf,
            vec![
                UPOS::DET as i32,
                UPOS::NOUN as i32,
                UPOS::VERB as i32,
                TAG_PAD_CLASS as i32,
            ]
        );
        // mask 1 for real, 0 for the padded 4th slot.
        assert_eq!(b.mask_buf, vec![1.0, 1.0, 1.0, 0.0]);
        // char_buf for the padded token slot is all CHAR_PAD.
        let last = &b.char_buf[(3 * b.max_word)..(4 * b.max_word)];
        assert_eq!(last, &[CHAR_PAD as i32, CHAR_PAD as i32, CHAR_PAD as i32]);

        // one suffix id per token slot; padded slot (index 3) is UNK (0).
        assert_eq!(b.suffix_buf.len(), b.batch * b.max_sent);
        assert_eq!(b.suffix_buf[3], crate::joint::suffix_vocab::SUFFIX_UNK);
    }
}
