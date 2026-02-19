#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod case;
mod char_ext;
mod char_string;
mod currency;
mod dict_word_metadata;
mod dict_word_metadata_orthography;
mod document;
mod edit_distance;
pub mod expr;
mod fat_token;
mod ignored_lints;
mod irregular_nouns;
mod irregular_verbs;
pub mod language_detection;
mod lexing;
pub mod linting;
mod mask;
mod number;
mod offsets;
pub mod parsers;
pub mod patterns;
mod punctuation;
mod render_markdown;
mod span;
pub mod spell;
mod sync;
mod thesaurus_helper;
mod title_case;
mod token;
mod token_kind;
mod token_string_ext;
mod vec_ext;
pub mod weir;
pub mod weirpack;

use render_markdown::render_markdown;
use std::collections::{BTreeMap, VecDeque};

pub use case::{Case, CaseIterExt};
pub use char_string::{CharString, CharStringExt};
pub use currency::Currency;
pub use dict_word_metadata::{
    AdverbData, ConjunctionData, Degree, DeterminerData, Dialect, DialectFlags, DictWordMetadata,
    NounData, PronounData, VerbData, VerbForm, VerbFormFlags,
};
pub use dict_word_metadata_orthography::{OrthFlags, Orthography};
pub use document::Document;
pub use fat_token::{FatStringToken, FatToken};
pub use ignored_lints::{IgnoredLints, LintContext};
pub use irregular_nouns::IrregularNouns;
pub use irregular_verbs::IrregularVerbs;
use linting::Lint;
pub use mask::{Mask, Masker, RegexMasker};
pub use number::{Number, OrdinalSuffix};
pub use punctuation::{Punctuation, Quote};
pub use span::Span;
pub use sync::{LSend, Lrc};
pub use title_case::{make_title_case, make_title_case_str};
pub use token::Token;
pub use token_kind::TokenKind;
pub use token_string_ext::TokenStringExt;
pub use vec_ext::VecExt;

/// Return `harper-core` version
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
    lints.sort_by_key(|l| l.priority);
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
        priority: u8,
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
                priority: lint.priority,
                rule_idx,
                lint_idx,
                start: lint.span.start,
                end: lint.span.end,
            });
        }
    }

    spans.sort_by_key(|span| span.priority);
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
    use std::hash::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::Arc;

    use itertools::Itertools;
    use quickcheck_macros::quickcheck;

    use crate::linting::Lint;
    use crate::remove_overlaps_map;
    use crate::spell::FstDictionary;
    use crate::{
        Dialect, Document,
        linting::{LintGroup, Linter},
        remove_overlaps,
    };

    #[test]
    fn keeps_space_lint() {
        let doc = Document::new_plain_english_curated("Ths  tet");

        let mut linter =
            LintGroup::new_curated(Arc::new(FstDictionary::curated()), Dialect::American);

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }

    #[quickcheck]
    fn overlap_removals_have_equivalent_behavior(s: String) {
        let doc = Document::new_plain_english_curated(&s);
        let mut linter =
            LintGroup::new_curated(Arc::new(FstDictionary::curated()), Dialect::American);

        let mut lint_map = linter.organized_lints(&doc);
        let mut lint_flat: Vec<_> = lint_map.values().flatten().cloned().collect();

        remove_overlaps_map(&mut lint_map);
        remove_overlaps(&mut lint_flat);

        let post_removal_flat: Vec<_> = lint_map.values().flatten().cloned().collect();

        fn hash_lint(lint: &Lint) -> u64 {
            let mut hasher = DefaultHasher::new();
            lint.hash(&mut hasher);
            hasher.finish()
        }

        // We want to ignore ordering, so let us hash these first and sort them.
        let lint_flat_hashes: Vec<_> = lint_flat.iter().map(hash_lint).sorted().collect();
        let post_removal_flat_hashes: Vec<_> =
            post_removal_flat.iter().map(hash_lint).sorted().collect();

        assert_eq!(post_removal_flat_hashes, lint_flat_hashes);
    }
}
