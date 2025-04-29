#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod char_ext;
mod char_string;
mod currency;
mod document;
mod edit_distance;
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
mod span;
pub mod spell;
mod sync;
mod title_case;
mod token;
mod token_kind;
mod token_string_ext;
mod vec_ext;
mod word_metadata;

use std::collections::VecDeque;

pub use char_string::{CharString, CharStringExt};
pub use currency::Currency;
pub use document::Document;
pub use fat_token::{FatStringToken, FatToken};
pub use ignored_lints::IgnoredLints;
use linting::Lint;
pub use mask::{Mask, Masker};
pub use number::{Number, OrdinalSuffix};
pub use punctuation::{Punctuation, Quote};
pub use span::Span;
pub use spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary, WordId};
pub use sync::{LSend, Lrc};
pub use title_case::{make_title_case, make_title_case_str};
pub use token::Token;
pub use token_kind::TokenKind;
pub use token_string_ext::TokenStringExt;
pub use vec_ext::VecExt;
pub use word_metadata::{
    AdverbData, ConjunctionData, Dialect, NounData, PronounData, Tense, VerbData, WordMetadata,
};

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

#[cfg(test)]
mod tests {
    use crate::{
        Dialect, Document, FstDictionary,
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
