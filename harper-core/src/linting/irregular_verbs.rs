use hashbrown::HashSet;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::sync::Arc;

type Verb = (String, String, String);

#[derive(Debug, Deserialize)]
pub struct VerbTable {
    verbs: Vec<Verb>,
}

/// The uncached function that is used to produce the original copy of the
/// irregular verb table.
fn uncached_inner_new() -> Arc<VerbTable> {
    VerbTable::from_json_file(
        include_str!("irregular_verbs.json")
    )
    .map(Arc::new)
    .unwrap_or_else(|e| panic!("Failed to load irregular verb table: {}", e))
}

lazy_static! {
    static ref VERBS: Arc<VerbTable> = uncached_inner_new();
}

impl VerbTable {
    pub fn new() -> Self {
        Self {
            verbs: VerbTable::default()
        }
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
}

impl Default for VerbTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verb_table() {
        let vt = VerbTable::from_json_file(include_str!("irregular_verbs.json"));
        eprintln!("{:#?}", vt);
    }
}
