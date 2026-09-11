use crate::UPOS;
use hashbrown::HashSet;
use rs_conllu::{Sentence, Token, TokenID, UPOS as ConlluUpos, parse_file};
use std::collections::VecDeque;
use std::{fs::File, path::Path};

/// The sentence's syntactic words only: plain `TokenID::Single` rows, with
/// multiword ranges and ellipsis nodes dropped. In this view a token's
/// position equals its CoNLL-U id − 1 — the alignment the head-link walk in
/// noun-phrase extraction depends on. Feeding raw `sent.tokens` to that walk
/// shifts every span in a sentence containing multiword tokens.
pub fn syntactic_words(sent: &Sentence) -> Vec<Token> {
    sent.tokens
        .iter()
        .filter(|t| matches!(t.id, TokenID::Single(_)))
        .cloned()
        .collect()
}

/// Produce an iterator over the sentences in a `.conllu` file.
/// Will panic on error, so this should not be used outside of training.
pub fn iter_sentences_in_conllu(path: impl AsRef<Path>) -> impl Iterator<Item = Sentence> {
    let file = File::open(path).unwrap();
    let doc = parse_file(file);

    doc.map(|v| v.unwrap())
}

/// Contraction suffixes that CoNLL-U splits onto their own row (`do` + `n't`).
/// We merge them back onto the preceding token. Single source of truth — every
/// extractor (tagger, Brill chunker, burn chunker) shares this list so they
/// can't drift apart.
const CONTRACTION_SUFFIXES: [&str; 8] = ["sn't", "n't", "'ll", "'ve", "'re", "'d", "'m", "'s"];

fn is_contraction_suffix(form: &str) -> bool {
    CONTRACTION_SUFFIXES.contains(&form)
}

/// Output of the shared sentence walk: the merged tokens/tags plus, for each
/// output token, the contiguous range of syntactic-word indices it absorbed,
/// as `(first, count)`. The span lets noun-phrase flags — which are computed on
/// the syntactic words — be folded back onto fused surface forms: a fused token
/// is in a noun phrase if any of its components is.
struct SentenceWalk {
    tokens: Vec<String>,
    tags: Vec<Option<UPOS>>,
    syn_spans: Vec<(usize, usize)>,
}

/// The one place that turns a CoNLL-U sentence into Harper's tokenizer shape.
///
/// - Multiword tokens (`1-2 im` → `1 in` + `2 dem`) collapse to the surface
///   form (`im`), tagged like their first component; the component rows and
///   ellipsis nodes (`5.1`) are dropped.
/// - Contraction suffixes (`n't`, `'s`, …) split into their own rows are
///   merged back onto the preceding token, keeping that token's tag.
///
/// Every absorbed `TokenID::Single` row advances the syntactic-word counter, so
/// `syn_spans` stays aligned with [`syntactic_words`] for noun-phrase folding.
fn walk_sentence(sent: &Sentence) -> SentenceWalk {
    let mut tokens: Vec<String> = Vec::new();
    let mut tags: Vec<Option<UPOS>> = Vec::new();
    let mut syn_spans: Vec<(usize, usize)> = Vec::new();

    // While inside a multiword token: index of the surface form whose tag is
    // still unset, and the last component id the range covers.
    let mut mwt: Option<(usize, usize)> = None;
    // Running count of syntactic words (`TokenID::Single` rows) seen so far —
    // equals the position of the next single token in `syntactic_words`.
    let mut syn_idx = 0usize;

    for token in &sent.tokens {
        match token.id {
            TokenID::Range(_, end) => {
                tokens.push(token.form.clone());
                tags.push(None);
                // Components fill this span as they are consumed below.
                syn_spans.push((syn_idx, 0));
                mwt = Some((tokens.len() - 1, end));
                continue;
            }
            TokenID::Empty(_, _) => continue,
            TokenID::Single(id) => {
                if let Some((fill, end)) = mwt {
                    if id <= end {
                        // First component with a usable UPOS tags the surface.
                        if tags[fill].is_none() {
                            tags[fill] = token.upos.and_then(UPOS::from_conllu);
                        }
                        syn_spans[fill].1 += 1;
                        syn_idx += 1;
                        continue;
                    }
                    mwt = None;
                }
            }
        }

        let form = token.form.clone();
        if let Some(last) = tokens.last_mut() {
            if is_contraction_suffix(&form) {
                last.push_str(&form);
                // The suffix is a syntactic word fused into the previous token.
                syn_spans.last_mut().unwrap().1 += 1;
                syn_idx += 1;
                continue;
            }
        }
        tokens.push(form);
        tags.push(token.upos.and_then(UPOS::from_conllu));
        syn_spans.push((syn_idx, 1));
        syn_idx += 1;
    }

    SentenceWalk {
        tokens,
        tags,
        syn_spans,
    }
}

/// Convert a CoNLL-U sentence into the `(token, tag)` shape Harper's tokenizer
/// produces. Used by the Brill tagger, which has no use for noun-phrase labels.
pub fn sentence_to_training_pair(sent: &Sentence) -> (Vec<String>, Vec<Option<UPOS>>) {
    let walk = walk_sentence(sent);
    (walk.tokens, walk.tags)
}

