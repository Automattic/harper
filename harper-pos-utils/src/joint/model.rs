//! Burn model: multi-width char-CNN + suffix embedding + BiLSTM encoder with
//! UPOS-tagger and NP-chunker heads.

use crate::joint::TAG_CLASSES;
use burn::{
    module::Module,
    nn::{
        BiLstm, BiLstmConfig, Embedding, EmbeddingConfig, Linear, LinearConfig, PaddingConfig1d,
        conv::{Conv1d, Conv1dConfig},
    },
    tensor::{Int, Tensor, backend::Backend},
};

/// Odd kernel widths for the parallel char convolutions (Same padding needs odd).
const KERNELS: [usize; 2] = [3, 5];

#[derive(Module, Debug)]
pub struct JointModel<B: Backend> {
    char_embed: Embedding<B>,
    conv_a: Conv1d<B>, // kernel KERNELS[0]
    conv_b: Conv1d<B>, // kernel KERNELS[1]
    /// Word-final morphology signal, anchored at the suffix.
    suffix_embed: Embedding<B>,
    word_lstm: BiLstm<B>,
    tag_head: Linear<B>,
    chunk_head: Linear<B>,
}

impl<B: Backend> JointModel<B> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        char_vocab: usize,
        char_dim: usize,
        conv_channels: usize,
        hidden: usize,
        suffix_vocab: usize,
        suffix_dim: usize,
        device: &B::Device,
    ) -> Self {
        let conv = |k: usize| {
            Conv1dConfig::new(char_dim, conv_channels, k)
                .with_padding(PaddingConfig1d::Same)
                .init(device)
        };
        let lstm_in = KERNELS.len() * conv_channels + suffix_dim;
        Self {
            char_embed: EmbeddingConfig::new(char_vocab, char_dim).init(device),
            conv_a: conv(KERNELS[0]),
            conv_b: conv(KERNELS[1]),
            suffix_embed: EmbeddingConfig::new(suffix_vocab, suffix_dim).init(device),
            word_lstm: BiLstmConfig::new(lstm_in, hidden, true).init(device),
            tag_head: LinearConfig::new(hidden * 2, TAG_CLASSES).init(device),
            chunk_head: LinearConfig::new(hidden * 2, 1).init(device),
        }
    }

    /// `char_ids`: `[batch*max_sent, max_word_len]` Int.
    /// `suffix_ids`: `[batch, max_sent]` Int.
    /// Returns `(tag_logits [batch, max_sent, TAG_CLASSES], chunk_logits [batch, max_sent])`.
    pub fn forward(
        &self,
        char_ids: Tensor<B, 2, Int>,
        suffix_ids: Tensor<B, 2, Int>,
        batch: usize,
        max_sent: usize,
    ) -> (Tensor<B, 3>, Tensor<B, 2>) {
        // [batch*max_sent, max_word_len, char_dim] -> [N, char_dim, L] for Conv1d.
        let embedded = self.char_embed.forward(char_ids).swap_dims(1, 2);
        // Each conv: [N, conv_channels, L] -> global max-pool over L -> [N, conv_channels].
        let pool = |c: Tensor<B, 3>| c.max_dim(2).squeeze_dim::<2>(2);
        let a = pool(self.conv_a.forward(embedded.clone()));
        let b = pool(self.conv_b.forward(embedded));
        // concat n-gram features -> [N, 2*conv_channels]
        let word_vecs = Tensor::cat(vec![a, b], 1);
        let width = word_vecs.dims()[1];
        let char_feats = word_vecs.reshape([batch, max_sent, width]);
        // suffix channel: [batch, max_sent, suffix_dim]
        let suffix_feats = self.suffix_embed.forward(suffix_ids);
        let combined = Tensor::cat(vec![char_feats, suffix_feats], 2);
        let (ctx, _) = self.word_lstm.forward(combined, None);
        let tag_logits = self.tag_head.forward(ctx.clone());
        let chunk_logits = self.chunk_head.forward(ctx).squeeze_dim::<2>(2);
        (tag_logits, chunk_logits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint::TAG_CLASSES;
    use burn_ndarray::{NdArray, NdArrayDevice};

    #[test]
    fn forward_produces_two_heads_with_right_shapes() {
        let device = NdArrayDevice::Cpu;
        // new(char_vocab, char_dim, conv_channels, hidden, suffix_vocab, suffix_dim, device)
        let model = JointModel::<NdArray>::new(10, 8, 16, 12, 20, 4, &device);
        let (batch, max_sent, max_word) = (2usize, 3usize, 4usize);
        let char_data: Vec<i32> = (0..(batch * max_sent * max_word) as i32)
            .map(|x| x % 10)
            .collect();
        let char_ids = burn::tensor::Tensor::<NdArray, 1, burn::tensor::Int>::from_data(
            burn::tensor::TensorData::from(char_data.as_slice()),
            &device,
        )
        .reshape([batch * max_sent, max_word]);
        let suffix_data: Vec<i32> = (0..(batch * max_sent) as i32).map(|x| x % 20).collect();
        let suffix_ids = burn::tensor::Tensor::<NdArray, 1, burn::tensor::Int>::from_data(
            burn::tensor::TensorData::from(suffix_data.as_slice()),
            &device,
        )
        .reshape([batch, max_sent]);
        let (tag_logits, chunk_logits) = model.forward(char_ids, suffix_ids, batch, max_sent);
        assert_eq!(tag_logits.dims(), [batch, max_sent, TAG_CLASSES]);
        assert_eq!(chunk_logits.dims(), [batch, max_sent]);
    }
}
