//! Orders Harper's spelling suggestions by how well they fit the words around a misspelled word.

#[cfg(feature = "training")]
mod corpus;
#[cfg(feature = "training")]
mod prepare;
mod ranker;
#[cfg(feature = "training")]
mod training;

use std::sync::LazyLock;

#[cfg(feature = "training")]
pub use prepare::{SentenceWord, Speller, prepare};
pub use ranker::CONTEXT_RADIUS;
#[cfg(feature = "training")]
pub use training::{TrainingOptions, train};

static RANKER: LazyLock<ranker::ContextRanker> = LazyLock::new(|| {
    ranker::ContextRanker::load_from_bytes(
        include_bytes!("../data/table.bin").as_slice(),
        include_bytes!("../data/vocab.json"),
        include_bytes!("../data/config.json"),
    )
});

/// Candidate positions, best first. Words are lowercase, and lower spelling scores are better.
pub fn rank(left: &[String], right: &[String], candidates: &[(String, i32)]) -> Vec<usize> {
    RANKER.rank(left, right, candidates)
}

#[cfg(test)]
mod tests {
    use super::{CONTEXT_RADIUS, rank};

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    fn candidates(list: &str) -> Vec<(String, i32)> {
        list.split_whitespace()
            .map(|word| (word.to_string(), 0))
            .collect()
    }

    fn first(left: &str, right: &str, list: &str) -> String {
        let candidates = candidates(list);
        let order = rank(&words(left), &words(right), &candidates);
        candidates[order[0]].0.clone()
    }

    fn is_permutation(order: &[usize], len: usize) -> bool {
        let mut sorted = order.to_vec();
        sorted.sort_unstable();
        sorted == (0..len).collect::<Vec<_>>()
    }

    #[test]
    fn the_embedded_table_picks_the_word_that_fits() {
        assert_eq!(first("one two three four", "", "fife file five"), "five");
        assert_eq!(first("the house caught", "", "file fire five"), "fire");
        assert_eq!(first("save the", "", "fife fire five file"), "file");
        assert_eq!(first("we ate", "at noon", "lush luck lunch"), "lunch");
        assert_eq!(
            first("people all over the", "", "words works world"),
            "world"
        );
    }

    #[test]
    fn the_right_side_alone_is_enough() {
        assert_eq!(
            first("", "alarm went off in the building", "fife file five fire"),
            "fire"
        );
        assert_eq!(
            first("", "six seven eight nine ten", "fife file fire five"),
            "five"
        );
    }

    #[test]
    fn unknown_context_keeps_the_order() {
        let candidates = candidates("fife file five");

        assert_eq!(rank(&[], &[], &candidates), [0, 1, 2]);
        assert_eq!(
            rank(&words("qzxv wkpj"), &words("xxqq"), &candidates),
            [0, 1, 2]
        );
    }

    #[test]
    fn fewer_than_two_candidates_keep_the_order() {
        let context = words("one two three four");

        assert!(rank(&context, &[], &[]).is_empty());
        assert_eq!(rank(&context, &[], &candidates("five")), [0]);
    }

    #[test]
    fn unknown_candidates_still_get_a_place() {
        let candidates = candidates("five qzxvw file");
        let order = rank(&words("one two three four"), &[], &candidates);

        assert!(is_permutation(&order, candidates.len()));
        assert_eq!(order[0], 0);
    }

    #[test]
    fn equal_candidates_keep_their_order() {
        let candidates = candidates("five five five");

        assert_eq!(
            rank(&words("one two three four"), &[], &candidates),
            [0, 1, 2]
        );
    }

    #[test]
    fn a_much_better_spelling_score_beats_the_context() {
        let candidates = vec![("file".to_string(), 0), ("five".to_string(), 500)];

        assert_eq!(rank(&words("one two three four"), &[], &candidates), [0, 1]);
    }

    #[test]
    fn long_contexts_use_only_the_nearest_words() {
        let unknown = words(&"qzxv ".repeat(CONTEXT_RADIUS));
        let numbers = words("one two three four");
        let left: Vec<String> = numbers.iter().chain(&unknown).cloned().collect();
        let right: Vec<String> = unknown.iter().chain(&numbers).cloned().collect();
        let candidates = candidates("fife file five");

        assert_eq!(rank(&left, &[], &candidates), [0, 1, 2]);
        assert_eq!(rank(&[], &right, &candidates), [0, 1, 2]);
        assert_eq!(rank(&numbers, &unknown, &candidates)[0], 2);
    }

    #[test]
    fn the_embedded_vocabulary_is_well_formed() {
        let words: Vec<String> =
            serde_json::from_slice(include_bytes!("../data/vocab.json")).unwrap();
        let unique: hashbrown::HashSet<&String> = words.iter().collect();

        assert_eq!(&words[..4], ["<PAD>", "<UNK>", "<TARGET>", "<NUM>"]);
        assert_eq!(unique.len(), words.len());
        assert!(words.len() <= u16::MAX as usize);
        assert!(
            words[4..]
                .iter()
                .all(|word| !word.is_empty() && *word == word.to_lowercase())
        );
    }

    #[test]
    fn every_result_is_a_permutation() {
        let cases = [
            ("the cat sat on the", "fife file five fire fine"),
            ("", "a b c"),
            ("<NUM> <NUM>", "fife five"),
        ];

        for (context, list) in cases {
            let candidates = candidates(list);
            let order = rank(&words(context), &words(context), &candidates);
            assert!(is_permutation(&order, candidates.len()));
        }
    }
}