/// A training example for the chunkers: the merged tokens and tags (identical
/// to [`sentence_to_training_pair`] by construction) plus a noun-phrase mask
/// aligned to `tokens`.
#[cfg(feature = "training")]
pub struct TrainingRecord {
    pub tokens: Vec<String>,
    pub tags: Vec<Option<UPOS>>,
    pub np_mask: Vec<bool>,
}

/// Like [`sentence_to_training_pair`], but also computes the noun-phrase mask.
///
/// Noun phrases are located on the syntactic words only (where a token's
/// position equals its CoNLL-U id − 1); the resulting per-syntactic-word flags
/// are folded back onto the merged tokens via the walk's `syn_spans`. This is
/// the correct, span-aligned source of truth for both chunkers — feeding raw
/// `sent.tokens` to the noun-phrase walk shifts every span in a sentence
/// containing multiword tokens.
#[cfg(feature = "training")]
pub fn sentence_to_training_record(sent: &Sentence) -> TrainingRecord {
    let walk = walk_sentence(sent);

    let syn_tokens = syntactic_words(sent);
    let mut np_flags = vec![false; syn_tokens.len()];
    for span in locate_noun_phrases_in_words(&syn_tokens) {
        for i in span {
            np_flags[i] = true;
        }
    }

    let np_mask = walk
        .syn_spans
        .iter()
        .map(|&(start, count)| (start..start + count).any(|i| np_flags[i]))
        .collect();

    TrainingRecord {
        tokens: walk.tokens,
        tags: walk.tags,
        np_mask,
    }
}

/// Extract the training record of every sentence across `files` in one pass:
/// the merged tokens, tags and NP mask per sentence (see
/// [`sentence_to_training_record`]), returned as parallel `(sents, tags, labs)`
/// vectors — the shape the chunkers' batchers consume.
///
/// Vocabulary construction is intentionally left to the caller: it owns its id
/// conventions (reserved `<PAD>`/`<UNK>` slots, …) and decides which split is
/// allowed to grow the vocab — typically only the training split, so held-out
/// words resolve to `<UNK>` at evaluation instead of leaking in.
#[cfg(feature = "training")]
#[allow(clippy::type_complexity)]
pub fn extract_records_from_files(
    files: &[impl AsRef<Path>],
) -> (Vec<Vec<String>>, Vec<Vec<Option<UPOS>>>, Vec<Vec<bool>>) {
    let mut sents = Vec::new();
    let mut tags = Vec::new();
    let mut labs = Vec::new();

    for file in files {
        for sent in iter_sentences_in_conllu(file) {
            let TrainingRecord {
                tokens,
                tags: sent_tags,
                np_mask,
            } = sentence_to_training_record(&sent);

            sents.push(tokens);
            tags.push(sent_tags);
            labs.push(np_mask);
        }
    }

    (sents, tags, labs)
}

/// Like [`extract_records_from_files`], but for consumers that only need the
/// merged tokens and tags — no noun-phrase labels (e.g. the Brill tagger).
/// Returns one `(tokens, tags)` pair per sentence, in order. Skipping the
/// noun-phrase pass keeps it cheaper than [`extract_records_from_files`] for
/// callers that would only discard the mask.
#[cfg(feature = "training")]
pub fn extract_pairs_from_files(
    files: &[impl AsRef<Path>],
) -> Vec<(Vec<String>, Vec<Option<UPOS>>)> {
    files
        .iter()
        .flat_map(iter_sentences_in_conllu)
        .map(|sent| sentence_to_training_pair(&sent))
        .collect()
}

/// Locate noun phrases among `tokens`, returned as sets of positions.
///
/// `tokens` must contain only syntactic words (plain `TokenID::Single` rows)
/// so that a token's position equals its CoNLL-U id − 1: the head-link walk
/// below resolves head *ids* by position. Passing raw `sent.tokens` shifts
/// every span in a sentence containing multiword-token ranges or ellipsis
/// nodes — use [`syntactic_words`] first.
fn locate_noun_phrases_in_words(tokens: &[Token]) -> Vec<HashSet<usize>> {
    let mut found_noun_phrases = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        if token.upos.is_some_and(is_root_upos) {
            let noun_phrase = locate_noun_phrase_with_head_at(i, tokens);

            found_noun_phrases.push(noun_phrase);
        }
    }

    found_noun_phrases.retain(is_contiguous);

    reduce_to_maximal_nonoverlapping(found_noun_phrases)
}

fn is_contiguous(indices: &HashSet<usize>) -> bool {
    if indices.is_empty() {
        return false;
    }
    let lo = *indices.iter().min().unwrap();
    let hi = *indices.iter().max().unwrap();
    hi - lo + 1 == indices.len()
}

