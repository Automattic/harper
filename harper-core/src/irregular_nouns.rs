use serde::Deserialize;
use std::sync::{Arc, LazyLock};

type Noun = (String, String);

#[derive(Debug, Deserialize)]
pub struct IrregularNouns {
    nouns: Vec<Noun>,
}

/// The uncached function that is used to produce the original copy of the
/// irregular noun table.
fn uncached_inner_new() -> Arc<IrregularNouns> {
    IrregularNouns::from_json_file(include_str!("../irregular_nouns.json"))
        .map(Arc::new)
        .unwrap_or_else(|e| panic!("Failed to load irregular noun table: {}", e))
}

static NOUNS: LazyLock<Arc<IrregularNouns>> = LazyLock::new(uncached_inner_new);

impl IrregularNouns {
    pub fn new() -> Self {
        Self { nouns: vec![] }
    }

    pub fn from_json_file(json: &str) -> Result<Self, serde_json::Error> {
        // Deserialize into Vec<serde_json::Value> to handle mixed types
        let values: Vec<serde_json::Value> =
            serde_json::from_str(json).expect("Failed to parse irregular nouns JSON");

        let mut nouns = Vec::new();

        for value in values {
            match value {
                serde_json::Value::Array(arr) if arr.len() == 2 => {
                    // Handle array of 2 strings
                    if let (Some(singular), Some(plural)) = (arr[0].as_str(), arr[1].as_str()) {
                        nouns.push((singular.to_string(), plural.to_string()));
                    }
                }
                // Strings are used for comments to guide contributors editing the file
                serde_json::Value::String(_) => {}
                _ => {}
            }
        }

        Ok(Self { nouns })
    }

    pub fn curated() -> Arc<Self> {
        (*NOUNS).clone()
    }

    pub fn get_plural_for_singular(&self, singular: &str) -> Option<&str> {
        self.nouns
            .iter()
            .find(|(sg, _)| sg.eq_ignore_ascii_case(singular))
            .map(|(_, pl)| pl.as_str())
    }

    pub fn get_singular_for_plural(&self, plural: &str) -> Option<&str> {
        self.nouns
            .iter()
            .find(|(_, pl)| pl.eq_ignore_ascii_case(plural))
            .map(|(sg, _)| sg.as_str())
    }
}

impl Default for IrregularNouns {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_find_irregular_plural_for_singular_lowercase() {
        assert_eq!(
            IrregularNouns::curated().get_plural_for_singular("man"),
            Some("men")
        );
    }

    #[test]
    fn can_find_irregular_plural_for_singular_uppercase() {
        assert_eq!(
            IrregularNouns::curated().get_plural_for_singular("WOMAN"),
            Some("women")
        );
    }

    #[test]
    fn can_find_singular_for_irregular_plural() {
        assert_eq!(
            IrregularNouns::curated().get_singular_for_plural("children"),
            Some("child")
        );
    }

    #[test]
    fn cant_find_regular_plural() {
        assert_eq!(
            IrregularNouns::curated().get_plural_for_singular("car"),
            None
        );
    }

    #[test]
    fn cant_find_non_noun() {
        assert_eq!(
            IrregularNouns::curated().get_plural_for_singular("the"),
            None
        );
    }
}
