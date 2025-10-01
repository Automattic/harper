#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod char_ext;
mod char_string;
mod currency;
pub mod dict_word_metadata;
pub mod dict_word_metadata_orthography;
mod document;
mod edit_distance;
pub mod expr;
mod fat_token;
mod ignored_lints;
pub mod language_detection;
mod lexing;
pub mod linting;
mod mask;
mod number;
pub mod parsers;
pub mod patterns;
mod punctuation;
mod render_markdown;
mod span;
pub mod spell;
mod sync;
mod title_case;
mod token;
mod token_kind;
mod token_string_ext;
mod vec_ext;

use render_markdown::render_markdown;
use std::collections::{BTreeMap, VecDeque};

pub use char_string::{CharString, CharStringExt};
pub use currency::Currency;
pub use dict_word_metadata::{
    AdverbData, ConjunctionData, Degree, DeterminerData, Dialect, DictWordMetadata, NounData,
    PronounData, VerbData, VerbForm,
};
pub use document::Document;
pub use fat_token::{FatStringToken, FatToken};
pub use ignored_lints::{IgnoredLints, LintContext};
use linting::Lint;
pub use mask::{Mask, Masker};
pub use number::{Number, OrdinalSuffix};
pub use punctuation::{Punctuation, Quote};
pub use span::Span;
pub use sync::{LSend, Lrc};
pub use title_case::{make_title_case, make_title_case_str};
pub use token::Token;
pub use token_kind::TokenKind;
pub use token_string_ext::TokenStringExt;
pub use vec_ext::VecExt;

/// Return harper-core version
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// A utility function that removes overlapping lints in a vector,
/// keeping the more important ones.
///
/// Note: this function will change the ordering of the lints.
pub fn remove_overlaps(lints: &mut Vec<Lint>) {
    if lints.len() < 2 {
        return;
    }

    let mut remove_indices = VecDeque::new();
    lints.sort_by_key(|l| (l.span.start, !0 - l.span.end));

    let mut cur = 0;

    for (i, lint) in lints.iter().enumerate() {
        if lint.span.start < cur {
            remove_indices.push_back(i);
            continue;
        }
        cur = lint.span.end;
    }

    lints.remove_indices(remove_indices);
}

/// Remove overlapping lints from a map keyed by rule name, similar to [`remove_overlaps`].
///
/// The map is treated as if all contained lints were in a single flat collection, ensuring the
/// same lint would be kept regardless of whether it originated from `lint` or `organized_lints`.
pub fn remove_overlaps_map<K: Ord>(lint_map: &mut BTreeMap<K, Vec<Lint>>) {
    let total: usize = lint_map.values().map(Vec::len).sum();
    if total < 2 {
        return;
    }

    struct IndexedSpan {
        rule_idx: usize,
        lint_idx: usize,
        start: usize,
        end: usize,
    }

    let mut removal_flags: Vec<Vec<bool>> = lint_map
        .values()
        .map(|lints| vec![false; lints.len()])
        .collect();

    let mut spans = Vec::with_capacity(total);
    for (rule_idx, (_, lints)) in lint_map.iter().enumerate() {
        for (lint_idx, lint) in lints.iter().enumerate() {
            spans.push(IndexedSpan {
                rule_idx,
                lint_idx,
                start: lint.span.start,
                end: lint.span.end,
            });
        }
    }

    spans.sort_by_key(|span| (span.start, usize::MAX - span.end));

    let mut cur = 0;
    for span in spans {
        if span.start < cur {
            removal_flags[span.rule_idx][span.lint_idx] = true;
        } else {
            cur = span.end;
        }
    }

    for (rule_idx, (_, lints)) in lint_map.iter_mut().enumerate() {
        if removal_flags[rule_idx].iter().all(|flag| !*flag) {
            continue;
        }

        let mut idx = 0;
        lints.retain(|_| {
            let remove = removal_flags[rule_idx][idx];
            idx += 1;
            !remove
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::spell::FstDictionary;
    use crate::{
        Dialect, Document,
        linting::{LintGroup, Linter},
        remove_overlaps,
    };

    #[test]
    fn keeps_space_lint() {
        let doc = Document::new_plain_english_curated("Ths  tet");

        let mut linter = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }
}
