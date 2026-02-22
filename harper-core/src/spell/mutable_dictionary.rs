use super::word_map::WordMap;

/// A basic dictionary that allows words to be added after instantiating.
/// This is useful for user and file dictionaries that may change at runtime.
///
/// For immutable use-cases that require fuzzy lookups, such as the curated dictionary, prefer [`super::FstDictionary`],
/// as it is much faster.
///
/// To combine the contents of multiple dictionaries, regardless of type, use
/// [`super::MergedDictionary`].
pub type MutableDictionary = WordMap;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use hashbrown::HashSet;
    use itertools::Itertools;

    use crate::char_string::char_string;
    use crate::spell::{CommonDictFuncs, Dictionary, MutableDictionary, WordMapEntry};

    #[test]
    fn curated_contains_no_duplicates() {
        let dict = MutableDictionary::curated();
        assert!(dict.words_iter().all_unique());
    }

    #[test]
    fn curated_matches_capitalized() {
        let dict = MutableDictionary::curated();
        assert!(dict.contains_word_str("this"));
        assert!(dict.contains_word_str("This"));
    }

    // "This" is a determiner when used similarly to "the"
    // but when used alone it's a "demonstrative pronoun".
    // Harper previously wrongly classified it as a noun.
    #[test]
    fn this_is_determiner() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("this")
                .unwrap()
                .metadata
                .is_determiner()
        );
    }

    #[test]
    fn several_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("several")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn few_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("few")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn fewer_is_quantifier() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("fewer")
                .unwrap()
                .metadata
                .is_quantifier()
        );
    }

    #[test]
    fn than_is_conjunction() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("than")
                .unwrap()
                .metadata
                .is_conjunction()
        );
    }

    #[test]
    fn herself_is_pronoun() {
        let dict = MutableDictionary::curated();
        assert!(
            dict.get_word_exact_str("herself")
                .unwrap()
                .metadata
                .is_pronoun()
        );
    }

    #[test]
    fn discussion_171() {
        let dict = MutableDictionary::curated();
        assert!(dict.contains_word_str("natively"));
    }

    #[test]
    fn im_is_common() {
        let dict = MutableDictionary::curated();
        assert!(dict.get_word_exact_str("I'm").unwrap().metadata.common);
    }

    #[test]
    fn fuzzy_result_sorted_by_edit_distance() {
        let dict = MutableDictionary::curated();

        let results = dict.fuzzy_match_str("hello", 3, 100);
        let is_sorted_by_dist = results
            .iter()
            .map(|fm| fm.edit_distance)
            .tuple_windows()
            .all(|(a, b)| a <= b);

        assert!(is_sorted_by_dist)
    }

    #[test]
    fn there_is_not_a_pronoun() {
        let dict = MutableDictionary::curated();
        let there_meta = dict.get_word_exact_str("there").unwrap();

        assert!(!there_meta.metadata.is_nominal());
        assert!(!there_meta.metadata.is_pronoun());
    }

    #[test]
    fn expanded_contains_giants() {
        assert!(MutableDictionary::curated().contains_word_str("giants"));
    }

    #[test]
    fn expanded_contains_deallocate() {
        assert!(MutableDictionary::curated().contains_word_str("deallocate"));
    }

    #[test]
    fn curated_contains_repo() {
        let dict = MutableDictionary::curated();

        assert!(dict.contains_word_str("repo"));
        assert!(dict.contains_word_str("repos"));
        assert!(dict.contains_word_str("repo's"));
    }

    #[test]
    fn curated_contains_possessive_abandonment() {
        assert!(
            MutableDictionary::curated()
                .get_word_exact_str("abandonment's")
                .unwrap()
                .metadata
                .is_possessive_noun()
        )
    }

    #[test]
    fn has_is_not_a_nominal() {
        let dict = MutableDictionary::curated();

        let has = dict.get_word_exact_str("has");
        assert!(has.is_some());

        assert!(!has.unwrap().metadata.is_nominal())
    }

    #[test]
    fn is_is_linking_verb() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_exact_str("is");

        assert!(is.is_some());
        assert!(is.unwrap().metadata.is_linking_verb());
    }

    #[test]
    fn are_merged_attrs_same_as_spread_attrs() {
        let curated_attr_list = include_str!("../../annotations.json");

        let merged = MutableDictionary::from_rune_files("1\nblork/DGS", curated_attr_list).unwrap();
        let spread =
            MutableDictionary::from_rune_files("2\nblork/DG\nblork/S", curated_attr_list).unwrap();

        assert_eq!(
            merged.into_iter().collect::<HashSet<_>>(),
            spread.into_iter().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn apart_is_not_noun() {
        let dict = MutableDictionary::curated();

        assert!(!dict.get_word_exact_str("apart").unwrap().metadata.is_noun());
    }

    #[test]
    fn be_is_verb_lemma() {
        let dict = MutableDictionary::curated();

        let is = dict.get_word_exact_str("be");

        assert!(is.is_some());
        assert!(is.unwrap().metadata.is_verb_lemma());
    }

    #[test]
    fn gets_prefixes_as_expected() {
        let mut dict = MutableDictionary::new();
        dict.insert(WordMapEntry::new_str("predict"));
        dict.insert(WordMapEntry::new_str("prelude"));
        dict.insert(WordMapEntry::new_str("preview"));
        dict.insert(WordMapEntry::new_str("dwight"));

        let with_prefix = dict.find_words_with_prefix(char_string!("pre").as_slice());

        assert_eq!(with_prefix.len(), 3);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("predict").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prelude").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("preview").into_vec())));
    }

    #[test]
    fn gets_common_prefixes_as_expected() {
        let mut dict = MutableDictionary::new();
        dict.insert(WordMapEntry::new_str("pre"));
        dict.insert(WordMapEntry::new_str("prep"));
        dict.insert(WordMapEntry::new_str("dwight"));

        let with_prefix =
            dict.find_words_with_common_prefix(char_string!("preposition").as_slice());

        assert_eq!(with_prefix.len(), 2);
        assert!(with_prefix.contains(&Cow::Owned(char_string!("pre").into_vec())));
        assert!(with_prefix.contains(&Cow::Owned(char_string!("prep").into_vec())));
    }
}
