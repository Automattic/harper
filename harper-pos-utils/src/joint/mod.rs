//! Joint char-based UPOS tagger + NP chunker. One model produces both
//! per-token outputs from a shared char-CNN + BiLSTM encoder.

pub mod char_vocab;
pub mod suffix_vocab;
pub mod model;
pub mod batch;
pub mod runtime;
#[cfg(feature = "training")]
pub mod eval;
#[cfg(feature = "training")]
pub mod train;
#[cfg(feature = "training")]
pub mod inject;

/// Char id reserved for word padding (never a real char).
pub const CHAR_PAD: usize = 0;
/// Char id for any char outside the trained char vocab.
pub const CHAR_UNK: usize = 1;
/// Cross-entropy target class for padded token slots (one past the 16 real
/// UPOS classes `0..=15`, where class `i` is the `i`-th `UPOS` variant in
/// declaration order, ADJ..=VERB). The `UPOS` enum has 16 variants (no `X`).
/// Masked out via `CrossEntropyLossConfig::with_pad_tokens`.
pub const TAG_PAD_CLASS: usize = 16;
/// Number of tag-head outputs: 16 real UPOS classes + 1 PAD class.
pub const TAG_CLASSES: usize = 17;
