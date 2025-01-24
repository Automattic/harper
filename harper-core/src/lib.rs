#![doc = include_str!("../README.md")]
#![allow(dead_code)]

mod char_ext;
mod char_string;
mod currency;
mod document;
mod fat_token;
pub mod language_detection;
mod lexing;
pub mod linting;
mod mask;
pub mod parsers;
pub mod patterns;
mod punctuation;
mod span;
mod spell;
mod sync;
mod title_case;
mod token;
mod token_kind;
mod vec_ext;
mod word_metadata;

use std::collections::VecDeque;

pub use char_string::{CharString, CharStringExt};
pub use currency::Currency;
pub use document::Document;
pub use fat_token::FatToken;
use linting::Lint;
pub use mask::{Mask, Masker};
pub use punctuation::{Punctuation, Quote};
pub use span::Span;
pub use spell::{Dictionary, FstDictionary, FullDictionary, MergedDictionary};
pub use sync::Lrc;
pub use title_case::{make_title_case, make_title_case_str};
pub use token::{Token, TokenStringExt};
pub use token_kind::{NumberSuffix, TokenKind};
pub use vec_ext::VecExt;
pub use word_metadata::{AdverbData, ConjunctionData, NounData, Tense, VerbData, WordMetadata};

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
            let Some(prev) = lints.get(i - 1) else {
                remove_indices.push_back(i);
                continue;
            };

            if lint.priority < prev.priority {
                remove_indices.push_back(i - 1);
            } else {
                remove_indices.push_back(i);
            }
            continue;
        }
        cur = lint.span.end;
    }

    lints.remove_indices(remove_indices);
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::{LintGroup, LintGroupConfig, Linter},
        remove_overlaps, Document, FstDictionary,
    };

    #[test]
    fn keeps_space_lint() {
        let doc = Document::new_plain_english_curated("Ths  tet");

        let lint_config = LintGroupConfig {
            spell_check: Some(true),
            spaces: Some(true),
            ..LintGroupConfig::none()
        };
        let mut linter = LintGroup::new(lint_config, FstDictionary::curated());

        let mut lints = linter.lint(&doc);

        dbg!(&lints);
        remove_overlaps(&mut lints);
        dbg!(&lints);

        assert_eq!(lints.len(), 3);
    }
}
