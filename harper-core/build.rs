use std::{env, fs, path::Path};

/// Configuration for a supported language
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LanguageConfig {
    /// Name of the language (e.g., "English", "German")
    name: &'static str,
    /// Directory name in the language folder (e.g., "english", "german")
    dir_name: &'static str,
    /// Cargo feature name (e.g., None for English, Some("de") for German)
    feature: Option<&'static str>,
    /// Module name for the dialect type (e.g., "GermanDialect")
    dialect_module: &'static str,
    /// Module name for the dialect flags type (e.g., "GermanDialectFlags")
    flags_module: &'static str,
}

/// Known non-language directories in src/language/ that should be skipped
fn get_non_language_directories() -> [&'static str; 2] {
    ["dialects", "testing_framework"]
}

/// Map directory names to Cargo feature names
fn map_directory_to_feature(dir_name: &str) -> Option<&'static str> {
    match dir_name {
        "german" => Some("de"),
        "portuguese" => Some("pt"),
        "slovak" => Some("sk"),
        "english" => None, // English is always included
        _ => None,         // Unknown languages have no feature (will be always included)
    }
}

/// Capitalize the first letter of a string
fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Discover all supported languages by scanning the language directory
fn discover_languages(manifest_dir: &Path) -> Vec<LanguageConfig> {
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
fn get_supported_languages(manifest_dir: &Path) -> Vec<LanguageConfig> {
    discover_languages(manifest_dir)
}

#[derive(Debug)]
struct StandaloneRule {
    name: String,
    relative_path: String,
}

#[derive(Debug)]
struct GroupedRule {
    public_name: String,
    children: Vec<StandaloneRule>,
}

/// Convert a Weir rule path to an `include_str!`-friendly relative path.
fn path_as_weir_relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

/// Top-level `RuleName.weir` files are public as `RuleName`.
fn rule_name_from_path(path: &Path) -> String {
    path.file_stem().unwrap().to_string_lossy().to_string()
}

/// Grouped child rules use their relative path as a private name.
fn rule_name_from_relative_path(path: &Path, root: &Path) -> String {
    let mut relative = path_as_weir_relative(path, root);
    relative.truncate(relative.len() - ".weir".len());
    relative
}

/// Recursively collect child `.weir` files for a grouped rule directory.
fn collect_weir_files(dir: &Path, group_root: &Path, weir_root: &Path) -> Vec<StandaloneRule> {
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut entries = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.path());

    let mut rules = Vec::new();

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().unwrap();

        if file_type.is_dir() {
            rules.extend(collect_weir_files(&path, group_root, weir_root));
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "weir") {
            println!("cargo:rerun-if-changed={}", path.display());

            rules.push(StandaloneRule {
                name: rule_name_from_relative_path(&path, group_root),
                relative_path: path_as_weir_relative(&path, weir_root),
            });
        }
    }

    rules
}

/// Render a string as an escaped Rust string literal for generated source.
fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn write_grouped_weir_boilerplate(weir_rule_dir: &Path, dest: &Path) {
    let mut entries = fs::read_dir(weir_rule_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.path());

    let mut standalone_rules = Vec::new();
    let mut grouped_rules = Vec::new();

    // Watch the root for top-level `.weir` files and group directories.
    println!("cargo:rerun-if-changed={}", weir_rule_dir.display());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().unwrap();

        if file_type.is_dir() {
            let public_name = entry.file_name().to_string_lossy().to_string();
            let children = collect_weir_files(&path, &path, weir_rule_dir);

            if !children.is_empty() {
                grouped_rules.push(GroupedRule {
                    public_name,
                    children,
                });
            }
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "weir") {
            println!("cargo:rerun-if-changed={}", path.display());

            standalone_rules.push(StandaloneRule {
                name: rule_name_from_path(&path),
                relative_path: path_as_weir_relative(&path, weir_rule_dir),
            });
        }
    }

    let mut code = String::new();

    code.push_str("generate_boilerplate! {\n");
    code.push_str("    standalone: [\n");
    for rule in standalone_rules {
        code.push_str(&format!(
            "        ({}, {}),\n",
            rust_string_literal(&rule.name),
            rust_string_literal(&rule.relative_path)
        ));
    }
    code.push_str("    ],\n");

    code.push_str("    groups: [\n");
    for group in grouped_rules {
        code.push_str(&format!(
            "        ({}, [\n",
            rust_string_literal(&group.public_name)
        ));

        for child in group.children {
            code.push_str(&format!(
                "            ({}, {}),\n",
                rust_string_literal(&child.name),
                rust_string_literal(&child.relative_path)
            ));
        }

        code.push_str("        ]),\n");
    }
    code.push_str("    ],\n");
    code.push_str("}\n");

    fs::write(dest, code).unwrap();
}

fn write_flat_weir_boilerplate(weir_rule_dir: &Path, dest: &Path) {
    let mut files = match fs::read_dir(weir_rule_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "weir"))
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    files.sort();

    let mut code = String::new();
    code.push_str("generate_boilerplate!{[");

    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        code.push_str(&format!(
            "{},\n",
            file.file_stem().unwrap().to_string_lossy()
        ));
    }

    code.push_str("]}");
    fs::write(dest, code).unwrap();
}

