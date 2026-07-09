use std::{fs, path::Path};

/// Configuration for a supported language
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Name of the language (e.g., "English", "German")
    pub name: String,
    /// Directory name in the language folder (e.g., "english", "german")
    pub dir_name: String,
    /// Cargo feature name (e.g., None for English, Some("de") for German)
    pub feature: Option<String>,
    /// Module name for the dialect type (e.g., "GermanDialect")
    pub dialect_module: String,
    /// Module name for the dialect flags type (e.g., "GermanDialectFlags")
    pub flags_module: String,
    /// Module name for the language module (e.g., "EnglishModule")
    pub module_name: String,
    /// Detector confidence value (0.0-1.0) for language detection ordering
    pub confidence: f64,
    /// Dialect alias groups: each group maps multiple aliases to a dialect variant
    /// e.g., [("us", vec!["us", "usa", ...]), "US")] for English
    pub dialect_alias_groups: Vec<(Vec<String>, String)>,
    /// Optional subdirectory for weir rules (e.g., "de" for German)
    pub weir_rules_subdirectory: Option<String>,
}

/// Known non-language directories in src/language/ that should be skipped
pub fn get_non_language_directories() -> [&'static str; 2] {
    ["dialects", "testing_framework"]
}

/// Load language configuration from a config.toml file in the language directory
pub fn load_language_config(dir_path: &Path, dir_name: &str) -> Option<LanguageConfig> {
    let config_path = dir_path.join("config.toml");
    if !config_path.exists() {
        return None;
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Try to parse as TOML
    let value: toml::Value = match content.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Extract language section
    let language_table = match value.get("language") {
        Some(toml::Value::Table(t)) => t,
        _ => {
            eprintln!(
                "Warning: Missing [language] section in config.toml for {}",
                dir_path.display()
            );
            return None;
        }
    };

    let name = match language_table.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => {
            eprintln!(
                "Warning: Missing 'name' in [language] section for {}",
                dir_path.display()
            );
            return None;
        }
    };

    let feature = language_table
        .get("feature")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // If feature is explicitly "null" or empty, treat as None
    let feature = if feature
        .as_ref()
        .is_some_and(|f| f == "null" || f.is_empty())
    {
        None
    } else {
        feature
    };

    // Extract metadata section
    let metadata_table = match value.get("metadata") {
        Some(toml::Value::Table(t)) => t,
        _ => {
            eprintln!(
                "Warning: Missing [metadata] section in config.toml for {}",
                dir_path.display()
            );
            return None;
        }
    };

    let confidence = match metadata_table.get("confidence") {
        Some(toml::Value::Float(f)) => *f,
        Some(toml::Value::Integer(i)) => *i as f64,
        _ => {
            eprintln!(
                "Warning: Missing or invalid 'confidence' in [metadata] section for {}",
                dir_path.display()
            );
            return None;
        }
    };

    // Extract dialects section - can be either inline table with alias_groups or array of tables
    let dialect_alias_groups = if let Some(toml::Value::Array(dialects_array)) =
        value.get("dialects")
    {
        // Array of tables format: [[dialects]], [[dialects]], ...
        let mut result = Vec::new();
        for dialect_table in dialects_array {
            if let toml::Value::Table(table) = dialect_table
                && let (Some(toml::Value::String(name)), Some(toml::Value::Array(aliases_arr))) =
                    (table.get("name"), table.get("aliases"))
            {
                let aliases: Vec<String> = aliases_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                result.push((aliases, name.clone()));
            }
        }
        result
    } else if let Some(toml::Value::Table(dialects_table)) = value.get("dialects") {
        // Inline table format with alias_groups
        if let Some(toml::Value::Table(groups)) = dialects_table.get("alias_groups") {
            let mut result = Vec::new();
            let mut keys: Vec<_> = groups.keys().cloned().collect();
            keys.sort();

            for dialect_name in keys {
                if let Some(toml::Value::Array(aliases_arr)) = groups.get(&dialect_name) {
                    let aliases: Vec<String> = aliases_arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    result.push((aliases, dialect_name.clone()));
                }
            }
            result
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Extract weir section (optional)
    let weir_rules_subdirectory = value
        .get("weir")
        .and_then(|w| w.get("rules_subdirectory"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Generate derived names
    let dialect_module = format!("{}Dialect", capitalize_first_letter(&name));
    let flags_module = format!("{}DialectFlags", capitalize_first_letter(&name));
    let module_name = format!("{}Module", capitalize_first_letter(&name));

    Some(LanguageConfig {
        name,
        dir_name: dir_name.to_string(),
        feature,
        dialect_module,
        flags_module,
        module_name,
        confidence,
        dialect_alias_groups,
        weir_rules_subdirectory,
    })
}

/// Capitalize the first letter of a string
pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Discover all supported languages by scanning the language directory
pub fn discover_languages(manifest_dir: &Path) -> Vec<LanguageConfig> {
    let language_dir = manifest_dir.join("src/language");
    let mut languages = Vec::new();
    let non_language_dirs = get_non_language_directories();

    // Check if language directory exists
    if !language_dir.exists() || !language_dir.is_dir() {
        eprintln!(
            "Warning: Language directory not found at {}",
            language_dir.display()
        );
        return languages;
    }

    // Read directory entries
    let entries = match fs::read_dir(&language_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Warning: Could not read language directory: {}", e);
            return languages;
        }
    };

    // Sort entries for deterministic ordering
    let mut sorted_entries: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();
    sorted_entries.sort_by_key(|entry| entry.path());

    for entry in sorted_entries {
        let dir_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Skip non-language directories
        if non_language_dirs.contains(&dir_name.as_str()) {
            continue;
        }

        // Skip directories that don't contain language modules
        // Look for module.rs file as a heuristic
        let module_path = entry.path().join("module.rs");
        if !module_path.exists() {
            continue;
        }

        // Load configuration from config.toml
        if let Some(config) = load_language_config(&entry.path(), &dir_name) {
            languages.push(config);
        }
    }

    languages
}
