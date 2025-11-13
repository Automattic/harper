use serde::{self, Deserialize, Serialize};

use super::Error;
use super::matcher::Matcher;

#[derive(Debug, Clone)]
pub struct AffixReplacement {
    pub metadata_condition: Option<serde_json::Value>,
    pub remove: Vec<char>,
    pub add: Vec<char>,
    pub condition: Matcher,
}

impl AffixReplacement {
    pub fn to_human_readable(&self) -> HumanReadableAffixReplacement {
        // let metadata_condition = self
        //     .metadata_condition
        //     .as_ref()
        //     .map(|x| serde_json::to_string(&x).unwrap());
        HumanReadableAffixReplacement {
            metadata_condition: self.metadata_condition.clone(),
            remove: self.remove.iter().collect(),
            add: self.add.iter().collect(),
            condition: self.condition.to_string(),
        }
    }
}

/// A version of [`AffixReplacement`] that can be serialized to JSON (or
/// whatever) and maintain the nice Regex syntax of the inner [`Matcher`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReadableAffixReplacement {
    pub metadata_condition: Option<serde_json::Value>,
    pub remove: String,
    pub add: String,
    pub condition: String,
}

impl HumanReadableAffixReplacement {
    pub fn to_normal(&self) -> Result<AffixReplacement, Error> {
        // let metadata_condition: Option<serde_json::Value> = self
        //     .metadata_condition
        //     .as_ref()
        //     .map(|data| serde_json::from_str(data).unwrap());
        Ok(AffixReplacement {
            metadata_condition: self.metadata_condition.clone(),
            remove: self.remove.chars().collect(),
            add: self.add.chars().collect(),
            condition: Matcher::parse(&self.condition)?,
        })
    }
}
