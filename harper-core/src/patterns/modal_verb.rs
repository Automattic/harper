use std::sync::LazyLock;

use super::{Pattern, WordSet};
use crate::Token;

enum Kind {
    True,   // can could may might must shall should will would
    Pseudo, // need ought dare
}

enum NegativeContraction {
    Irregular(&'static str), // can't shan't won't
    Regular,                 // just append -n't
    Nonstandard,             // mayn't
}

struct Entry {
    kind: Kind,
    positive: &'static str,
    special_negative: Option<&'static str>, // cannot
    negative_contraction: NegativeContraction,
}

const MODAL_VERB_TABLE: [Entry; 12] = [
    Entry {
        kind: Kind::True,
        positive: "can",
        special_negative: Some("cannot"),
        negative_contraction: NegativeContraction::Irregular("can't"),
    },
    Entry {
        kind: Kind::True,
        positive: "could",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::True,
        positive: "may",
        special_negative: None,
        negative_contraction: NegativeContraction::Nonstandard,
    },
    Entry {
        kind: Kind::True,
        positive: "might",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::True,
        positive: "must",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::True,
        positive: "shall",
        special_negative: None,
        negative_contraction: NegativeContraction::Irregular("shan't"),
    },
    Entry {
        kind: Kind::True,
        positive: "should",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::True,
        positive: "will",
        special_negative: None,
        negative_contraction: NegativeContraction::Irregular("won't"),
    },
    Entry {
        kind: Kind::True,
        positive: "would",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::Pseudo,
        positive: "dare",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::Pseudo,
        positive: "need",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
    Entry {
        kind: Kind::Pseudo,
        positive: "ought",
        special_negative: None,
        negative_contraction: NegativeContraction::Regular,
    },
];

pub struct ModalVerb {
    inner: &'static WordSet,
}

impl Default for ModalVerb {
    fn default() -> Self {
        Self::without_common_errors()
    }
}

impl ModalVerb {
    pub fn positive_only() -> Self {
        static CACHED_POSITIVE_ONLY: LazyLock<WordSet> =
            LazyLock::new(|| ModalVerb::build_word_set(false, true));
        Self {
            inner: &CACHED_POSITIVE_ONLY,
        }
    }

    pub fn without_common_errors() -> Self {
        static CACHED_WITHOUT_COMMON_ERRORS: LazyLock<WordSet> =
            LazyLock::new(|| ModalVerb::build_word_set(false, false));
        Self {
            inner: &CACHED_WITHOUT_COMMON_ERRORS,
        }
    }

    pub fn with_common_errors() -> Self {
        static CACHED_WITH_COMMON_ERRORS: LazyLock<WordSet> =
            LazyLock::new(|| ModalVerb::build_word_set(true, false));
        Self {
            inner: &CACHED_WITH_COMMON_ERRORS,
        }
    }

    /// Construct the word set exactly once per LazyLock initialization.
    fn build_word_set(include_lazy: bool, positive_only: bool) -> WordSet {
        let mut words = WordSet::new(&[]);

        for mv in &MODAL_VERB_TABLE {
            words.add(mv.positive);

            if positive_only {
                continue;
            }

            if let Some(special_negative) = mv.special_negative {
                words.add(special_negative);
            }

            let contraction = match mv.negative_contraction {
                NegativeContraction::Irregular(irregular) => irregular.to_string(),
                _ => format!("{}n't", mv.positive),
            };

            words.add(&contraction);

            if include_lazy {
                let lazy_spelling = contraction.replace('\'', "");
                words.add(&lazy_spelling);
            }
        }

        words
    }
}

impl Pattern for ModalVerb {
    fn matches(&self, tokens: &[Token], source: &[char]) -> Option<usize> {
        self.inner.matches(tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, Span, Token, patterns::DocPattern};

    trait SpanVecExt {
        fn to_strings(&self, doc: &Document) -> Vec<String>;
    }

    impl SpanVecExt for Vec<Span<Token>> {
        fn to_strings(&self, doc: &Document) -> Vec<String> {
            self.iter()
                .map(|sp| {
                    doc.get_tokens()[sp.start..sp.end]
                        .iter()
                        .map(|tok| doc.get_span_content_str(&tok.span))
                        .collect::<String>()
                })
                .collect()
        }
    }

    const ALL_POSITIVE_TRUE_MODALS: &[&str] = &[
        "can", "could", "may", "might", "must", "shall", "should", "will", "would",
    ];
    const ALL_POSITIVE_PSEUDO_MODALS: &[&str] = &["dare", "need", "ought"];
    const ALL_REGULAR_NEGATIVES: &[&str] =
        &["couldn't", "mightn't", "mustn't", "shouldn't", "wouldn't"];
    const ALL_SPECIAL_NEGATIVES: &[&str] = &["cannot"];
    const ALL_IRREGULAR_NEGATIVES: &[&str] = &["can't", "shan't", "won't"];
    const ALL_NONSTANDARD_NEGATIVES: &[&str] = &["mayn't"];
    const ALL_LAZY_NEGATIVES: &[&str] = &[
        "cant", "couldnt", "maynt", "mightnt", "mustnt", "shant", "shouldnt", "wont", "wouldnt",
        "darent", "neednt", "oughtnt",
    ];

    #[test]
    fn without_common_errors_includes_all_positive_true_modals() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated(
            "can could may might must shall should will would",
        );

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_POSITIVE_TRUE_MODALS
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_all_positive_pseudo_modals() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated("dare need ought");

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_POSITIVE_PSEUDO_MODALS
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_all_regular_negatives() {
        let mv = ModalVerb::without_common_errors();
        let doc =
            Document::new_markdown_default_curated("couldn't mightn't mustn't shouldn't wouldn't");

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_REGULAR_NEGATIVES
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_all_special_negatives() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated("cannot");

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_SPECIAL_NEGATIVES
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_all_irregular_negatives() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated("can't shan't won't");

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_IRREGULAR_NEGATIVES
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_all_nonstandard_negatives() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated("mayn't");

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert!(
            ALL_NONSTANDARD_NEGATIVES
                .iter()
                .all(|m| matches.iter().any(|s| s == m))
        );
    }

    #[test]
    fn without_common_errors_includes_no_lazy_negatives() {
        let mv = ModalVerb::without_common_errors();
        let doc = Document::new_markdown_default_curated(
            "cant couldnt maynt mightnt mustnt shant shouldnt wont wouldnt darent neednt oughtnt",
        );

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn with_common_errors_includes_all_lazy_negatives() {
        let mv = ModalVerb::with_common_errors();
        let doc = Document::new_markdown_default_curated(
            "cant couldnt maynt mightnt mustnt shant shouldnt wont wouldnt darent neednt oughtnt",
        );

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert_eq!(matches.len(), ALL_LAZY_NEGATIVES.len());
    }

    #[test]
    fn positives_doesnt_include_various_kinds_of_negatives() {
        let mv = ModalVerb::positive_only();
        let doc = Document::new_markdown_default_curated(
            "REGULAR couldn't SPECIAL cannot IRREGULAR can't NONSTANDARD mayn't LAZY couldnt cant maynt PSEUDO darent neednt oughtnt",
        );

        let matches = mv.find_all_matches_in_doc(&doc).to_strings(&doc);

        assert_eq!(matches.len(), 0);
    }
}
