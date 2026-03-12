use serde::Deserialize;
use std::sync::{Arc, LazyLock};

type Verb = (String, String, String);

#[derive(Debug, Deserialize)]
pub struct IrregularVerbs {
    verbs: Vec<Verb>,
}

/// The uncached function that is used to produce the original copy of the
/// irregular verb table.
fn uncached_inner_new() -> Arc<IrregularVerbs> {
    IrregularVerbs::from_json_file(include_str!("../irregular_verbs.json"))
        .map(Arc::new)
        .unwrap_or_else(|e| panic!("Failed to load irregular verb table: {}", e))
}

static VERBS: LazyLock<Arc<IrregularVerbs>> = LazyLock::new(uncached_inner_new);

impl IrregularVerbs {
    pub fn new() -> Self {
        Self { verbs: vec![] }
    }

    pub fn from_json_file(json: &str) -> Result<Self, serde_json::Error> {
        // Deserialize into Vec<serde_json::Value> to handle mixed types
        let values: Vec<serde_json::Value> =
            serde_json::from_str(json).expect("Failed to parse irregular verbs JSON");

        let mut verbs = Vec::new();

        for value in values {
            match value {
                serde_json::Value::Array(arr) if arr.len() == 3 => {
                    // Handle array of 3 strings
                    if let (Some(lemma), Some(preterite), Some(past_participle)) =
                        (arr[0].as_str(), arr[1].as_str(), arr[2].as_str())
                    {
                        verbs.push((
                            lemma.to_string(),
                            preterite.to_string(),
                            past_participle.to_string(),
                        ));
                    }
                }
                // Strings are used for comments to guide contributors editing the file
                serde_json::Value::String(_) => {}
                _ => {}
            }
        }

        Ok(Self { verbs })
    }

    pub fn curated() -> Arc<Self> {
        (*VERBS).clone()
    }

    pub fn get_past_participle_for_preterite(&self, preterite: &str) -> Option<&str> {
        self.verbs
            .iter()
            .find(|(_, pt, _)| pt.eq_ignore_ascii_case(preterite))
            .map(|(_, _, pp)| pp.as_str())
    }

    pub fn get_lemma_for_preterite(&self, preterite: &str) -> Option<&str> {
        self.verbs
            .iter()
            .find(|(_, pt, _)| pt.eq_ignore_ascii_case(preterite))
            .map(|(lemma, _, _)| lemma.as_str())
    }
}

impl Default for IrregularVerbs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_find_irregular_past_participle_for_preterite_lowercase() {
        assert_eq!(
            IrregularVerbs::curated().get_past_participle_for_preterite("arose"),
            Some("arisen")
        );
    }

    #[test]
    fn can_find_irregular_past_participle_for_preterite_uppercase() {
        assert_eq!(
            IrregularVerbs::curated().get_past_participle_for_preterite("WENT"),
            Some("gone")
        );
    }

    #[test]
    fn can_find_irregular_past_participle_same_as_past_tense() {
        assert_eq!(
            IrregularVerbs::curated().get_past_participle_for_preterite("taught"),
            Some("taught")
        );
    }

    #[test]
    fn cant_find_regular_past_participle() {
        assert_eq!(
            IrregularVerbs::curated().get_past_participle_for_preterite("walked"),
            None
        );
    }

    #[test]
    fn cant_find_non_verb() {
        assert_eq!(
            IrregularVerbs::curated().get_past_participle_for_preterite("the"),
            None
        );
    }
}