fn reduce_to_maximal_nonoverlapping(mut phrases: Vec<HashSet<usize>>) -> Vec<HashSet<usize>> {
    phrases.sort_by_key(|s| usize::MAX - s.len());
    let mut selected = Vec::new();
    let mut occupied = HashSet::new();

    for p in phrases {
        if p.is_disjoint(&occupied) {
            occupied.extend(&p);
            selected.push(p);
        }
    }

    selected
}

fn locate_noun_phrase_with_head_at(head_index: usize, tokens: &[Token]) -> HashSet<usize> {
    let mut children = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(head_index);

    while let Some(c_i) = queue.pop_front() {
        if children.contains(&c_i) {
            continue;
        }

        let tok = &tokens[c_i];

        if is_noun_phrase_constituent(tok) || tok.upos.is_some_and(is_root_upos) {
            children.insert(c_i);
            queue.extend(get_children(tokens, c_i));
        }
    }

    children
}

fn is_root_upos(upos: ConlluUpos) -> bool {
    use ConlluUpos::*;
    matches!(upos, NOUN | PROPN | PRON)
}

/// Get the indices of the children of a given node.
fn get_children(tokens: &[Token], of_node: usize) -> Vec<usize> {
    let mut children = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if index == of_node {
            continue;
        }

        if let Some(head) = token.head {
            let is_child = match head {
                TokenID::Single(i) => i != 0 && i - 1 == of_node,
                TokenID::Range(start, end) => (start - 1..end - 1).contains(&of_node),
                TokenID::Empty(_, _) => false,
            };

            if is_child {
                children.push(index)
            }
        }
    }

    children
}

fn is_noun_phrase_constituent(token: &Token) -> bool {
    let Some(ref deprel) = token.deprel else {
        return false;
    };

    matches!(
        deprel.as_str(),
        "det" | "amod" | "nummod" | "compound" | "fixed" | "flat" | "acl" | "aux:pass"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rs_conllu::parse_sentence;

    // German "Im Garten steht ...": `1-2 Im` is a multiword token expanding to
    // `In` + `dem`. The surface form `Im` should survive, tagged like its first
    // component (`In`, ADP), and the component rows must not leak as tokens.
    const MWT_SENTENCE: &str = concat!(
        "1-2\tIm\t_\t_\t_\t_\t_\t_\t_\t_\n",
        "1\tIn\tin\tADP\t_\t_\t3\tcase\t_\t_\n",
        "2\tdem\tder\tDET\t_\t_\t3\tdet\t_\t_\n",
        "3\tGarten\tGarten\tNOUN\t_\t_\t0\troot\t_\t_\n",
        "4\tsteht\tstehen\tVERB\t_\t_\t3\tacl\t_\t_\n",
    );

    // English "I do n't know": the contraction suffix `n't` is its own row and
    // must merge back onto the preceding token, keeping that token's tag.
    const CONTRACTION_SENTENCE: &str = concat!(
        "1\tI\tI\tPRON\t_\t_\t4\tnsubj\t_\t_\n",
        "2\tdo\tdo\tAUX\t_\t_\t4\taux\t_\t_\n",
        "3\tn't\tnot\tPART\t_\t_\t4\tadvmod\t_\t_\n",
        "4\tknow\tknow\tVERB\t_\t_\t0\troot\t_\t_\n",
    );

    #[test]
    fn multiword_token_collapses_to_surface_form() {
        let sent = parse_sentence(MWT_SENTENCE).unwrap();
        let (toks, tags) = sentence_to_training_pair(&sent);

        assert_eq!(toks, vec!["Im", "Garten", "steht"]);
        assert_eq!(tags.len(), toks.len());
        // Surface form is tagged like its first component (`In` = ADP).
        assert_eq!(tags[0], Some(UPOS::ADP));
        assert_eq!(tags[1], Some(UPOS::NOUN));
        assert_eq!(tags[2], Some(UPOS::VERB));
    }

    #[test]
    fn contraction_suffix_merges_onto_previous_token() {
        let sent = parse_sentence(CONTRACTION_SENTENCE).unwrap();
        let (toks, tags) = sentence_to_training_pair(&sent);

        assert_eq!(toks, vec!["I", "don't", "know"]);
        // The merged token keeps `do`'s tag; `n't`'s PART tag is dropped.
        assert_eq!(tags.len(), toks.len());
    }

    // The span-shift bug produced NP masks that were misaligned with the merged
    // tokens whenever a sentence contained a multiword token. The shared record
    // must always return a mask the same length as `tokens`, and identical
    // tokens/tags to the pair extractor.
    #[cfg(feature = "training")]
    #[test]
    fn record_matches_pair_and_np_mask_is_aligned() {
        for raw in [MWT_SENTENCE, CONTRACTION_SENTENCE] {
            let sent = parse_sentence(raw).unwrap();
            let (toks, tags) = sentence_to_training_pair(&sent);
            let record = sentence_to_training_record(&sent);

            assert_eq!(record.tokens, toks);
            assert_eq!(record.tags, tags);
            assert_eq!(
                record.np_mask.len(),
                record.tokens.len(),
                "NP mask must stay aligned to the merged tokens"
            );
        }
    }
}
