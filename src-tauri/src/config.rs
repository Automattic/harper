use harper_core::{
    Dialect, IgnoredLints,
    linting::{FlatConfig, LintGroup},
    spell::{FstDictionary, MergedDictionary, MutableDictionary},
};
use harper_dictionary_wordlist::{load_dict, save_dict};
use serde::de::{DeserializeOwned, Error};
use std::{fs, io, path::PathBuf, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("system config directory is unavailable")]
    ConfigDirUnavailable,
    #[error("failed to serialize or deserialize config")]
    Serde(#[from] serde_json::Error),
    #[error("failed to access config file")]
    Io(#[from] io::Error),
}

/// User-controlled app state needed by Tauri commands and the highlighter process.
pub struct Config {
    pub mutable_dictionary: MutableDictionary,
    pub dialect: Dialect,
    pub ignored_lints: IgnoredLints,
    pub lint_config: FlatConfig,
    pub allowed_bundle_identifiers: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            mutable_dictionary: MutableDictionary::new(),
            dialect: Dialect::American,
            ignored_lints: IgnoredLints::new(),
            lint_config: FlatConfig::new_curated(),
            allowed_bundle_identifiers: Self::curated_allowed_bundle_identifiers(),
        }
    }

    pub fn curated_allowed_bundle_identifiers() -> Vec<String> {
        vec!["com.apple.TextEdit".to_string()]
    }

    pub fn is_approved_bundle_identifier(&self, bundle_identifier: &str) -> bool {
        Self::is_bundle_identifier_approved_by(&self.allowed_bundle_identifiers, bundle_identifier)
    }

    pub fn is_bundle_identifier_approved_by(
        allowed_bundle_identifiers: &[String],
        bundle_identifier: &str,
    ) -> bool {
        allowed_bundle_identifiers
            .iter()
            .any(|allowed| allowed == bundle_identifier)
    }

    pub fn add_allowed_bundle_identifier(&mut self, bundle_identifier: String) {
        let bundle_identifier = bundle_identifier.trim();
        if bundle_identifier.is_empty() || self.is_approved_bundle_identifier(bundle_identifier) {
            return;
        }

        self.allowed_bundle_identifiers
            .push(bundle_identifier.to_string());
        self.allowed_bundle_identifiers.sort();
    }

    pub fn remove_allowed_bundle_identifier(&mut self, bundle_identifier: &str) {
        self.allowed_bundle_identifiers
            .retain(|allowed| allowed != bundle_identifier);
    }

    pub async fn save_to_system(&self) -> Result<(), ConfigError> {
        let folder_path = Self::folder_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let main_path = Self::main_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let dictionary_path = Self::dictionary_path().ok_or(ConfigError::ConfigDirUnavailable)?;

        fs::create_dir_all(folder_path)?;
        fs::write(main_path, self.serialize_main()?)?;
        save_dict(dictionary_path, &self.mutable_dictionary).await?;

        Ok(())
    }

    pub async fn load_from_system() -> Result<Self, ConfigError> {
        let main_path = Self::main_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let dictionary_path = Self::dictionary_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let serialized = fs::read_to_string(main_path)?;
        let mut config = Self::deserialize_main(&serialized)?;
        config.mutable_dictionary = load_dict(dictionary_path, config.dialect).await?;

        Ok(config)
    }

    pub fn dictionary_from_user_dictionary(
        user_dictionary: MutableDictionary,
    ) -> Arc<MergedDictionary> {
        let mut dictionary = MergedDictionary::new();
        dictionary.add_dictionary(FstDictionary::curated());
        dictionary.add_dictionary(Arc::new(user_dictionary));

        Arc::new(dictionary)
    }

    pub fn create_dictionary(&self) -> Arc<MergedDictionary> {
        Self::dictionary_from_user_dictionary(self.mutable_dictionary.clone())
    }

    pub fn create_linter(&self) -> LintGroup {
        LintGroup::new_curated(self.create_dictionary(), self.dialect)
            .with_lint_config(self.lint_config.clone())
    }

    #[allow(dead_code)]
    fn folder_path() -> Option<PathBuf> {
        dirs::config_dir().map(|path| path.join("harper-desktop"))
    }

    #[allow(dead_code)]
    fn main_path() -> Option<PathBuf> {
        Self::folder_path().map(|path| path.join("config.json"))
    }

    #[allow(dead_code)]
    fn dictionary_path() -> Option<PathBuf> {
        Self::folder_path().map(|path| path.join("dictionary.txt"))
    }

    #[allow(dead_code)]
    fn serialize_main(&self) -> serde_json::Result<String> {
        serde_json::to_string(&serde_json::json!({
            "dialect": &self.dialect,
            "ignored_lints": &self.ignored_lints,
            "lint_config": &self.lint_config,
            "allowed_bundle_identifiers": &self.allowed_bundle_identifiers,
        }))
    }

    #[allow(dead_code)]
    fn deserialize_main(serialized: &str) -> serde_json::Result<Self> {
        let mut value = serde_json::from_str::<serde_json::Value>(serialized)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| serde_json::Error::custom("config must be a JSON object"))?;

        Ok(Self {
            mutable_dictionary: MutableDictionary::new(),
            dialect: deserialize_field(object, "dialect")?,
            ignored_lints: deserialize_field(object, "ignored_lints")?,
            lint_config: deserialize_field(object, "lint_config")?,
            allowed_bundle_identifiers: deserialize_optional_field(
                object,
                "allowed_bundle_identifiers",
            )?
            .unwrap_or_else(Self::curated_allowed_bundle_identifiers),
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

#[allow(dead_code)]
fn deserialize_optional_field<T>(
    object: &mut serde_json::Map<String, serde_json::Value>,
    field: &'static str,
) -> serde_json::Result<Option<T>>
where
    T: DeserializeOwned,
{
    let Some(value) = object.remove(field) else {
        return Ok(None);
    };

    serde_json::from_value(value).map(Some)
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use harper_core::DictWordMetadata;

    #[test]
    fn serialize_main_excludes_dictionary_word_list() {
        let mut config = Config::new();
        config
            .mutable_dictionary
            .append_word_str("blorple", DictWordMetadata::default());

        let serialized = config.serialize_main().unwrap();

        assert!(!serialized.contains("mutable_dictionary"));
        assert!(!serialized.contains("blorple"));
        assert!(serialized.contains("dialect"));
        assert!(serialized.contains("ignored_lints"));
        assert!(serialized.contains("lint_config"));
        assert!(serialized.contains("allowed_bundle_identifiers"));
    }

    #[test]
    fn deserialize_main_restores_main_serialized_fields() {
        let mut config = Config::new();
        config
            .mutable_dictionary
            .append_word_str("blorple", DictWordMetadata::default());
        let serialized = config.serialize_main().unwrap();

        let deserialized = Config::deserialize_main(&serialized).unwrap();

        assert_eq!(deserialized.dialect, config.dialect);
        assert_eq!(deserialized.lint_config, config.lint_config);
        assert_eq!(
            deserialized.allowed_bundle_identifiers,
            config.allowed_bundle_identifiers
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&deserialized.serialize_main().unwrap())
                .unwrap(),
            serde_json::from_str::<serde_json::Value>(&serialized).unwrap()
        );
    }

    #[test]
    fn new_uses_curated_allowed_bundle_identifiers() {
        let config = Config::new();

        assert_eq!(
            config.allowed_bundle_identifiers,
            Config::curated_allowed_bundle_identifiers()
        );
        assert!(config.is_approved_bundle_identifier("com.apple.TextEdit"));
    }

    #[test]
    fn deserialize_main_uses_curated_bundle_identifiers_when_missing() {
        let config = Config::new();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&config.serialize_main().unwrap()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("allowed_bundle_identifiers");

        let deserialized = Config::deserialize_main(&value.to_string()).unwrap();

        assert_eq!(
            deserialized.allowed_bundle_identifiers,
            Config::curated_allowed_bundle_identifiers()
        );
    }

    #[test]
    fn deserialize_main_preserves_allowed_bundle_identifiers() {
        let mut config = Config::new();
        config.allowed_bundle_identifiers = vec!["com.example.Editor".to_string()];

        let deserialized = Config::deserialize_main(&config.serialize_main().unwrap()).unwrap();

        assert_eq!(
            deserialized.allowed_bundle_identifiers,
            vec!["com.example.Editor".to_string()]
        );
    }

    #[test]
    fn allowed_bundle_identifier_helpers_add_remove_and_check() {
        let mut config = Config::new();

        config.add_allowed_bundle_identifier(" com.example.Editor ".to_string());
        config.add_allowed_bundle_identifier("com.example.Editor".to_string());
        assert!(config.is_approved_bundle_identifier("com.example.Editor"));
        assert_eq!(
            config
                .allowed_bundle_identifiers
                .iter()
                .filter(|bundle_identifier| bundle_identifier.as_str() == "com.example.Editor")
                .count(),
            1
        );

        config.remove_allowed_bundle_identifier("com.example.Editor");

        assert!(!config.is_approved_bundle_identifier("com.example.Editor"));
    }

    #[test]
    fn main_path_points_to_harper_desktop_config_file() {
        let path = Config::main_path().unwrap();

        assert_eq!(path.file_name().unwrap(), "config.json");
        assert_eq!(
            path.parent().unwrap().file_name().unwrap(),
            "harper-desktop"
        );
    }

    #[test]
    fn dictionary_path_points_to_harper_desktop_dictionary_file() {
        let path = Config::dictionary_path().unwrap();

        assert_eq!(path.file_name().unwrap(), "dictionary.txt");
        assert_eq!(
            path.parent().unwrap().file_name().unwrap(),
            "harper-desktop"
        );
    }
}
