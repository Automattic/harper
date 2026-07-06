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
    /// Module name for the language module (e.g., "EnglishModule")
    pub module_name: &'static str,
    /// Language aliases for parsing (e.g., ["de", "german", "deutsch"])
    pub aliases: Vec<&'static str>,
    /// Detector confidence value (0.0-1.0) for language detection ordering
    pub confidence: f64,
    /// Dialect alias groups: each group maps multiple aliases to a dialect variant
    /// e.g., [("us", vec!["us", "usa", "america", "american", "en-us", "en_us"]), "US")] for English
    pub dialect_alias_groups: Vec<(Vec<&'static str>, &'static str)>,
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

/// Get language-specific metadata and configuration
pub fn get_language_metadata(dir_name: &str) -> Option<(Vec<&'static str>, f64)> {
    match dir_name {
        "english" => Some((vec![
            "us", "usa", "america", "american", "en-us", "en_us",
            "uk", "gb", "british", "britain", "en-gb", "en_gb",
            "au", "aus", "australia", "australian", "en-au", "en_au",
            "in", "india", "indian", "bharat", "en-in", "en_in",
            "ca", "canada", "canadian", "en-ca", "en_ca",
        ], 0.30)),
        "german" => Some((vec![
            "de", "german", "deutsch", "de-de", "de_de",
            "at", "austria", "austrian", "de-at", "de_at",
            "ch", "switzerland", "swiss", "de-ch", "de_ch",
        ], 0.95)),
        "portuguese" => Some((vec![
            "pt", "pt-pt", "pt_pt", "portuguese", "portugu\u{00ea}s",
            "br", "brazil", "portuguese-brazilian", "portuguese_brazilian", "pt-br", "pt_br",
            "ao",
        ], 0.85)),
        "slovak" => Some((vec![
            "sk", "slovak", "slovensko", "sk-sk", "sk_sk",
        ], 0.90)),
        _ => None, // Unknown languages have no metadata
    }
}

/// Get language-specific dialect alias groups for parsing
/// Returns a vector of (aliases, dialect_name) where aliases is a group of alias strings
/// that all map to the same dialect variant name (what try_from_abbr expects)
#[allow(dead_code)]
pub fn get_language_dialect_alias_groups(dir_name: &str) -> Vec<(Vec<&'static str>, &'static str)> {
    match dir_name {
        "english" => vec![
            (vec!["us", "usa", "america", "american", "en-us", "en_us"], "US"),
            (vec!["uk", "gb", "british", "britain", "en-gb", "en_gb"], "GB"),
            (vec!["au", "aus", "australia", "australian", "en-au", "en_au"], "AU"),
            (vec!["in", "india", "indian", "bharat", "en-in", "en_in"], "IN"),
            (vec!["ca", "canada", "canadian", "en-ca", "en_ca"], "CA"),
        ],
        "german" => vec![
            (vec!["de", "german", "deutsch", "de-de", "de_de"], "Standard"),
            (vec!["at", "austria", "austrian", "de-at", "de_at"], "Austrian"),
            (vec!["ch", "switzerland", "swiss", "de-ch", "de_ch"], "Swiss"),
        ],
        "portuguese" => vec![
            (vec!["pt", "pt-pt", "pt_pt", "portuguese", "portugu\u{00ea}s"], "European"),
            (vec!["br", "brazil", "portuguese-brazilian", "portuguese_brazilian", "pt-br", "pt_br"], "Brazilian"),
            (vec!["ao"], "African"),
        ],
        "slovak" => vec![
            (vec!["sk", "slovak", "slovensko", "sk-sk", "sk_sk"], "Standard"),
        ],
        _ => vec![], // Unknown languages have no dialect aliases
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
        let module_name = format!("{}Module", capitalize_first_letter(&dir_name));
        
        // Get language-specific metadata
        let (aliases, confidence) = get_language_metadata(&dir_name)
            .unwrap_or_else(|| (vec![], 0.5)); // Default confidence for unknown languages
        
        // Get dialect alias groups
        let dialect_alias_groups = get_language_dialect_alias_groups(&dir_name);

        // Helper function to convert owned string to &'static str
        fn to_static_str(s: String) -> &'static str {
            Box::leak(s.into_boxed_str())
        }

        // Helper function to convert Vec<&'static str> to Vec<&'static str> with Box::leak
        fn to_static_str_vec_static(v: Vec<&'static str>) -> Vec<&'static str> {
            v.into_iter().map(|s| to_static_str(s.to_string())).collect()
        }

        // Helper function to convert Vec<(Vec<&'static str>, &'static str)> to Vec<(Vec<&'static str>, &'static str)>
        fn to_static_str_dialect_groups(v: Vec<(Vec<&'static str>, &'static str)>) -> Vec<(Vec<&'static str>, &'static str)> {
            v.into_iter().map(|(aliases, dialect)| {
                let aliases_static: Vec<&'static str> = aliases.into_iter().map(|a| to_static_str(a.to_string())).collect();
                let dialect_static = to_static_str(dialect.to_string());
                (aliases_static, dialect_static)
            }).collect()
        }

        // Convert feature to static reference
        let feature_static: Option<&'static str> = feature.map(|f| to_static_str(f.to_string()));
        
        // Convert aliases to static references
        let aliases_static = to_static_str_vec_static(aliases);
        
        // Convert dialect alias groups to static references
        let dialect_alias_groups_static = to_static_str_dialect_groups(dialect_alias_groups);

        // Store the owned strings in LanguageConfig using Box::leak
        languages.push(LanguageConfig {
            name: to_static_str(name),
            dir_name: to_static_str(dir_name.clone()),
            feature: feature_static,
            dialect_module: to_static_str(dialect_module),
            flags_module: to_static_str(flags_module),
            module_name: to_static_str(module_name),
            aliases: aliases_static,
            confidence,
            dialect_alias_groups: dialect_alias_groups_static,
        });
    }

    languages
}

/// Get configuration for all supported languages
#[allow(dead_code)]
pub fn get_supported_languages(manifest_dir: &Path) -> Vec<LanguageConfig> {
    discover_languages(manifest_dir)
}