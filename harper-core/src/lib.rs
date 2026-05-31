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
    // Suggestions suggested by multiple overlapping lints are ranked first (by count, descending),
    // then remaining suggestions preserve their original order.
    for (target_idx, extra_suggestions) in pending_merges {
        let original = lints[target_idx].suggestions.clone();
        let merged: Vec<Suggestion> = original
            .iter()
            .chain(extra_suggestions.iter())
            .cloned()
            .collect();

        // Count how many times each suggestion appears across the merged set.
        // Use a Vec of (Suggestion, count) pairs since Suggestion doesn't impl Ord.
        let mut count_pairs: Vec<(Suggestion, usize)> = Vec::new();
        for s in &merged {
            if let Some(entry) = count_pairs.iter_mut().find(|(k, _)| k == s) {
                entry.1 += 1;
            } else {
                count_pairs.push((s.clone(), 1));
            }
        }

        // Stable-sort by count descending so that suggestions from multiple lints
        // come first, while preserving relative order within the same count.
        lints[target_idx].suggestions = merged;
        lints[target_idx].suggestions.sort_by(|a, b| {
            let ca = count_pairs
                .iter()
                .find(|(k, _)| k == a)
                .map(|p| p.1)
                .unwrap_or(1);
            let cb = count_pairs
                .iter()
                .find(|(k, _)| k == b)
                .map(|p| p.1)
                .unwrap_or(1);
            cb.cmp(&ca)
        });
        lints[target_idx].suggestions.dedup();
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
    // Suggestions from multiple overlapping lints are ranked first (by count, descending).
    for ((rule_idx, lint_idx), extra_suggestions) in pending_merges {
        if let Some((_, lints)) = lint_map.iter_mut().nth(rule_idx)
            && let Some(lint) = lints.get_mut(lint_idx)
        {
            let original = lint.suggestions.clone();
            let merged: Vec<Suggestion> = original
                .iter()
                .chain(extra_suggestions.iter())
                .cloned()
                .collect();

            let mut count_pairs: Vec<(Suggestion, usize)> = Vec::new();
            for s in &merged {
                if let Some(entry) = count_pairs.iter_mut().find(|(k, _)| k == s) {
                    entry.1 += 1;
                } else {
                    count_pairs.push((s.clone(), 1));
                }
            }

            lint.suggestions = merged;
            lint.suggestions.sort_by(|a, b| {
                let ca = count_pairs
                    .iter()
                    .find(|(k, _)| k == a)
                    .map(|p| p.1)
                    .unwrap_or(1);
                let cb = count_pairs
                    .iter()
                    .find(|(k, _)| k == b)
                    .map(|p| p.1)
                    .unwrap_or(1);
                cb.cmp(&ca)
            });
            lint.suggestions.dedup();
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

    /// Regression test for <https://github.com/Automattic/harper/issues/2460>
    /// When SpellCheck and SplitWords both flag a compound like "titlecase",
    /// suggestions from both linters should be preserved.
    #[test]
    fn overlapping_spellcheck_splitwords_merge_suggestions() {
        use crate::linting::{LintKind, Suggestion};
        use crate::span::Span;

        let span = Span::new(0, 9);
        let spell_lint = Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::ReplaceWith("title's".chars().collect())],
            message: "Possibly misspelled word.".to_string(),
            priority: 63,
        };
        let split_lint = Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::ReplaceWith("title case".chars().collect())],
            message: "Possibly missing space.".to_string(),
            priority: 63,
        };

        let mut lints = vec![spell_lint, split_lint];
        remove_overlaps(&mut lints);

        // After overlap removal, one lint should remain with both suggestions.
        assert_eq!(lints.len(), 1);
        assert!(lints[0].suggestions.len() >= 2);
    }

    /// When overlapping lints share identical suggestions, deduplication
    /// should ensure no duplicates remain.
    #[test]
    fn overlapping_lints_deduplicate_suggestions() {
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
            suggestions: vec![Suggestion::ReplaceWith("although".chars().collect())],
            message: "Another checker.".to_string(),
            priority: 63,
        };

        let mut lints = vec![lint_a, lint_b];
        remove_overlaps(&mut lints);

        assert_eq!(lints.len(), 1);
        // Same suggestion from both lints should be deduplicated.
        assert_eq!(lints[0].suggestions.len(), 1);
    }

    /// When multiple overlapping lints share a suggestion, that suggestion should
    /// appear first after frequency-based sorting (ranked by how many lints
    /// proposed it, descending).
    #[test]
    fn overlapping_lints_rank_shared_suggestions_first() {
        use crate::linting::{LintKind, Suggestion};
        use crate::span::Span;

        let span = Span::new(0, 5);
        let shared = Suggestion::ReplaceWith("their".chars().collect());
        let unique_a = Suggestion::ReplaceWith("there".chars().collect());
        let unique_b = Suggestion::ReplaceWith("they're".chars().collect());

        let lint_a = Lint {
            span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![shared.clone(), unique_a],
            message: "Possibly misspelled word.".to_string(),
            priority: 63,
        };
        let lint_b = Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![shared.clone(), unique_b],
            message: "Consider alternative.".to_string(),
            priority: 63,
        };

        let mut lints = vec![lint_a, lint_b];
        remove_overlaps(&mut lints);

        assert_eq!(lints.len(), 1);
        let suggestions = &lints[0].suggestions;
        // All three suggestions should be present (shared + unique_a + unique_b)
        assert_eq!(suggestions.len(), 3);
        // The shared suggestion ("their") appears in both original lints,
        // so it should be ranked first after frequency-based sorting.
        assert_eq!(suggestions[0], shared);
    }
}
