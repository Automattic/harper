use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::mem;

use cached::proc_macro::cached;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::LintGroup;
use crate::Dialect;
use crate::spell::MutableDictionary;

fn ser_ordered<S>(map: &HashMap<String, Option<bool>>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
    ordered.serialize(ser)
}

fn de_hashbrown<'de, D>(de: D) -> Result<HashMap<String, Option<bool>>, D::Error>
where
    D: Deserializer<'de>,
{
    let ordered: BTreeMap<String, Option<bool>> = BTreeMap::deserialize(de)?;
    Ok(ordered.into_iter().collect())
}

/// The rule-level configuration for a [`LintGroup`].
/// Each child linter can be enabled, disabled, or set to a curated value.
/// So named because it represents the structure of a [`LintGroup`] exactly: it's flat.
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct FlatConfig {
    /// We do this shenanigans with the [`BTreeMap`] to keep the serialized format consistent.
    #[serde(serialize_with = "ser_ordered", deserialize_with = "de_hashbrown")]
    inner: HashMap<String, Option<bool>>,
}

#[cached]
fn curated_config() -> FlatConfig {
    // The Dictionary and Dialect do not matter, we're just after the config.
    let group = LintGroup::new_curated(MutableDictionary::new().into(), Dialect::American);
    group.config
}

impl FlatConfig {
    /// Check if a rule exists in the configuration.
    pub fn has_rule(&self, key: impl AsRef<str>) -> bool {
        self.inner.contains_key(key.as_ref())
    }

    pub fn set_rule_enabled(&mut self, key: impl ToString, val: bool) {
        self.inner.insert(key.to_string(), Some(val));
    }

    /// Remove any configuration attached to a rule.
    /// This allows it to assume its default (curated) state.
    pub fn unset_rule_enabled(&mut self, key: impl AsRef<str>) {
        self.inner.remove(key.as_ref());
    }

    pub fn set_rule_enabled_if_unset(&mut self, key: impl AsRef<str>, val: bool) {
        if !self.inner.contains_key(key.as_ref()) {
            self.set_rule_enabled(key.as_ref().to_string(), val);
        }
    }

    pub fn is_rule_enabled(&self, key: &str) -> bool {
        self.inner.get(key).cloned().flatten().unwrap_or(false)
    }

    /// Clear all config options.
    /// This will reset them all to disable them.
    pub fn clear(&mut self) {
        for val in self.inner.values_mut() {
            *val = None
        }
    }

    /// Merge the contents of another [`FlatConfig`] into this one.
    ///
    /// Conflicting keys will be overridden by the value in the other group.
    pub fn merge_from(&mut self, other: FlatConfig) {
        for (key, val) in other.inner {
            if val.is_none() {
                continue;
            }

            self.inner.insert(key.to_string(), val);
        }
    }

    /// Fill the group with the values for the curated lint group.
    pub fn fill_with_curated(&mut self) {
        let mut temp = Self::new_curated();
        mem::swap(self, &mut temp);
        self.merge_from(temp);
    }

    pub fn new_curated() -> Self {
        curated_config()
    }
}

impl Hash for FlatConfig {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for (key, value) in &self.inner {
            hasher.write(key.as_bytes());
            if let Some(value) = value {
                hasher.write_u8(1);
                hasher.write_u8(*value as u8);
            } else {
                // Do it twice so we fill the same number of bytes as the other branch.
                hasher.write_u8(0);
                hasher.write_u8(0);
            }
        }
    }
}
