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
mod indefinite_article;
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
mod regular_nouns;
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

use hashbrown::HashSet;
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
pub use indefinite_article::{InitialSound, starts_with_vowel};
pub use irregular_nouns::IrregularNouns;
pub use irregular_verbs::IrregularVerbs;
use linting::{Lint, Suggestion};
pub use mask::{Mask, Masker, RegexMasker};
pub use number::{Number, OrdinalSuffix};
pub use punctuation::{Punctuation, Quote};
pub use regular_nouns::{get_plurals, get_singulars};
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
/// When two lints cover the same span, their suggestions are merged into the
/// surviving lint so that useful corrections are not lost.
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
    let mut last_kept = 0usize;
    // Collect suggestions to merge: (target_index, extra_suggestions)
    let mut pending_merges: Vec<(usize, Vec<Suggestion>)> = Vec::new();

    for (i, lint) in lints.iter().enumerate() {
        if lint.span.start < cur {
            // This lint overlaps with the previous one.
            // Merge its suggestions into the surviving lint if the spans match exactly.
            if lint.span == lints[last_kept].span {
                pending_merges.push((last_kept, lint.suggestions.clone()));
            }
            remove_indices.push_back(i);
            continue;
        }
        cur = lint.span.end;
        last_kept = i;
    }

    // Apply pending merges: extend surviving lints with suggestions from removed overlapping lints.
    for (target_idx, extra_suggestions) in pending_merges {
        lints[target_idx].suggestions.extend(extra_suggestions);
        // Deduplicate suggestions while preserving order.
        let mut seen = HashSet::new();
        lints[target_idx]
            .suggestions
            .retain(|s| seen.insert(s.clone()));
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

    #[derive(Clone)]
    struct IndexedSpan {
        rule_idx: usize,
        lint_idx: usize,
        priority: u8,
        start: usize,
        end: usize,
        suggestions: Vec<Suggestion>,
    }

    let mut removal_flags: Vec<Vec<bool>> = lint_map
        .values()
        .map(|lints| vec![false; lints.len()])
        .collect();

    let mut spans: Vec<IndexedSpan> = Vec::with_capacity(total);
    for (rule_idx, (_, lints)) in lint_map.iter().enumerate() {
        for (lint_idx, lint) in lints.iter().enumerate() {
            spans.push(IndexedSpan {
                priority: lint.priority,
                rule_idx,
                lint_idx,
                start: lint.span.start,
                end: lint.span.end,
                suggestions: lint.suggestions.clone(),
            });
        }
    }

    spans.sort_by_key(|span| span.priority);
    spans.sort_by_key(|span| (span.start, usize::MAX - span.end));

    // Determine which lints to remove, and collect suggestions to merge into surviving lints.
    let mut cur = 0;
    let mut last_kept: Option<&IndexedSpan> = None;
    // (kept_rule_idx, kept_lint_idx) -> suggestions from removed lints
    let mut pending_merges: BTreeMap<(usize, usize), Vec<Suggestion>> = BTreeMap::new();

    for span in &spans {
        if span.start < cur {
            // This lint overlaps with the previous one.
            // If spans match exactly, queue suggestions for merging.
            if let Some(kept) = last_kept
                && span.start == kept.start
                && span.end == kept.end
            {
                pending_merges
                    .entry((kept.rule_idx, kept.lint_idx))
                    .or_default()
                    .extend(span.suggestions.iter().cloned());
            }
            removal_flags[span.rule_idx][span.lint_idx] = true;
        } else {
            cur = span.end;
            last_kept = Some(span);
        }
    }

    // Apply merges: extend surviving lints with suggestions from removed overlapping lints.
    for ((rule_idx, lint_idx), extra_suggestions) in pending_merges {
        if let Some((_, lints)) = lint_map.iter_mut().nth(rule_idx)
            && let Some(lint) = lints.get_mut(lint_idx)
        {
            lint.suggestions.extend(extra_suggestions);
            let mut seen = HashSet::new();
            lint.suggestions.retain(|s| seen.insert(s.clone()));
        }
    }

    // Remove flagged lints.
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

        let mut linter = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }

    #[quickcheck]
    fn overlap_removals_have_equivalent_behavior(s: String) {
        let doc = Document::new_plain_english_curated(&s);
        let mut linter = LintGroup::new_curated(FstDictionary::curated(), Dialect::American);

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

    /// Regression test for <https://github.com/Automattic/harper/issues/3362>
    /// When two lints cover the same span, their suggestions should be merged
    /// rather than one lint being silently dropped.
    #[test]
    fn overlapping_lints_merge_suggestions() {
        use crate::linting::{LintKind, Suggestion};
        use crate::span::Span;

        let span = Span::new(0, 7);
        let lint_a = Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::ReplaceWith("although".chars().collect())],
            message: "Possibly misspelled word.".to_string(),
            priority: 63,
        };
        let lint_b = Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::ReplaceWith("a though".chars().collect())],
            message: "Possibly missing space.".to_string(),
            priority: 63,
        };

        let mut lints = vec![lint_a, lint_b];
        remove_overlaps(&mut lints);

        // After overlap removal, only one lint should remain,
        // but it should contain BOTH suggestions.
        assert_eq!(lints.len(), 1);
        assert_eq!(lints[0].suggestions.len(), 2);
    }
}
