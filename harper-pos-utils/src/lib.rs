mod annotator;
mod chunker;
#[cfg(feature = "training")]
pub mod conllu_utils;
pub mod joint;
mod patch_criteria;
mod tagger;
mod upos;
#[cfg(feature = "training")]
mod word_counter;

pub use annotator::{Annotator, TagSet};
pub use chunker::{
    BrillChunker, BurnChunker, BurnChunkerCpu, CachedChunker, Chunker, UPOSFreqDict,
};
pub use tagger::{BrillTagger, FreqDict, FreqDictBuilder, Tagger};
pub use upos::{UPOS, UPOSIter};

/// Serialize `value` as pretty JSON for committed model artifacts: indentation
/// is tabs and the output ends with a trailing newline, matching the repo's
/// Biome formatter (`indentStyle: tab`) so artifacts stay format-clean and
/// don't churn on reformat.
///
/// Deterministic key ordering is the caller's responsibility — serialize from a
/// `BTreeMap` (or other sorted structure). This stays correct regardless of
/// whether serde_json's `preserve_order` feature is enabled anywhere in the
/// workspace, since the keys are already sorted before they reach serde.
pub(crate) fn to_json_tabs<T: serde::Serialize>(value: &T) -> String {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value.serialize(&mut ser).expect("serialize json");
    // PrettyFormatter omits the trailing newline that Biome enforces.
    buf.push(b'\n');
    String::from_utf8(buf).expect("json is valid utf-8")
}
pub(crate) fn map_to_json_tabs<K, V, I>(entries: I) -> String
where
    K: Ord + serde::Serialize,
    V: serde::Serialize,
    I: IntoIterator<Item = (K, V)>,
{
    let ordered: std::collections::BTreeMap<K, V> = entries.into_iter().collect();
    to_json_tabs(&ordered)
}
