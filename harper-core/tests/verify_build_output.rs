//! Source-tree checks on the inputs the build script consumes.
//!
//! Like `language_conformance.rs`, these walk whatever is in the tree rather
//! than naming a language, so a new one is checked as soon as it is added.
//! They validate file layout and rule structure, not generated output.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const LANGUAGE_DIR: &str = "src/language";

/// Every `linting/weir_rules` directory under `src/language/`, with the
/// language directory it belongs to.
fn language_weir_dirs() -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(LANGUAGE_DIR) else {
        panic!("{LANGUAGE_DIR} should exist");
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let rules = path.join("linting/weir_rules");
        if rules.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            found.push((name, rules));
        }
    }
    found.sort();
    found
}

/// `.weir` files sit either directly in `weir_rules/` or in the one
/// subdirectory a language's `config.toml` names.
fn weir_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension() == Some(OsStr::new("weir")) {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn english_ships_the_bulk_of_the_weir_rules() {
    let dir = Path::new("src/linting/weir_rules");
    assert!(dir.is_dir(), "{} should exist", dir.display());
    let count = weir_files(dir).len();
    assert!(
        count > 50,
        "English should have many .weir rules, found {count}"
    );
}

/// The shape `WeirLinter` needs to build a linter out of a rule file. Applied
/// to the language modules only: English's rule set predates the convention
/// that every rule carries its own examples.
#[test]
fn every_language_weir_rule_is_well_formed() {
    for (language, dir) in language_weir_dirs() {
        for path in weir_files(&dir) {
            let content = fs::read_to_string(&path).unwrap();
            let file = path.display();

            assert!(
                content.contains("expr main"),
                "{language}: {file} defines no 'expr main' pattern"
            );
            assert!(
                content.contains("let message"),
                "{language}: {file} defines no message"
            );
            assert!(
                content.contains("let becomes") || content.contains("let strategy"),
                "{language}: {file} defines neither a replacement nor a strategy"
            );
            assert!(
                content.contains("test ") || content.contains("allows "),
                "{language}: {file} has no test or allows statement"
            );
        }
    }
}

/// The two files the build script needs in order to see a language at all.
#[test]
fn every_language_directory_has_a_config_and_a_module() {
    let mut checked = 0;

    for entry in fs::read_dir(LANGUAGE_DIR).unwrap().filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        // Shared infrastructure, not languages.
        if !path.is_dir() || matches!(name.as_str(), "dialects" | "testing_framework") {
            continue;
        }

        let config = path.join("config.toml");
        let module = path.join("module.rs");
        assert_eq!(
            config.exists(),
            module.exists(),
            "{name} has only one of config.toml and module.rs; the build script \
             needs both and silently ignores the directory otherwise"
        );

        if config.exists() {
            checked += 1;
            assert!(
                path.join("mod.rs").exists(),
                "{name} has no mod.rs, so the generated `pub mod {name};` cannot resolve"
            );
            assert!(
                path.join("dialects.rs").exists(),
                "{name} has no dialects.rs, which languages.rs imports its dialect from"
            );
        }
    }

    assert!(checked > 0, "no language directories found");
}

#[test]
fn verify_build_script_exists() {
    let content = fs::read_to_string("build.rs").expect("harper-core/build.rs should exist");

    assert!(
        content.contains("mod build_lib"),
        "build.rs should import the build_lib module"
    );
    assert!(
        content.contains("build_lib::run_build()"),
        "build.rs should call build_lib::run_build()"
    );
}
