// Build script for harper-wasm
// This script generates the Dialect enum from harper-core's language
// configurations, so every language shipped in harper-core is reflected in the
// WebAssembly-facing enum without manual updates.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

/// Format generated Rust source using the `rustfmt` binary, reading from stdin.
///
/// If `rustfmt` is unavailable or fails, the original content is returned
/// unformatted so that a real change is still persisted.
fn format_rust_content(content: &str) -> String {
    let mut child = match Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            eprintln!("Warning: rustfmt not found; writing unformatted generated code.");
            return content.to_string();
        }
    };

    if child
        .stdin
        .as_mut()
        .and_then(|stdin| stdin.write_all(content.as_bytes()).ok())
        .is_none()
    {
        eprintln!("Warning: failed to write to rustfmt; writing unformatted generated code.");
        return content.to_string();
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_else(|_| content.to_string())
        }
        _ => {
            eprintln!("Warning: rustfmt failed; writing unformatted generated code.");
            content.to_string()
        }
    }
}

/// Convert a string to PascalCase.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_ascii_uppercase().to_string();
                    result.extend(chars.map(|c| c.to_ascii_lowercase()));
                    result
                }
            }
        })
        .collect()
}

/// Convert a dialect name for use in the Dialect enum variant.
/// Preserves uppercase for short codes (like PT, BR, AO) and converts others
/// to PascalCase.
fn to_dialect_variant(dialect_name: &str) -> String {
    // If the dialect name is all uppercase (like PT, BR, AO), keep it as-is.
    if dialect_name.chars().all(|c| c.is_ascii_uppercase()) {
        dialect_name.to_string()
    } else {
        // Otherwise convert to PascalCase.
        to_pascal_case(dialect_name)
    }
}

/// Known non-language directories in harper-core's src/language/ directory.
const NON_LANGUAGE_DIRS: [&str; 2] = ["dialects", "testing_framework"];

/// A language discovered from a harper-core config.toml.
struct LanguageConfig {
    /// Directory name under `harper-core/src/language/`, e.g. `german`.
    dir_name: String,
    /// Human-readable name used for enum variants, e.g. `German`.
    name: String,
    /// Cargo feature gating the language, e.g. `de`.
    feature: String,
    /// Dialect names as written in `config.toml`. These are also the
    /// abbreviations accepted by the language's `Dialect::try_from_abbr`.
    dialects: Vec<String>,
}

/// Discover non-English languages from harper-core's language config files.
///
/// Returns the languages sorted by directory name for deterministic output.
/// English is skipped because its dialects are always emitted first.
fn discover_languages(language_dir: &Path) -> Vec<LanguageConfig> {
    let mut languages = Vec::new();

    if !language_dir.exists() || !language_dir.is_dir() {
        eprintln!(
            "Warning: Language directory not found at {}",
            language_dir.display()
        );
        return languages;
    }

    let mut entries: Vec<_> = fs::read_dir(language_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .collect();

    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let dir_name = match entry.file_name().to_str() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Skip non-language directories and English (always emitted first).
        if NON_LANGUAGE_DIRS.contains(&dir_name.as_str()) || dir_name == "english" {
            continue;
        }

        let config_path = entry.path().join("config.toml");
        if !config_path.exists() {
            continue;
        }

        println!("cargo:rerun-if-changed={}", config_path.display());

        let content = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let table = match content.parse::<toml::Table>() {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Warning: Failed to parse {} as TOML", config_path.display());
                continue;
            }
        };

        let Some(toml::Value::Table(language_table)) = table.get("language") else {
            eprintln!(
                "Warning: Missing [language] section in {}",
                config_path.display()
            );
            continue;
        };

        let Some(name) = language_table.get("name").and_then(|v| v.as_str()) else {
            eprintln!(
                "Warning: Missing 'name' in [language] section for {}",
                config_path.display()
            );
            continue;
        };

        let mut dialects = Vec::new();
        if let Some(toml::Value::Array(dialects_array)) = table.get("dialects") {
            for dialect_table in dialects_array {
                if let toml::Value::Table(table) = dialect_table
                    && let Some(toml::Value::String(dialect_name)) = table.get("name")
                {
                    dialects.push(dialect_name.clone());
                }
            }
        }

        let Some(feature) = language_table.get("feature").and_then(|v| v.as_str()) else {
            eprintln!(
                "Warning: Missing 'feature' in [language] section for {}",
                config_path.display()
            );
            continue;
        };

        languages.push(LanguageConfig {
            dir_name: dir_name.clone(),
            name: name.to_string(),
            feature: feature.to_string(),
            dialects,
        });
    }

    languages
}

