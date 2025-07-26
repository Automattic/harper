mod affix_replacement;
mod attribute_list;
mod error;
mod expansion;
mod matcher;
pub mod word_list;

pub use attribute_list::AttributeList;
pub use error::Error;

pub use self::word_list::parse_word_list;

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;
    use once_cell::sync::Lazy;
    use serde_json::json;

    use super::super::word_map::WordMap;
    use super::word_list::parse_word_list;
    use crate::CharStringExt;
    use crate::spell::rune::AttributeList;

    pub const TEST_WORD_LIST: &str = "4\nhello\ntry/B\nwork/AB\nblank/";

    pub const TEST_WORD_LIST_WITH_BLANK_LINES: &str = "4\n\nhello\n\ntry/B\nwork/AB\n\n\nblank/";

    pub const TEST_WORD_LIST_WITH_FULL_LINE_COMMENTS: &str =
        "4\n#\nhello\n#with\ntry/B\nwork/AB\n# some\n# comments aded\nblank/";

    pub const TEST_WORD_LIST_WITH_COMMENTS: &str = "4\nhello       # a word without attributes\ntry/B   \t  # a word with empty attributes\nwork/AB\t   #a word with one attribute\nblank/      #a word with two attributes";

    pub static TEST_AFFIX_JSON: Lazy<serde_json::Value> = Lazy::new(|| {
        json!({
            "affixes": {
                "A": {
                    "kind": "prefix",
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "re",
                        "condition": "."
                      }
                    ],
                    "target": [],
                    "base_metadata": {}
                },
                "B": {
                    "kind": "suffix",
                    "cross_product": true,
                    "replacements": [
                      {
                        "remove": "",
                        "add": "ed",
                        "condition": "[^y]"
                      },
                      {
                        "remove": "y",
                        "add": "ied",
                        "condition": "y"
                      }
                    ],
                    "target": [
                        {
                            "metadata": {
                                "noun": {}
                            }
                        }
                    ],
                    "base_metadata": {}
                }
            },
            "properties": {}
        })
    });

    fn assert_expansion_results(test_word_list: &str, expected: Vec<&str>) {
        let words = parse_word_list(test_word_list).unwrap();
        let attributes = AttributeList::parse(&TEST_AFFIX_JSON.to_string()).unwrap();

        let mut expanded = WordMap::default();

        attributes.expand_annotated_words(words, &mut expanded);

        let expanded: HashSet<String> = expanded
            .into_iter()
            .map(|v| v.canonical_spelling.into_iter().collect())
            .collect();

        assert_eq!(expanded, expected.into_iter().map(|v| v.into()).collect());
    }

    #[test]
    fn correctly_expands_test_files() {
        assert_expansion_results(
            TEST_WORD_LIST,
            vec![
                "reworked", "rework", "tried", "try", "hello", "worked", "work", "blank",
            ],
        );
    }

    #[test]
    fn correctly_expands_test_files_with_blank_lines() {
        assert_expansion_results(
            TEST_WORD_LIST_WITH_BLANK_LINES,
            vec![
                "reworked", "rework", "tried", "try", "hello", "worked", "work", "blank",
            ],
        );
    }

    fn correctly_expands_test_files_with_full_line_comments() {
        assert_expansion_results(
            TEST_WORD_LIST_WITH_FULL_LINE_COMMENTS,
            vec![
                "reworked", "rework", "tried", "try", "hello", "worked", "work", "blank",
            ],
        );
    }

    #[test]
    fn correctly_expands_test_files_with_comments() {
        let words = parse_word_list(TEST_WORD_LIST_WITH_COMMENTS).unwrap();
        let attributes = AttributeList::parse(&TEST_AFFIX_JSON.to_string()).unwrap();

        let mut expanded = WordMap::default();

        attributes.expand_annotated_words(words, &mut expanded);
        let expanded: HashSet<String> = expanded
            .into_iter()
            .map(|v| v.canonical_spelling.to_string())
            .collect();

        assert_eq!(
            expanded,
            vec![
                "reworked", "rework", "tried", "try", "hello", "worked", "work", "blank"
            ]
            .into_iter()
            .map(|v| v.into())
            .collect()
        )
    }

    #[test]
    fn plural_giants() {
        let words = parse_word_list("1\ngiant/SM").unwrap();

        let attributes = AttributeList::parse(
            &json!({
                "affixes": {
                    "S": {
                        "kind": "suffix",
                        "cross_product": true,
                        "replacements": [
                          {
                            "remove": "y",
                            "add": "ies",
                            "condition": "[^aeiou]"
                          },
                          {
                            "remove": "",
                            "add": "s",
                            "condition": "[aeiou]y"
                          },
                          {
                            "remove": "",
                            "add": "s",
                            "condition": "[^sxzhy]"
                          }
                        ],
                        "target": [
                            {
                                "metadata": {
                                    "noun": {
                                        "is_plural": true
                                    }
                                }
                            }
                        ],
                        "base_metadata": {
                            "noun": {}
                        }
                    },
                    "M": {
                        "kind": "suffix",
                        "cross_product": true,
                        "replacements": [
                          {
                            "remove": "",
                            "add": "'s",
                            "condition": "."
                          }
                        ],
                        "target": [],
                        "base_metadata": {}
                    }
                },
                "properties": {}
            })
            .to_string(),
        )
        .unwrap();

        let mut expanded = WordMap::default();

        attributes.expand_annotated_words(words, &mut expanded);

        let giant_data = expanded.get_with_str("giant").unwrap();
        assert!(giant_data.metadata.is_noun());

        let giants_data = expanded.get_with_str("giants").unwrap();
        assert!(giants_data.metadata.is_plural_noun());
    }
}
