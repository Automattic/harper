use harper_core::{
    Dialect, IgnoredLints,
    linting::{FlatConfig, LintGroup},
    spell::{FstDictionary, MergedDictionary, MutableDictionary},
};
use harper_dictionary_wordlist::{load_dict, save_dict};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeOwned, Error},
};
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
    pub integrations: Vec<Integration>,
    pub debounce_ms: u64,
    pub auto_update: bool,
    pub last_update_check: Option<u64>,
    pub highlighter_service_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integration {
    pub bundle_id: String,
    pub enabled: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            mutable_dictionary: MutableDictionary::new(),
            dialect: Self::detect_system_dialect(),
            ignored_lints: IgnoredLints::new(),
            lint_config: FlatConfig::new_curated(),
            integrations: Self::curated_integrations(),
            debounce_ms: 0,
            auto_update: true,
            last_update_check: None,
            highlighter_service_enabled: true,
        }
    }

    pub fn detect_system_dialect() -> Dialect {
        tauri_plugin_os::locale()
            .and_then(|bcp47| Dialect::try_from_bcp47(&bcp47))
            .unwrap_or(Dialect::American)
    }

    pub fn curated_integrations() -> Vec<Integration> {
        [
            "com.apple.TextEdit",
            "com.apple.mail",
            "com.apple.MobileSMS",
            "com.apple.Notes",
            "com.tinyspeck.slackmacgap",
            "com.hnc.Discord",
        ]
        .into_iter()
        .map(|bundle_id| Integration {
            bundle_id: bundle_id.to_string(),
            enabled: true,
        })
        .collect()
    }

    pub fn is_integration_enabled(&self, bundle_id: &str) -> bool {
        Self::is_integration_enabled_in(&self.integrations, bundle_id)
    }

    pub fn is_integration_enabled_in(integrations: &[Integration], bundle_id: &str) -> bool {
        integrations
            .iter()
            .any(|integration| integration.bundle_id == bundle_id && integration.enabled)
    }

    pub fn add_integration(&mut self, bundle_id: String) {
        let bundle_id = bundle_id.trim();
        if bundle_id.is_empty()
            || self
                .integrations
                .iter()
                .any(|item| item.bundle_id == bundle_id)
        {
            return;
        }

        self.integrations.push(Integration {
            bundle_id: bundle_id.to_string(),
            enabled: true,
        });
        self.integrations
            .sort_by(|a, b| a.bundle_id.cmp(&b.bundle_id));
    }

    pub fn remove_integration(&mut self, bundle_id: &str) {
        self.integrations
            .retain(|integration| integration.bundle_id != bundle_id);
    }

    pub fn set_integration_enabled(&mut self, bundle_id: &str, enabled: bool) {
        if let Some(integration) = self
            .integrations
            .iter_mut()
            .find(|integration| integration.bundle_id == bundle_id)
        {
            integration.enabled = enabled;
        }
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

    pub fn main_config_exists() -> Result<bool, ConfigError> {
        let main_path = Self::main_path().ok_or(ConfigError::ConfigDirUnavailable)?;

        match fs::metadata(main_path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn load_from_system() -> Result<Self, ConfigError> {
        let main_path = Self::main_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let dictionary_path = Self::dictionary_path().ok_or(ConfigError::ConfigDirUnavailable)?;
        let serialized = fs::read_to_string(main_path)?;
        let mut config = Self::deserialize_main(&serialized)?;
        config.lint_config.fill_with_curated();
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
            "integrations": &self.integrations,
            "debounce_ms": self.debounce_ms,
            "auto_update": self.auto_update,
            "last_update_check": self.last_update_check,
            "highlighter_service_enabled": self.highlighter_service_enabled,
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
            integrations: deserialize_optional_field(object, "integrations")?
                .unwrap_or_else(Self::curated_integrations),
            debounce_ms: deserialize_optional_field(object, "debounce_ms")?.unwrap_or(0),
            auto_update: deserialize_optional_field(object, "auto_update")?.unwrap_or(true),
            last_update_check: deserialize_optional_field::<Option<u64>>(
                object,
                "last_update_check",
            )?
            .flatten(),
            highlighter_service_enabled: deserialize_optional_field(
                object,
                "highlighter_service_enabled",
            )?
            .unwrap_or(true),
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
    use super::{Config, Integration};
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
        assert!(serialized.contains("integrations"));
        assert!(serialized.contains("debounce_ms"));
        assert!(serialized.contains("auto_update"));
        assert!(serialized.contains("last_update_check"));
        assert!(serialized.contains("highlighter_service_enabled"));
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
        assert_eq!(deserialized.integrations, config.integrations);
        assert_eq!(deserialized.debounce_ms, config.debounce_ms);
        assert_eq!(deserialized.auto_update, config.auto_update);
        assert_eq!(deserialized.last_update_check, config.last_update_check);
        assert_eq!(
            deserialized.highlighter_service_enabled,
            config.highlighter_service_enabled
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&deserialized.serialize_main().unwrap())
                .unwrap(),
            serde_json::from_str::<serde_json::Value>(&serialized).unwrap()
        );
    }

    #[test]
    fn new_uses_curated_integrations() {
        let config = Config::new();

        assert_eq!(config.integrations, Config::curated_integrations());
        assert!(config.is_integration_enabled("com.apple.TextEdit"));
        assert!(config.is_integration_enabled("com.apple.mail"));
        assert!(config.is_integration_enabled("com.apple.MobileSMS"));
        assert!(config.is_integration_enabled("com.apple.Notes"));
        assert!(config.is_integration_enabled("com.tinyspeck.slackmacgap"));
        assert!(config.is_integration_enabled("com.hnc.Discord"));
    }

    #[test]
    fn deserialize_main_uses_curated_integrations_when_missing() {
        let config = Config::new();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&config.serialize_main().unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("integrations");

        let deserialized = Config::deserialize_main(&value.to_string()).unwrap();

        assert_eq!(deserialized.integrations, Config::curated_integrations());
        assert!(deserialized.is_integration_enabled("com.tinyspeck.slackmacgap"));
        assert!(deserialized.is_integration_enabled("com.hnc.Discord"));
    }

    #[test]
    fn deserialize_main_uses_zero_debounce_when_missing() {
        let config = Config::new();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&config.serialize_main().unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("debounce_ms");

        let deserialized = Config::deserialize_main(&value.to_string()).unwrap();

        assert_eq!(deserialized.debounce_ms, 0);
    }

    #[test]
    fn deserialize_main_uses_default_update_settings_when_missing() {
        let config = Config::new();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&config.serialize_main().unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("auto_update");
        value.as_object_mut().unwrap().remove("last_update_check");

        let deserialized = Config::deserialize_main(&value.to_string()).unwrap();

        assert!(deserialized.auto_update);
        assert_eq!(deserialized.last_update_check, None);
    }

    #[test]
    fn deserialize_main_enables_highlighter_service_when_missing() {
        let config = Config::new();
        let mut value =
            serde_json::from_str::<serde_json::Value>(&config.serialize_main().unwrap()).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("highlighter_service_enabled");

        let deserialized = Config::deserialize_main(&value.to_string()).unwrap();

        assert!(deserialized.highlighter_service_enabled);
    }

    #[test]
    fn deserialize_main_preserves_integrations() {
        let mut config = Config::new();
        config.integrations = vec![Integration {
            bundle_id: "com.example.Editor".to_string(),
            enabled: false,
        }];

        let deserialized = Config::deserialize_main(&config.serialize_main().unwrap()).unwrap();

        assert_eq!(
            deserialized.integrations,
            vec![Integration {
                bundle_id: "com.example.Editor".to_string(),
                enabled: false,
            }]
        );
    }

    #[test]
    fn integration_helpers_add_remove_enable_and_check() {
        let mut config = Config::new();

        config.add_integration(" com.example.Editor ".to_string());
        config.add_integration("com.example.Editor".to_string());
        assert!(config.is_integration_enabled("com.example.Editor"));
        assert_eq!(
            config
                .integrations
                .iter()
                .filter(|integration| integration.bundle_id == "com.example.Editor")
                .count(),
            1
        );

        config.set_integration_enabled("com.example.Editor", false);
        assert!(!config.is_integration_enabled("com.example.Editor"));

        config.remove_integration("com.example.Editor");

        assert!(
            !config
                .integrations
                .iter()
                .any(|integration| integration.bundle_id == "com.example.Editor")
        );
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
    fn main_config_exists_reports_missing_file() {
        let exists = Config::main_config_exists().unwrap();
        let expected = Config::main_path().unwrap().exists();

        assert_eq!(exists, expected);
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