fn process_language_weir_rules(language_dir: &Path, out_dir: &Path) {
    if let Ok(language_entries) = fs::read_dir(language_dir) {
        for language_entry in language_entries.filter_map(Result::ok) {
            let language_path = language_entry.path();
            if !language_path.is_dir() {
                continue;
            }

            // Extract language name from directory
            let language_name = language_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_uppercase());

            if language_name.is_none() {
                continue;
            }

            let language_name = language_name.unwrap();
            let weir_rules_dir = language_path.join("linting/weir_rules");

            // Check if the weir_rules directory exists
            if !weir_rules_dir.exists() {
                continue;
            }

            // For German, look in the 'de' subdirectory for locale-specific rules
            let final_weir_dir = if language_name == "GERMAN" {
                weir_rules_dir.join("de")
            } else {
                weir_rules_dir
            };

            // Only process if the directory exists and contains .weir files
            if !final_weir_dir.exists() {
                continue;
            }

            let has_weir_files = fs::read_dir(&final_weir_dir)
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .any(|entry| entry.path().extension().is_some_and(|ext| ext == "weir"))
                })
                .unwrap_or(false);

            if !has_weir_files {
                // Always generate the infrastructure even if empty
                let lowercase_name = language_name.to_lowercase();
                let dest = out_dir.join(format!("{}_weir_rules_generated_list.rs", lowercase_name));
                fs::write(&dest, "generate_boilerplate!{[]}").unwrap();
                println!(
                    "cargo:rustc-env={}_WEIR_RULE_DIR={}",
                    language_name,
                    final_weir_dir.display()
                );
                println!(
                    "cargo:rustc-env={}_WEIR_RULE_LIST={}",
                    language_name,
                    dest.display()
                );
                continue;
            }

            let lowercase_name = language_name.to_lowercase();
            let dest = out_dir.join(format!("{}_weir_rules_generated_list.rs", lowercase_name));

            write_flat_weir_boilerplate(&final_weir_dir, &dest);

            println!(
                "cargo:rustc-env={}_WEIR_RULE_DIR={}",
                language_name,
                final_weir_dir.display()
            );
            println!(
                "cargo:rustc-env={}_WEIR_RULE_LIST={}",
                language_name,
                dest.display()
            );
        }
    }
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();

    // Main English weir rules (in linting/weir_rules/)
    let english_weir_rule_dir = manifest_dir.join("./src/linting/weir_rules");
    let english_dest = out_dir.join("weir_rules_generated_list.rs");
    write_grouped_weir_boilerplate(&english_weir_rule_dir, &english_dest);
    println!(
        "cargo:rustc-env=WEIR_RULE_DIR={}",
        english_weir_rule_dir.display()
    );
    println!("cargo:rustc-env=WEIR_RULE_LIST={}", english_dest.display());

    // Language-specific weir rules (in language/<name>/linting/weir_rules/)
    // Automatically discover all language directories that have weir_rules
    let language_dir = manifest_dir.join("./src/language");
    process_language_weir_rules(&language_dir, &out_dir);

    println!("cargo:rerun-if-changed=build.rs");

    // Generate language module and related files
    generate_language_modules(&out_dir);
}

/// Generate language module file that contains all language-specific code
/// This consolidates all the #[cfg(feature)] attributes into one place
fn generate_language_modules(_out_dir: &Path) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src/language");

    let mut code = String::new();

    // Add header
    code.push_str("// Auto-generated by build.rs - do not edit manually\n");
    code.push_str("// This file centralizes all language-specific conditional compilation\n\n");

    // Generate module declarations
    code.push_str("pub mod dialects;\n");
    code.push_str("pub mod languages;\n");
    code.push_str("pub mod module;\n");
    code.push_str("pub mod registry;\n\n");

    // English is always included
    code.push_str("pub mod english;\n\n");

    // Optional language modules
    code.push_str("#[cfg(feature = \"de\")]\n");
    code.push_str("pub mod german;\n\n");

    code.push_str("#[cfg(feature = \"pt\")]\n");
    code.push_str("pub mod portuguese;\n\n");

    code.push_str("#[cfg(feature = \"sk\")]\n");
    code.push_str("pub mod slovak;\n\n");

    // Re-exports
    code.push_str("pub use languages::{Language, LanguageFamily, parse_language};\n");
    code.push_str("pub use module::{LanguageDetector, LanguageModule};\n");
    code.push_str("pub use registry::{\n");
    code.push_str(
        "    ProseLanguage, add_language_specific_linters, detect_language, dictionary,\n",
    );
    code.push_str("    dictionary_for_language, new_curated_for_language, parser_for_prose, prose_language,\n");
    code.push_str("    weir_rules_lint_group,\n");
    code.push_str("};\n");

    // Write directly to mod.rs to replace the hand-written file
    let dest = src_dir.join("mod.rs");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(&dest, code).unwrap();

    // Also generate languages.rs
    generate_languages_file(&src_dir);

    println!("cargo:rerun-if-changed=src/language/mod.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Track dialect flags file for regeneration
    println!("cargo:rerun-if-changed=src/language/dialects/dialect_flags.rs");
}

