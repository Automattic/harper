use std::{env, fs, path::Path};

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

    if let Ok(language_entries) = fs::read_dir(&language_dir) {
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

            // Only process if the final directory exists and contains .weir files
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
        "        \"us\" | \"usa\" | \"america\" | \"american\" | \"en-us\" | \"en_us\" =>\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::American)),\n");
    code.push_str(
        "        \"uk\" | \"gb\" | \"british\" | \"britain\" | \"en-gb\" | \"en_gb\" =>\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::British)),\n");
    code.push_str(
        "        \"au\" | \"aus\" | \"australia\" | \"australian\" | \"en-au\" | \"en_au\" =>\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::Australian)),\n");
    code.push_str(
        "        \"in\" | \"india\" | \"indian\" | \"bharat\" | \"en-in\" | \"en_in\" =>\n",
    );
    code.push_str("            Some(Language::English(EnglishDialect::Indian)),\n");
    code.push_str("        \"ca\" | \"canada\" | \"canadian\" | \"en-ca\" | \"en_ca\" =>\n");
    code.push_str("            Some(Language::English(EnglishDialect::Canadian)),\n");

    code.push_str("        // German\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"de\" | \"german\" | \"deutsch\" | \"de-de\" | \"de_de\" =>\n");
    code.push_str("            Some(Language::German(GermanDialect::Standard)),\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"at\" | \"austria\" | \"austrian\" | \"de-at\" | \"de_at\" =>\n");
    code.push_str("            Some(Language::German(GermanDialect::Austrian)),\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        \"ch\" | \"switzerland\" | \"swiss\" | \"de-ch\" | \"de_ch\" =>\n");
    code.push_str("            Some(Language::German(GermanDialect::Swiss)),\n");

    code.push_str("        // Portuguese\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str(
        "        \"pt\" | \"pt-pt\" | \"pt_pt\" | \"portuguese\" | \"portugu\\u{00ea}s\" =>\n",
    );
    code.push_str("            Some(Language::Portuguese(PortugueseDialect::European)),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        \"br\" | \"brazil\" | \"portuguese-brazilian\" | \"portuguese_brazilian\" | \"pt-br\" | \"pt_br\" =>\n");
    code.push_str("            Some(Language::Portuguese(PortugueseDialect::Brazilian)),\n");
    code.push_str("        #[cfg(feature = \"pt\")]\n");
    code.push_str("        \"ao\" => Some(Language::Portuguese(PortugueseDialect::African)),\n");

    code.push_str("        // Slovak\n");
    code.push_str("        #[cfg(feature = \"sk\")]\n");
    code.push_str("        \"sk\" | \"slovak\" | \"slovensko\" | \"sk-sk\" | \"sk_sk\" =>\n");
    code.push_str("            Some(Language::Slovak(SlovakDialect::Standard)),\n");

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
    code.push_str("    Default, Debug, Clone, Copy, Serialize, Deserialize,\n");
    code.push_str(
        "    PartialEq, PartialOrd, Eq, Hash, EnumCount, EnumString, EnumIter, Display,\n",
    );
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
    code.push_str("            let group = EnglishModule::curated_lint_group(_dialect);\n");
    code.push_str("            group\n");
    code.push_str("        }\n");
    code.push_str("        #[cfg(feature = \"de\")]\n");
    code.push_str("        Language::German(_dialect) => {\n");
    code.push_str("            use crate::language::german::module::GermanModule;\n\n");
    code.push_str("            let lang_dict = GermanModule::dictionary();\n");
    code.push_str("            let mut group = LintGroup::empty();\n");
    code.push_str("            group.merge_from(GermanModule::weir_lint_group());\n");
    code.push_str("            group.merge_from(GermanModule::rust_lint_group(lang_dict));\n");
    code.push_str("            group.set_all_rules_to(Some(true));\n");
    code.push_str("            group\n");
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
}