/// Generate `impl From<Dialect> for Language`, plus the dialect-type imports.
///
/// Hand-writing this meant ~18 `#[cfg]` arms to add per new language, and
/// forgetting the `#[cfg(not(...))]` fallback produced a non-exhaustive match
/// only in builds that excluded the language. Both are now derived from
/// `config.toml`.
///
/// Dialects are converted via `try_from_abbr`, which accepts the dialect names
/// exactly as written in `config.toml` -- Portuguese `PT`/`BR`/`AO` map to
/// `European`/`Brazilian`/`African`, so the Rust variant names are never needed
/// here.
///
/// `try_from_abbr` comes from the `Dialect` trait, which is called through
/// fully-qualified syntax rather than a `use`. A plain import would be unused in
/// a build with no dialect-bearing language compiled in (`--no-default-features`),
/// and CI builds with `-D warnings`.
fn generate_language_conversion(code: &mut String, languages: &[LanguageConfig]) {
    let with_dialects: Vec<&LanguageConfig> = languages
        .iter()
        .filter(|lang| !lang.dialects.is_empty())
        .collect();

    for lang in &with_dialects {
        code.push_str(&format!("#[cfg(feature = \"{}\")]\n", lang.feature));
        code.push_str(&format!(
            "use harper_core::language::{}::dialects::{}Dialect;\n",
            lang.dir_name, lang.name
        ));
    }
    code.push('\n');

    code.push_str("impl From<Dialect> for harper_core::language::languages::Language {\n");
    code.push_str("    fn from(dialect: Dialect) -> Self {\n");
    code.push_str("        use harper_core::language::languages::Language;\n\n");
    code.push_str("        match dialect {\n");

    // English is always compiled in.
    for variant in ["American", "British", "Australian", "Canadian", "Indian"] {
        code.push_str(&format!(
            "            Dialect::{} => Language::English(EnglishDialect::{}),\n",
            variant, variant
        ));
    }

    for lang in &with_dialects {
        for dialect_name in &lang.dialects {
            let variant = format!("{}{}", lang.name, to_dialect_variant(dialect_name));
            code.push_str(&format!(
                "            #[cfg(feature = \"{}\")]\n",
                lang.feature
            ));
            code.push_str(&format!(
                "            Dialect::{} => Language::{}(<{}Dialect as \
                 harper_core::language::dialects::dialect_trait::Dialect>::try_from_abbr(\"{}\")\n\
                 {:16}.expect(\"dialect abbreviation comes from config.toml\")),\n",
                variant, lang.name, lang.name, dialect_name, ""
            ));
        }
    }

    // Fall back to English for any language not compiled into this build.
    for lang in &with_dialects {
        let variants: Vec<String> = lang
            .dialects
            .iter()
            .map(|d| format!("Dialect::{}{}", lang.name, to_dialect_variant(d)))
            .collect();

        code.push_str(&format!(
            "            #[cfg(not(feature = \"{}\"))]\n",
            lang.feature
        ));
        code.push_str(&format!(
            "            {} => Language::English(EnglishDialect::American),\n",
            variants.join(" | ")
        ));
    }

    code.push_str("        }\n");
    code.push_str("    }\n");
    code.push_str("}\n");
}

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // Path to harper-core's language configuration directory.
    let language_dir = manifest_dir
        .parent()
        .unwrap()
        .join("harper-core/src/language");

    let languages = discover_languages(&language_dir);

    let mut code = String::new();
    code.push_str("// Auto-generated by harper-wasm build.rs - do not edit manually\n");
    code.push_str("// This file provides the Dialect enum for harper-wasm.\n");
    code.push_str("// It is auto-generated from harper-core's language configurations\n");
    code.push_str("// to ensure all languages are automatically supported.\n\n");

    // Generate the Dialect enum.
    code.push_str("/// Specifies a dialect, often used for linting.\n");
    code.push_str("#[wasm_bindgen]\n");
    code.push_str("#[derive(Serialize, Deserialize, Debug, Clone, Copy)]\n");
    code.push_str("pub enum Dialect {\n");

    // English dialects - always included.
    code.push_str("    // English dialects\n");
    code.push_str("    American,\n");
    code.push_str("    British,\n");
    code.push_str("    Australian,\n");
    code.push_str("    Canadian,\n");
    code.push_str("    Indian,\n");

    for lang in &languages {
        if lang.dialects.is_empty() {
            continue;
        }

        code.push_str(&format!("    // {} dialects\n", lang.name));
        for dialect_name in &lang.dialects {
            let dialect_variant = to_dialect_variant(dialect_name);
            let variant = format!("{}{}", lang.name, dialect_variant);
            code.push_str(&format!("    {},\n", variant));
        }
    }

    code.push_str("}\n\n");

    generate_language_conversion(&mut code, &languages);

    let formatted = format_rust_content(&code);

    fs::create_dir_all(out_dir).unwrap();
    let dest = out_dir.join("generated_dialect.rs");
    fs::write(&dest, formatted).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
