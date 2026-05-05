use harper_core::{Dialect, IgnoredLints, linting::FlatConfig, spell::MutableDictionary};
use harper_dictionary_wordlist::{dictionary_to_word_list, mutable_dictionary_from_word_list};
use serde::de::{DeserializeOwned, Error};
use std::{fs, io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("system config directory is unavailable")]
    ConfigDirUnavailable,
    #[error("failed to serialize config")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to access config file")]
    Io(#[from] io::Error),
}

/// User-controlled linting state needed to construct and apply a Harper lint group.
pub struct Config {
    pub mutable_dictionary: MutableDictionary,
    pub dialect: Dialect,
    pub ignored_lints: IgnoredLints,
    pub lint_config: FlatConfig,
}

impl Config {
    pub fn new() -> Self {
        Self {
            mutable_dictionary: MutableDictionary::new(),
            dialect: Dialect::American,
            ignored_lints: IgnoredLints::new(),
            lint_config: FlatConfig::new_curated(),
        }
    }

    pub fn save_to_system(&self) -> Result<(), ConfigError> {
        let path = Self::path().ok_or(ConfigError::ConfigDirUnavailable)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, self.serialize()?)?;

        Ok(())
    }

    #[allow(dead_code)]
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|path| path.join("harper-desktop").join("config.json"))
    }

    #[allow(dead_code)]
    fn serialize(&self) -> serde_json::Result<String> {
        serde_json::to_string(&serde_json::json!({
            "mutable_dictionary": dictionary_to_word_list(&self.mutable_dictionary),
            "dialect": &self.dialect,
            "ignored_lints": &self.ignored_lints,
            "lint_config": &self.lint_config,
        }))
    }

    #[allow(dead_code)]
    fn deserialize(serialized: &str) -> serde_json::Result<Self> {
        let mut value = serde_json::from_str::<serde_json::Value>(serialized)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| serde_json::Error::custom("config must be a JSON object"))?;

        let dialect = deserialize_field(object, "dialect")?;
        let mutable_dictionary = mutable_dictionary_from_word_list(
            &deserialize_field::<String>(object, "mutable_dictionary")?,
            dialect,
        );

        Ok(Self {
            mutable_dictionary,
            dialect,
            ignored_lints: deserialize_field(object, "ignored_lints")?,
            lint_config: deserialize_field(object, "lint_config")?,
        })
    }
}

#[allow(dead_code)]
fn deserialize_field<T>(
    object: &mut serde_json::Map<String, serde_json::Value>,
    field: &'static str,
) -> serde_json::Result<T>
where
    T: DeserializeOwned,
{
    let value = object
        .remove(field)
        .ok_or_else(|| serde_json::Error::custom(format!("missing config field `{field}`")))?;

    serde_json::from_value(value)
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use harper_core::{DictWordMetadata, spell::Dictionary};

    #[test]
    fn serialize_includes_dictionary_word_list() {
        let mut config = Config::new();
        config
            .mutable_dictionary
            .append_word_str("blorple", DictWordMetadata::default());

        let serialized = config.serialize().unwrap();

        assert!(serialized.contains("mutable_dictionary"));
        assert!(serialized.contains("blorple"));
        assert!(serialized.contains("dialect"));
        assert!(serialized.contains("ignored_lints"));
        assert!(serialized.contains("lint_config"));
    }

    #[test]
    fn deserialize_restores_serialized_fields() {
        let mut config = Config::new();
        config
            .mutable_dictionary
            .append_word_str("blorple", DictWordMetadata::default());
        let serialized = config.serialize().unwrap();

        let deserialized = Config::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.dialect, config.dialect);
        assert_eq!(deserialized.lint_config, config.lint_config);
        assert!(deserialized.mutable_dictionary.contains_word_str("blorple"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&deserialized.serialize().unwrap()).unwrap(),
            serde_json::from_str::<serde_json::Value>(&serialized).unwrap()
        );
    }

    #[test]
    fn path_points_to_harper_desktop_config_file() {
        let path = Config::path().unwrap();

        assert_eq!(path.file_name().unwrap(), "config.json");
        assert_eq!(
            path.parent().unwrap().file_name().unwrap(),
            "harper-desktop"
        );
    }
}
