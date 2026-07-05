use std::{fs, path::Path};

/// Configuration for a supported language
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LanguageConfig {
    /// Name of the language (e.g., "English", "German")
    pub name: &'static str,
    /// Directory name in the language folder (e.g., "english", "german")
    pub dir_name: &'static str,
    /// Cargo feature name (e.g., None for English, Some("de") for German)
    pub feature: Option<&'static str>,
    /// Module name for the dialect type (e.g., "GermanDialect")
    pub dialect_module: &'static str,
    /// Module name for the dialect flags type (e.g., "GermanDialectFlags")
    pub flags_module: &'static str,
}

/// Known non-language directories in src/language/ that should be skipped
pub fn get_non_language_directories() -> [&'static str; 2] {
    ["dialects", "testing_framework"]
}

/// Map directory names to Cargo feature names
pub fn map_directory_to_feature(dir_name: &str) -> Option<&'static str> {
    match dir_name {
        "german" => Some("de"),
        "portuguese" => Some("pt"),
        "slovak" => Some("sk"),
        "english" => None, // English is always included
        _ => None,         // Unknown languages have no feature (will be always included)
    }
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

        let name = capitalize_first_letter(&dir_name);
        let feature = map_directory_to_feature(&dir_name);
        let dialect_module = format!("{}Dialect", capitalize_first_letter(&dir_name));
        let flags_module = format!("{}DialectFlags", capitalize_first_letter(&dir_name));

        // Helper function to convert owned string to &'static str
        fn to_static_str(s: String) -> &'static str {
            Box::leak(s.into_boxed_str())
        }

        // Convert feature to static reference
        let feature_static: Option<&'static str> = feature.map(|f| to_static_str(f.to_string()));

        // Store the owned strings in LanguageConfig using Box::leak
        languages.push(LanguageConfig {
            name: to_static_str(name),
            dir_name: to_static_str(dir_name.clone()),
            feature: feature_static,
            dialect_module: to_static_str(dialect_module),
            flags_module: to_static_str(flags_module),
        });
    }

    languages
}

/// Get configuration for all supported languages
#[allow(dead_code)]
pub fn get_supported_languages(manifest_dir: &Path) -> Vec<LanguageConfig> {
    discover_languages(manifest_dir)
}