/// Generate languages.rs with all the language-related code
fn generate_languages_file(src_dir: &Path) {
    let mut code = String::new();

    code.push_str("//! Language support framework for Harper.\n");
    code.push_str("//!\n");
    code.push_str(
        "//! This module provides the core types for supporting multiple languages in Harper,\n",
    );
    code.push_str(
        "//! including language families and specific language variants with dialects.\n",
    );
    code.push_str("use crate::language::english::dialects::EnglishDialect;\n\n");

    code.push_str("#[cfg(feature = \"de\")]\n");
    code.push_str("use crate::language::german::dialects::GermanDialect;\n\n");

    code.push_str("#[cfg(feature = \"pt\")]\n");
    code.push_str("use crate::language::portuguese::dialects::PortugueseDialect;\n\n");

    code.push_str("#[cfg(feature = \"sk\")]\n");
    code.push_str("use crate::language::slovak::dialects::SlovakDialect;\n");
    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use strum_macros::{Display, EnumCount, EnumIter, EnumString};\n\n");

    // parse_language function
    code.push_str("/// Parse a language from a string representation.\n");
    code.push_str("pub fn parse_language(s: &str) -> Option<Language> {\n");
    code.push_str("    let s_lower = s.to_ascii_lowercase();\n\n");
    code.push_str("    match s_lower.as_str() {\n");
    code.push_str("        // English\n");
    code.push_str(
        "        \"us\" | \"usa\" | \"america\" | \"american\" | \"en-us\" | \"en_us\" => {\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::American))\n");
    code.push_str("        }\n");
    code.push_str(
        "        \"uk\" | \"gb\" | \"british\" | \"britain\" | \"en-gb\" | \"en_gb\" => {\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::British))\n");
    code.push_str("        }\n");
    code.push_str(
        "        \"au\" | \"aus\" | \"australia\" | \"australian\" | \"en-au\" | \"en_au\" => {\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::Australian))\n");
    code.push_str("        }\n");
    code.push_str(
        "        \"in\" | \"india\" | \"indian\" | \"bharat\" | \"en-in\" | \"en_in\" => {\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::Indian))\n");
    code.push_str("        }\n");
    code.push_str("        \"ca\" | \"canada\" | \"canadian\" | \"en-ca\" | \"en_ca\" => {\n");
    code.push_str("            Some(Language::English(EnglishDialect::Canadian))\n");
    code.push_str("        }\n");

    code.push_str("        // German\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"de\" | \"german\" | \"deutsch\" | \"de-de\" | \"de_de\" => {\n");
    code.push_str("            Some(Language::German(GermanDialect::Standard))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"at\" | \"austria\" | \"austrian\" | \"de-at\" | \"de_at\" => {\n");
    code.push_str("            Some(Language::German(GermanDialect::Austrian))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"ch\" | \"switzerland\" | \"swiss\" | \"de-ch\" | \"de_ch\" => {\n");
    code.push_str("            Some(Language::German(GermanDialect::Swiss))\n");
    code.push_str("        }\n");

    code.push_str("        // Portuguese\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str(
        "        \"pt\" | \"pt-pt\" | \"pt_pt\" | \"portuguese\" | \"portugu\\u{00ea}s\" => {\n",
    );
    code.push_str("            Some(Language::Portuguese(PortugueseDialect::European))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        \"br\" | \"brazil\" | \"portuguese-brazilian\" | \"portuguese_brazilian\" | \"pt-br\" | \"pt_br\" => {\n");
    code.push_str("            Some(Language::Portuguese(PortugueseDialect::Brazilian))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        \"ao\" => Some(Language::Portuguese(PortugueseDialect::African)),\n");

    code.push_str("        // Slovak\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        \"sk\" | \"slovak\" | \"slovensko\" | \"sk-sk\" | \"sk_sk\" => {\n");
    code.push_str("            Some(Language::Slovak(SlovakDialect::Standard))\n");
    code.push_str("        }\n");

    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Language enum
    code.push_str("/// A specific language with its dialects.\n");
    code.push_str("#[derive(\n");
    code.push_str("    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, EnumCount, Display,\n");
    code.push_str(")]\n");
    code.push_str("pub enum Language {\n");
    code.push_str("    /// English language with its dialects\n");
    code.push_str("    English(EnglishDialect),\n");
    code.push_str("    /// German language with its dialects\n");
    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    German(GermanDialect),\n");
    code.push_str("    /// Portuguese language with its dialects\n");
    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    Portuguese(PortugueseDialect),\n");
    code.push_str("    /// Slovak language with its dialects\n");
    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    Slovak(SlovakDialect),\n");
    code.push_str("}\n\n");

    // LanguageFamily enum
    code.push_str("/// A family of languages.\n");
    code.push_str("#[derive(\n");
    code.push_str("    Default,\n");
    code.push_str("    Debug,\n");
    code.push_str("    Clone,\n");
    code.push_str("    Copy,\n");
    code.push_str("    Serialize,\n");
    code.push_str("    Deserialize,\n");
    code.push_str("    PartialEq,\n");
    code.push_str("    PartialOrd,\n");
    code.push_str("    Eq,\n");
    code.push_str("    Hash,\n");
    code.push_str("    EnumCount,\n");
    code.push_str("    EnumString,\n");
    code.push_str("    EnumIter,\n");
    code.push_str("    Display,\n");
    code.push_str(")]\n");
    code.push_str("pub enum LanguageFamily {\n");
    code.push_str("    #[default]\n");
    code.push_str("    English,\n");
    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    German,\n");
    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    Portuguese,\n");
    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    Slovak,\n");
    code.push_str("}\n\n");

    // From<Language> for LanguageFamily
    code.push_str("impl From<Language> for LanguageFamily {\n");
    code.push_str("    fn from(value: Language) -> Self {\n");
    code.push_str("        match value {\n");
    code.push_str("            Language::English(_) => Self::English,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            Language::German(_) => Self::German,\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            Language::Portuguese(_) => Self::Portuguese,\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            Language::Slovak(_) => Self::Slovak,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // LanguageFamily impl
    code.push_str("impl LanguageFamily {\n");
    code.push_str("    pub fn dict_suffix(&self) -> &'static str {\n");
    code.push_str("        match self {\n");
    code.push_str("            Self::English => \"\",\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            Self::German => \"-de\",\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            Self::Portuguese => \"-pt\",\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            Self::Slovak => \"-sk\",\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Language impl
    code.push_str("impl Language {\n");
    code.push_str("    pub fn family(&self) -> LanguageFamily {\n");
    code.push_str("        (*self).into()\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("impl Default for Language {\n");
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        Self::English(EnglishDialect::American)\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    let dest = src_dir.join("languages.rs");
    fs::write(&dest, code).unwrap();

    // Also generate registry.rs
    generate_registry_file(src_dir);
}

/// Generate registry.rs with all language-specific registry code
fn generate_registry_file(src_dir: &Path) {
    let mut code = String::new();

    code.push_str(
        "//! Language registry - central integration point using LanguageModule trait.\n",
    );
    code.push_str("//!\n");
    code.push_str("//! This module provides all orchestration functions for language support.\n");
    code.push_str(
        "//! It is the only place that imports concrete language module implementations.\n\n",
    );

    code.push_str("use std::fmt::Debug;\n");
    code.push_str("use std::sync::{Arc, LazyLock};\n\n");

    code.push_str("use super::languages::{Language, LanguageFamily};\n");
    code.push_str("use crate::spell::{Dictionary, FstDictionary};\n");
    code.push_str("use crate::{\n");
    code.push_str("    LintGroup,\n");
    code.push_str("    parsers::{Markdown, MarkdownOptions, OrgMode, Parser},\n");
    code.push_str("};\n\n");

    code.push_str("use super::english::module::EnglishModule;\n");
    code.push_str("use super::module::{LanguageDetector, LanguageModule};\n\n");

    code.push_str("#[cfg(feature = \"de\")]\n");
    code.push_str("use super::german::module::GermanModule;\n\n");

    code.push_str("#[cfg(feature = \"pt\")]\n");
    code.push_str("use super::portuguese::module::PortugueseModule;\n\n");

    code.push_str("#[cfg(feature = \"sk\")]\n");
    code.push_str("use super::slovak::module::SlovakModule;\n\n");

    // DETECTION
    code.push_str("/// All language detectors, sorted by confidence (highest to lowest).\n");
    code.push_str("#[allow(clippy::vec_init_then_push)]\n");
    code.push_str(
        "static DETECTORS: LazyLock<Vec<(Box<dyn LanguageDetector>, f64)>> = LazyLock::new(|| {\n",
    );
    code.push_str("    let mut detectors: Vec<(Box<dyn LanguageDetector>, f64)> = Vec::new();\n\n");

    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    detectors.push((Box::new(GermanModule::detector()), 0.95));\n\n");

    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    detectors.push((Box::new(SlovakModule::detector()), 0.90));\n\n");

    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    detectors.push((Box::new(PortugueseModule::detector()), 0.85));\n\n");

    code.push_str("    detectors.push((Box::new(EnglishModule::detector()), 0.30));\n\n");
    code.push_str("    detectors\n");
    code.push_str("});\n\n");

    // detect_language function
    code.push_str("/// Detect the language of the given source text.\n");
    code.push_str("pub fn detect_language(source: &str, dict: &FstDictionary, default_language: Language) -> Language {\n");
    code.push_str("    use crate::parsers::PlainEnglish;\n\n");
    code.push_str("    let source_chars: Vec<char> = source.chars().collect();\n");
    code.push_str("    let tokens = PlainEnglish.parse(&source_chars);\n\n");
    code.push_str("    if tokens.is_empty() {\n");
    code.push_str("        return default_language;\n");
    code.push_str("    }\n\n");
    code.push_str("    for (detector, _confidence) in DETECTORS.iter() {\n");
    code.push_str(
        "        if let Some(language) = detector.detect(&tokens, &source_chars, dict) {\n",
    );
    code.push_str("            return language;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("    default_language\n");
    code.push_str("}\n\n");

    // PROSE LANGUAGE
    code.push_str("/// Prose languages supported by Harper for text parsing.\n");
    code.push_str("#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n");
    code.push_str("pub enum ProseLanguage {\n");
    code.push_str("    English,\n");
    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    German,\n");
    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    Portuguese,\n");
    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    Slovak,\n");
    code.push_str("}\n\n");

    code.push_str("/// Convert a Harper Language to a ProseLanguage.\n");
    code.push_str("pub fn prose_language(language: &Language) -> ProseLanguage {\n");
    code.push_str("    match language {\n");
    code.push_str("        Language::English(_) => ProseLanguage::English,\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        Language::German(_) => ProseLanguage::German,\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        Language::Portuguese(_) => ProseLanguage::Portuguese,\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        Language::Slovak(_) => ProseLanguage::Slovak,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // DICTIONARIES
    code.push_str("/// Get the dictionary for a language family.\n");
    code.push_str(
        "pub fn dictionary_for_language(family: LanguageFamily) -> Arc<FstDictionary> {\n",
    );
    code.push_str("    match family {\n");
    code.push_str("        LanguageFamily::English => EnglishModule::dictionary(),\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        LanguageFamily::German => GermanModule::dictionary(),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        LanguageFamily::Portuguese => PortugueseModule::dictionary(),\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        LanguageFamily::Slovak => SlovakModule::dictionary(),\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("/// Get the dictionary for a language.\n");
    code.push_str("pub fn dictionary(language: Language) -> Arc<FstDictionary> {\n");
    code.push_str("    dictionary_for_language(language.family())\n");
    code.push_str("}\n\n");

    // PARSERS - this is complex, let me include the original file for now
    // For now, I'll just note that this needs to be generated too
    code.push_str("// ========== PARSERS ==========\n\n");
    code.push_str("/// Get a parser for the given language ID and language.\n");
    code.push_str("pub fn parser_for_prose(\n");
    code.push_str("    language_id: &str,\n");
    code.push_str("    language: Language,\n");
    code.push_str("    markdown_options: MarkdownOptions,\n");
    code.push_str(") -> Option<Box<dyn Parser>> {\n");
    code.push_str("    match (language_id, prose_language(&language)) {\n");
    code.push_str("        (\"mail\", ProseLanguage::English) => Some(Box::new(EnglishModule::plain_parser())),\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        (\"mail\", ProseLanguage::German) => Some(Box::new(GermanModule::plain_parser())),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        (\"mail\", ProseLanguage::Portuguese) => Some(Box::new(PortugueseModule::plain_parser())),\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        (\"mail\", ProseLanguage::Slovak) => Some(Box::new(SlovakModule::plain_parser())),\n");
    code.push('\n');
    code.push_str("        // Markdown/Quarto format\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        (\"markdown\" | \"quarto\", ProseLanguage::German) => Some(Box::new(\n");
    code.push_str("            Markdown::with_inline_parser(markdown_options, |source| {\n");
    code.push_str("                GermanModule::plain_parser().parse(source)\n");
    code.push_str("            }),\n");
    code.push_str("        )),\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        (\"markdown\" | \"quarto\", ProseLanguage::Slovak) => Some(Box::new(\n");
    code.push_str("            Markdown::with_inline_parser(markdown_options, |source| {\n");
    code.push_str("                SlovakModule::plain_parser().parse(source)\n");
    code.push_str("            }),\n");
    code.push_str("        )),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str(
        "        (\"markdown\" | \"quarto\", ProseLanguage::Portuguese) => Some(Box::new(\n",
    );
    code.push_str("            Markdown::with_inline_parser(markdown_options, |source| {\n");
    code.push_str("                PortugueseModule::plain_parser().parse(source)\n");
    code.push_str("            }),\n");
    code.push_str("        )),\n");
    code.push_str("        (\"markdown\" | \"quarto\", _) => Some(Box::new(Markdown::new(markdown_options))),\n");
    code.push('\n');
    code.push_str("        // Org mode format\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        (\"org\", ProseLanguage::German) => Some(Box::new(OrgMode::with_inline_parser(|source| {\n");
    code.push_str("            GermanModule::plain_parser().parse(source)\n");
    code.push_str("        }))),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        (\"org\", ProseLanguage::Portuguese) => {\n");
    code.push_str("            Some(Box::new(OrgMode::with_inline_parser(|source| {\n");
    code.push_str("                PortugueseModule::plain_parser().parse(source)\n");
    code.push_str("            })))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        (\"org\", ProseLanguage::Slovak) => Some(Box::new(OrgMode::with_inline_parser(|source| {\n");
    code.push_str("            SlovakModule::plain_parser().parse(source)\n");
    code.push_str("        }))),\n");
    code.push_str("        (\"org\", _) => Some(Box::new(OrgMode::default())),\n");
    code.push('\n');
    code.push_str("        // Plain text format\n");
    code.push_str("        (\"plaintext\" | \"text\", ProseLanguage::English) => {\n");
    code.push_str("            Some(Box::new(EnglishModule::plain_parser()))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        (\"plaintext\" | \"text\", ProseLanguage::German) => {\n");
    code.push_str("            Some(Box::new(GermanModule::plain_parser()))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        (\"plaintext\" | \"text\", ProseLanguage::Portuguese) => {\n");
    code.push_str("            Some(Box::new(PortugueseModule::plain_parser()))\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        (\"plaintext\" | \"text\", ProseLanguage::Slovak) => {\n");
    code.push_str("            Some(Box::new(SlovakModule::plain_parser()))\n");
    code.push_str("        }\n");
    code.push_str("        _ => None,\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // LINTERS
    code.push_str("// ========== LINTERS ==========\n\n");
    code.push_str("/// Add language-specific linters to the lint group.\n");
    code.push_str("pub fn add_language_specific_linters(\n");
    code.push_str("    out: &mut LintGroup,\n");
    code.push_str("    language: Language,\n");
    code.push_str("    dictionary: Arc<impl Dictionary + 'static>,\n");
    code.push_str(") {\n");
    code.push_str("    match language {\n");
    code.push_str("        Language::English(_dialect) => {\n");
    code.push_str("            let lang_group = EnglishModule::rust_lint_group(dictionary);\n");
    code.push_str("            out.merge_from(lang_group);\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        Language::German(_dialect) => {\n");
    code.push_str("            let lang_group = GermanModule::rust_lint_group(dictionary);\n");
    code.push_str("            out.merge_from(lang_group);\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        Language::Portuguese(_dialect) => {\n");
    code.push_str("            let lang_group = PortugueseModule::rust_lint_group(dictionary);\n");
    code.push_str("            out.merge_from(lang_group);\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        Language::Slovak(_dialect) => {\n");
    code.push_str("            let lang_group = SlovakModule::rust_lint_group(dictionary);\n");
    code.push_str("            out.merge_from(lang_group);\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // WIR RULES
    code.push_str("// ========== WIR RULES ==========\n\n");
    code.push_str("/// Get the Weir rule lint group for a specific language.\n");
    code.push_str("pub fn weir_rules_lint_group(language: Language) -> LintGroup {\n");
    code.push_str("    match language {\n");
    code.push_str("        Language::English(_) => EnglishModule::weir_lint_group(),\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        Language::German(_) => GermanModule::weir_lint_group(),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        Language::Portuguese(_) => PortugueseModule::weir_lint_group(),\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        Language::Slovak(_) => SlovakModule::weir_lint_group(),\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // CURATED LINT GROUPS
    code.push_str("// ========== CURATED LINT GROUPS ==========\n\n");
    code.push_str(
        "/// Create a new curated lint group for a specific language with a custom dictionary.\n",
    );
    code.push_str("pub fn new_curated_for_language(\n");
    code.push_str("    _dictionary: Arc<impl Dictionary + 'static>,\n");
    code.push_str("    language: Language,\n");
    code.push_str(") -> LintGroup {\n");
    code.push_str("    use crate::language::module::LanguageModule;\n\n");
    code.push_str("    match language {\n");
    code.push_str("        Language::English(_dialect) => {\n");
    code.push_str("            #[allow(clippy::let_and_return)]\n");
    code.push_str("            let group = EnglishModule::curated_lint_group(_dialect);\n");
    code.push_str("            group\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        Language::German(_dialect) => {\n");
    code.push_str("            use crate::language::german::linting::new_curated_german;\n");
    code.push_str("            new_curated_german(_dialect)\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        Language::Portuguese(_dialect) => {\n");
    code.push_str("            use crate::language::portuguese::module::PortugueseModule;\n\n");
    code.push_str("            let lang_dict = PortugueseModule::dictionary();\n");
    code.push_str("            let mut group = LintGroup::empty();\n");
    code.push_str("            group.merge_from(PortugueseModule::weir_lint_group());\n");
    code.push_str("            group.merge_from(PortugueseModule::rust_lint_group(lang_dict));\n");
    code.push_str("            group.set_all_rules_to(Some(true));\n");
    code.push_str("            group\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        Language::Slovak(_dialect) => {\n");
    code.push_str("            use crate::language::slovak::module::SlovakModule;\n\n");
    code.push_str("            let lang_dict = SlovakModule::dictionary();\n");
    code.push_str("            let mut group = LintGroup::empty();\n");
    code.push_str("            group.merge_from(SlovakModule::weir_lint_group());\n");
    code.push_str("            group.merge_from(SlovakModule::rust_lint_group(lang_dict));\n");
    code.push_str("            group.set_all_rules_to(Some(true));\n");
    code.push_str("            group\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    let dest = src_dir.join("registry.rs");
    fs::write(&dest, code).unwrap();

    // Generate dialect flags
    generate_dialect_flags_file(src_dir);
}

/// Generate dialect_flags.rs with dynamic dialect flags collection
fn generate_dialect_flags_file(src_dir: &Path) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let languages = get_supported_languages(manifest_dir);
    let mut code = String::new();

    // Header
    code.push_str("// Auto-generated by build.rs - do not edit manually\n");
    code.push_str(
        "// This file provides a centralized DialectFlags struct for all supported languages.\n",
    );
    code.push_str(
        "// Language-specific dialect flags are defined in each language's dialects.rs file.\n\n",
    );

    // Imports
    code.push_str("use serde::ser::SerializeStruct;\n");
    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use serde_json::Value;\n\n");

    // Import dialect types - these come from individual language modules
    code.push_str("#[allow(unused_imports)]\n");
    code.push_str("use crate::language::dialects::dialect_trait::DialectFlags as _;\n");

    // Add imports for each language
    for lang in &languages {
        if let Some(feature) = &lang.feature {
            code.push_str(&format!("#[cfg(feature = \"{}\")]\n", feature));
        }
        code.push_str(&format!(
            "use crate::language::{}::dialects::{{{}, {}}};\n",
            lang.dir_name, lang.dialect_module, lang.flags_module
        ));
        if lang.feature.is_none() {
            code.push('\n');
        }
    }

    code.push('\n');

    // Main DialectFlags struct
    code.push_str(
        "/// This represents a collection of dialect flags for all supported languages.\n",
    );
    code.push_str("/// Each language has its own set of dialect flags.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash)]\n");
    code.push_str("pub struct DialectFlags {\n");

    // Add struct fields
    for lang in &languages {
        if let Some(feature) = &lang.feature {
            code.push_str(&format!("    #[cfg(feature = \"{}\")]\n", feature));
        }
        code.push_str(&format!(
            "    pub {}: {},\n",
            lang.dir_name.to_lowercase(),
            lang.flags_module
        ));
    }

    code.push_str("}\n\n");

    // Serialize implementation - this delegates to individual language serialization via their Serialize derives
    code.push_str("impl Serialize for DialectFlags {\n");
    code.push_str("    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>\n");
    code.push_str("    where\n");
    code.push_str("        S: serde::Serializer,\n");
    code.push_str("    {\n");
    code.push_str("        let mut scoped = serializer.serialize_struct(\"DialectFlags\", 4)?;\n");
    code.push_str("        scoped.serialize_field(\"english\", &self.english)?;\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        scoped.serialize_field(\"german\", &self.german)?;\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        scoped.serialize_field(\"portuguese\", &self.portuguese)?;\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        scoped.serialize_field(\"slovak\", &self.slovak)?;\n");
    code.push_str("        scoped.end()\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // This is where it gets complex - the Deserialize implementation
    // The current implementation uses a ScopedDialectFlagsSerde helper
    code.push_str("impl<'de> Deserialize<'de> for DialectFlags {\n");
    code.push_str("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n");
    code.push_str("    where\n");
    code.push_str("        D: serde::Deserializer<'de>,\n");
    code.push_str("    {\n");
    code.push_str(
        "        // Only accept the new scoped, language-specific dialect flags format.\n",
    );
    code.push_str("        let scoped = ScopedDialectFlagsSerde::deserialize(deserializer)?;\n");
    code.push_str("        Ok(scoped.into())\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // From implementation
    code.push_str("impl From<ScopedDialectFlagsSerde> for DialectFlags {\n");
    code.push_str("    fn from(value: ScopedDialectFlagsSerde) -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            english: value.english,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german: value.german,\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese: value.portuguese,\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak: value.slovak,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // empty() method
    code.push_str("impl DialectFlags {\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub const fn empty() -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            english: EnglishDialectFlags::empty(),\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german: GermanDialectFlags::empty(),\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese: PortugueseDialectFlags::empty(),\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak: SlovakDialectFlags::empty(),\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // new() method
    code.push_str("    /// Creates a DialectFlags with the specified dialect flags.\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub const fn new(\n");
    code.push_str("        english: EnglishDialectFlags,\n");
    code.push_str("        #[cfg(feature = \"de\")] german: GermanDialectFlags,\n");
    code.push_str("        #[cfg(feature = \"pt\")] portuguese: PortugueseDialectFlags,\n");
    code.push_str("        #[cfg(feature = \"sk\")] slovak: SlovakDialectFlags,\n");
    code.push_str("    ) -> Self {\n");
    code.push_str("        Self {\n");
    code.push_str("            english,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german,\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese,\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak,\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // from_english_dialect() method
    code.push_str(
        "    /// Creates a DialectFlags with only the specified English dialect enabled.\n",
    );
    code.push_str("    /// This is a convenience method for tests and cases where only English dialects are needed.\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub fn from_english_dialect(dialect: EnglishDialect) -> Self {\n");
    code.push_str("        let english_flags = match dialect {\n");
    code.push_str("            EnglishDialect::American => EnglishDialectFlags::AMERICAN,\n");
    code.push_str("            EnglishDialect::Canadian => EnglishDialectFlags::CANADIAN,\n");
    code.push_str("            EnglishDialect::Australian => EnglishDialectFlags::AUSTRALIAN,\n");
    code.push_str("            EnglishDialect::British => EnglishDialectFlags::BRITISH,\n");
    code.push_str("            EnglishDialect::Indian => EnglishDialectFlags::INDIAN,\n");
    code.push_str("        };\n\n");
    code.push_str("        Self {\n");
    code.push_str("            english: english_flags,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german: GermanDialectFlags::empty(),\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese: PortugueseDialectFlags::empty(),\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak: SlovakDialectFlags::empty(),\n");
    code.push_str("        }\n");
    code.push_str("    }\n\n");

    // is_empty() method
    code.push_str("    #[must_use]\n");
    code.push_str("    #[allow(unused_mut)]\n");
    code.push_str("    pub fn is_empty(self) -> bool {\n");
    code.push_str("        let mut result = self.english.is_empty();\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        {\n");
    code.push_str("            result = result && self.german.is_empty();\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        {\n");
    code.push_str("            result = result && self.portuguese.is_empty();\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        {\n");
    code.push_str("            result = result && self.slovak.is_empty();\n");
    code.push_str("        }\n");
    code.push_str("        result\n");
    code.push_str("    }\n\n");

    // English dialect helper methods
    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn is_english_dialect_enabled(self, dialect: EnglishDialect) -> bool {\n",
    );
    code.push_str("        self.english.is_dialect_enabled(dialect)\n");
    code.push_str("    }\n\n");

    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn is_english_dialect_enabled_strict(self, dialect: EnglishDialect) -> bool {\n",
    );
    code.push_str("        self.english.is_dialect_enabled_strict(dialect)\n");
    code.push_str("    }\n\n");

    // German dialect helper methods
    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub fn is_german_dialect_enabled(self, dialect: GermanDialect) -> bool {\n");
    code.push_str("        self.german.is_dialect_enabled(dialect)\n");
    code.push_str("    }\n\n");

    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn is_german_dialect_enabled_strict(self, dialect: GermanDialect) -> bool {\n",
    );
    code.push_str("        self.german.is_dialect_enabled_strict(dialect)\n");
    code.push_str("    }\n\n");

    // Portuguese dialect helper methods
    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn is_portuguese_dialect_enabled(self, dialect: PortugueseDialect) -> bool {\n",
    );
    code.push_str("        self.portuguese.is_dialect_enabled(dialect)\n");
    code.push_str("    }\n\n");

    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub fn is_portuguese_dialect_enabled_strict(self, dialect: PortugueseDialect) -> bool {\n");
    code.push_str("        self.portuguese.is_dialect_enabled_strict(dialect)\n");
    code.push_str("    }\n\n");

    // Slovak dialect helper methods
    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str("    pub fn is_slovak_dialect_enabled(self, dialect: SlovakDialect) -> bool {\n");
    code.push_str("        self.slovak.is_dialect_enabled(dialect)\n");
    code.push_str("    }\n\n");

    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn is_slovak_dialect_enabled_strict(self, dialect: SlovakDialect) -> bool {\n",
    );
    code.push_str("        self.slovak.is_dialect_enabled_strict(dialect)\n");
    code.push_str("    }\n\n");

    // get_most_used_dialects_from_document method
    code.push_str("    /// Gets the most commonly used dialect(s) in the document.\n");
    code.push_str("    ///\n");
    code.push_str("    /// If multiple dialects are used equally often, they will all be enabled in the returned\n");
    code.push_str("    /// `DialectFlags`. On the other hand, if there is a single dialect that is used the most, it\n");
    code.push_str("    /// will be the only one enabled.\n");
    code.push_str("    #[must_use]\n");
    code.push_str(
        "    pub fn get_most_used_dialects_from_document(document: &crate::Document) -> Self {\n",
    );
    code.push_str("        // Get the most used dialects for each language separately\n");
    code.push_str("        let english_flags = EnglishDialectFlags::get_most_used_dialects_from_document(document);\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        let german_flags = GermanDialectFlags::get_most_used_dialects_from_document(document);\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        let portuguese_flags =\n");
    code.push_str(
        "            PortugueseDialectFlags::get_most_used_dialects_from_document(document);\n",
    );
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        let slovak_flags = SlovakDialectFlags::get_most_used_dialects_from_document(document);\n\n");
    code.push_str("        Self {\n");
    code.push_str("            english: english_flags,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german: german_flags,\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese: portuguese_flags,\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak: slovak_flags,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");

    // BitOr implementation
    code.push_str("}\n");
    code.push_str("impl std::ops::BitOr for DialectFlags {\n");
    code.push_str("    type Output = Self;\n\n");
    code.push_str("    fn bitor(self, rhs: Self) -> Self::Output {\n");
    code.push_str("        Self {\n");
    code.push_str("            english: self.english | rhs.english,\n");
    code.push_str("            #[cfg(feature = \"de\")]\n");
    code.push_str("            german: self.german | rhs.german,\n");
    code.push_str("            #[cfg(feature = \"pt\")]\n");
    code.push_str("            portuguese: self.portuguese | rhs.portuguese,\n");
    code.push_str("            #[cfg(feature = \"sk\")]\n");
    code.push_str("            slovak: self.slovak | rhs.slovak,\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // BitOrAssign implementation
    code.push_str("impl std::ops::BitOrAssign for DialectFlags {\n");
    code.push_str("    fn bitor_assign(&mut self, rhs: Self) {\n");
    code.push_str("        self.english |= rhs.english;\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        {\n");
    code.push_str("            self.german |= rhs.german;\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        {\n");
    code.push_str("            self.portuguese |= rhs.portuguese;\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        {\n");
    code.push_str("            self.slovak |= rhs.slovak;\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // Default implementation
    code.push_str("impl Default for DialectFlags {\n");
    code.push_str("    /// A default value with no dialects explicitly enabled.\n");
    code.push_str("    /// Implicitly, this state corresponds to all dialects being enabled.\n");
    code.push_str("    fn default() -> Self {\n");
    code.push_str("        Self::empty()\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    // ScopedDialectFlagsSerde for deserialization
    code.push_str("// Use the ScopedDialectFlagsSerde and DialectFlags (language-scoped) for serialization/deserialization.\n");
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Default)]\n");
    code.push_str("struct ScopedDialectFlagsSerde {\n");
    code.push_str("    english: EnglishDialectFlags,\n");
    code.push_str("    #[cfg(feature = \"de\")]\n");
    code.push_str("    german: GermanDialectFlags,\n");
    code.push_str("    #[cfg(feature = \"pt\")]\n");
    code.push_str("    portuguese: PortugueseDialectFlags,\n");
    code.push_str("    #[cfg(feature = \"sk\")]\n");
    code.push_str("    slovak: SlovakDialectFlags,\n");
    code.push_str("}\n\n");

    // Deserialize implementation for ScopedDialectFlagsSerde
    code.push_str("impl<'de> Deserialize<'de> for ScopedDialectFlagsSerde {\n");
    code.push_str("    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>\n");
    code.push_str("    where\n");
    code.push_str("        D: serde::Deserializer<'de>,\n");
    code.push_str("    {\n");
    code.push_str("        use serde::de::{Error, Unexpected};\n\n");
    code.push_str("        let value = Value::deserialize(deserializer)?;\n\n");
    code.push_str("        match value {\n");
    code.push_str("            Value::Object(map) => {\n");
    code.push_str("                let mut english = EnglishDialectFlags::default();\n");
    code.push_str("                #[cfg(feature = \"de\")]\n");
    code.push_str("                let mut german = GermanDialectFlags::default();\n");
    code.push_str("                #[cfg(feature = \"pt\")]\n");
    code.push_str("                let mut portuguese = PortugueseDialectFlags::default();\n");
    code.push_str("                #[cfg(feature = \"sk\")]\n");
    code.push_str("                let mut slovak = SlovakDialectFlags::default();\n\n");
    code.push_str("                for (key, val) in map {\n");
    code.push_str("                    match key.as_str() {\n");
    code.push_str("                        \"english\" => {\n");
    code.push_str("                            english = match val {\n");
    code.push_str("                                Value::String(s) => match s.as_str() {\n");
    code.push_str(
        "                                    \"AMERICAN\" => Ok(EnglishDialectFlags::AMERICAN),\n",
    );
    code.push_str(
        "                                    \"CANADIAN\" => Ok(EnglishDialectFlags::CANADIAN),\n",
    );
    code.push_str("                                    \"AUSTRALIAN\" => Ok(EnglishDialectFlags::AUSTRALIAN),\n");
    code.push_str(
        "                                    \"BRITISH\" => Ok(EnglishDialectFlags::BRITISH),\n",
    );
    code.push_str(
        "                                    \"INDIAN\" => Ok(EnglishDialectFlags::INDIAN),\n",
    );
    code.push_str("                                    _ => {\n");
    code.push_str("                                        Err(Error::custom(format!(\"Unknown English dialect: {s}\")))\n");
    code.push_str("                                    }\n");
    code.push_str("                                },\n");
    code.push_str("                                _ => Err(Error::invalid_type(\n");
    code.push_str("                                    Unexpected::Other(\"english\"),\n");
    code.push_str("                                    &\"string\",\n");
    code.push_str("                                )),\n");
    code.push_str("                            }?;\n");
    code.push_str("                        }\n");

    // German deserialization
    code.push_str("                        #[cfg(feature = \"de\")]\n");
    code.push_str("                        \"german\" => {\n");
    code.push_str("                            german = match val {\n");
    code.push_str("                                Value::String(s) => match s.as_str() {\n");
    code.push_str(
        "                                    \"STANDARD\" => Ok(GermanDialectFlags::STANDARD),\n",
    );
    code.push_str(
        "                                    \"AUSTRIAN\" => Ok(GermanDialectFlags::AUSTRIAN),\n",
    );
    code.push_str(
        "                                    \"SWISS\" => Ok(GermanDialectFlags::SWISS),\n",
    );
    code.push_str("                                    _ => Err(Error::custom(format!(\"Unknown German dialect: {s}\"))),\n");
    code.push_str("                                },\n");
    code.push_str("                                _ => {\n");
    code.push_str("                                    Err(Error::invalid_type(Unexpected::Other(\"german\"), &\"string\"))\n");
    code.push_str("                                }\n");
    code.push_str("                            }?;\n");
    code.push_str("                        }\n");

    // Portuguese deserialization
    code.push_str("                        #[cfg(feature = \"pt\")]\n");
    code.push_str("                        \"portuguese\" => {\n");
    code.push_str("                            portuguese = match val {\n");
    code.push_str("                                Value::String(s) => match s.as_str() {\n");
    code.push_str("                                    \"EUROPEAN\" => Ok(PortugueseDialectFlags::EUROPEAN),\n");
    code.push_str("                                    \"BRAZILIAN\" => Ok(PortugueseDialectFlags::BRAZILIAN),\n");
    code.push_str(
        "                                    \"AFRICAN\" => Ok(PortugueseDialectFlags::AFRICAN),\n",
    );
    code.push_str("                                    _ => Err(Error::custom(format!(\n");
    code.push_str("                                        \"Unknown Portuguese dialect: {s}\"\n");
    code.push_str("                                    ))),\n");
    code.push_str("                                },\n");
    code.push_str("                                _ => Err(Error::invalid_type(\n");
    code.push_str("                                    Unexpected::Other(\"portuguese\"),\n");
    code.push_str("                                    &\"string\",\n");
    code.push_str("                                )),\n");
    code.push_str("                            }?;\n");
    code.push_str("                        }\n");

    // Slovak deserialization
    code.push_str("                        #[cfg(feature = \"sk\")]\n");
    code.push_str("                        \"slovak\" => {\n");
    code.push_str("                            slovak = match val {\n");
    code.push_str("                                Value::String(s) => match s.as_str() {\n");
    code.push_str(
        "                                    \"STANDARD\" => Ok(SlovakDialectFlags::STANDARD),\n",
    );
    code.push_str("                                    _ => Err(Error::custom(format!(\"Unknown Slovak dialect: {s}\"))),\n");
    code.push_str("                                },\n");
    code.push_str("                                _ => {\n");
    code.push_str("                                    Err(Error::invalid_type(Unexpected::Other(\"slovak\"), &\"string\"))\n");
    code.push_str("                                }\n");
    code.push_str("                            }?;\n");
    code.push_str("                        }\n");
    code.push_str("                        _ => {\n");
    code.push_str(
        "                            // Build list of valid fields based on enabled features\n",
    );
    code.push_str("                            let valid_fields: Vec<&'static str> = {\n");
    code.push_str("                                #[allow(unused_mut)]\n");
    code.push_str("                                let mut fields = vec![\"english\"];\n");
    code.push_str("                                #[cfg(feature = \"de\")]\n");
    code.push_str("                                {\n");
    code.push_str("                                    fields.push(\"german\");\n");
    code.push_str("                                }\n");
    code.push_str("                                #[cfg(feature = \"pt\")]\n");
    code.push_str("                                {\n");
    code.push_str("                                    fields.push(\"portuguese\");\n");
    code.push_str("                                }\n");
    code.push_str("                                #[cfg(feature = \"sk\")]\n");
    code.push_str("                                {\n");
    code.push_str("                                    fields.push(\"slovak\");\n");
    code.push_str("                                }\n");
    code.push_str("                                fields\n");
    code.push_str("                            };\n");
    code.push_str(
        "                            // Convert to a static slice by leaking the memory\n",
    );
    code.push_str("                            // This is safe as it's only done during deserialization error handling\n");
    code.push_str(
        "                            let valid_fields_static: &'static [&'static str] =\n",
    );
    code.push_str("                                Box::leak(valid_fields.into_boxed_slice());\n");
    code.push_str("                            return Err(Error::unknown_field(&key, valid_fields_static));\n");
    code.push_str("                        }\n");
    code.push_str("                    }\n");
    code.push_str("                }\n");
    code.push_str("                Ok(ScopedDialectFlagsSerde {\n");
    code.push_str("                    english,\n");
    code.push_str("                    #[cfg(feature = \"de\")]\n");
    code.push_str("                    german,\n");
    code.push_str("                    #[cfg(feature = \"pt\")]\n");
    code.push_str("                    portuguese,\n");
    code.push_str("                    #[cfg(feature = \"sk\")]\n");
    code.push_str("                    slovak,\n");
    code.push_str("                })\n");
    code.push_str("            }\n");
    code.push_str("            Value::String(s) => Err(Error::custom(format!(\n");
    code.push_str("                \"Legacy flat string format for dialect flags is no longer supported: {s}\"\n");
    code.push_str("            ))),\n");
    code.push_str("            _ => Err(Error::custom(\"Expected object for dialect flags\")),\n");
    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");
    // Write the generated file
    let dest = src_dir.join("dialects").join("dialect_flags.rs");
    fs::create_dir_all(dest.parent().unwrap()).unwrap();
    fs::write(&dest, code).unwrap();

    println!("cargo:rerun-if-changed=src/language/dialects/dialect_flags.rs");
}
