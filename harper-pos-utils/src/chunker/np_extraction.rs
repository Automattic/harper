use std::collections::VecDeque;

use hashbrown::HashSet;
use rs_conllu::{Sentence, Token, TokenID, UPOS};

pub fn locate_noun_phrases_in_sent(sent: &Sentence) -> Vec<HashSet<usize>> {
    let mut found_noun_phrases = Vec::new();

    for (i, token) in sent.tokens.iter().enumerate() {
        if token.upos.is_some_and(is_root_upos) {
            let noun_phrase = locate_noun_phrase_with_head_at(i, sent);

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

fn locate_noun_phrase_with_head_at(head_index: usize, sent: &Sentence) -> HashSet<usize> {
    let mut children = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(head_index);

    while let Some(c_i) = queue.pop_front() {
        if children.contains(&c_i) {
            continue;
        }

        let tok = &sent.tokens[c_i];

        if is_noun_phrase_constituent(tok) || tok.upos.is_some_and(is_root_upos) {
            children.insert(c_i);
            queue.extend(get_children(sent, c_i));
        }
    }

    children
}

fn is_root_upos(upos: UPOS) -> bool {
    use UPOS::*;
    matches!(upos, NOUN | PROPN | PRON)
}

/// Get the indices of the children of a given node.
fn get_children(sent: &Sentence, of_node: usize) -> Vec<usize> {
    let mut children = Vec::new();

    for (index, token) in sent.tokens.iter().enumerate() {
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
