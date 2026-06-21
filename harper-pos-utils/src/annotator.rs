use crate::{Chunker, Tagger, UPOS};
use smallvec::SmallVec;

/// Per-token plausible-tag set: the argmax first, then any runner-up above the
/// model's probability floor. Inline up to 4 tags (the common case is 1–3), so a
/// confidently-tagged token needs no heap allocation. Matches harper-core's
/// `DictWordMetadata::pos_tag_topk`, so it flows straight in without re-collecting.
pub type TagSet = SmallVec<[UPOS; 4]>;

/// A model that does **both** part-of-speech tagging and noun-phrase chunking,
/// and can produce every per-token annotation a document needs in a single call.
///
/// The supertrait bound (`Tagger + Chunker`) states the relationship directly:
/// an `Annotator` *is* a tagger and a chunker, plus a combined lookup. Tagging
/// and chunking are always consumed together when annotating a document, and a
/// model that does both jointly (the joint runtime) derives both outputs from
/// one cached forward pass — so callers reach for [`Annotator::annotate`]
/// instead of juggling the two traits and recomputing the argmax to bridge them.
pub trait Annotator: Tagger + Chunker {
    /// For each token in `sentence`, returns its plausible-tag set and whether
    /// it belongs to a noun phrase. The most-likely tag is `tags[i].first()`
    /// (`None` only when the set is empty — the model's top class was padding
    /// and no runner-up cleared the floor).
    fn annotate(&self, sentence: &[String]) -> (Vec<TagSet>, Vec<bool>);
}
