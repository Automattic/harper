//! Build script for Harper core library.
//!
//! This script generates test boilerplate for Weir rules (pattern-based linters).
//! Weir rules are defined in `.weir` files with inline tests, and this script
//! automatically generates Rust test functions at compile time.
//!
//! ## German Language Support
//!
//! For German language support, we maintain a separate directory of Weir rules
//! at `src/linting/weir_rules/de/`. Each rule defines common German error patterns
//! (e.g., "bei beim" → "beim", "viele Dank" → "vielen Dank").
//!
//! The build script:
//! 1. Scans both English and German Weir rule directories
//! 2. Generates two separate test lists: `weir_rules_generated_list.rs` and
//!    `german_weir_rules_generated_list.rs`
//! 3. Sets environment variables so the code can locate the rules at runtime
//!
//! ## Adding New Weir Rules
//!
//! To add a new German Weir rule:
//! 1. Create a new `.weir` file in `src/linting/weir_rules/de/`
//! 2. Define the pattern, message, and inline tests
//! 3. Rebuild - the build script will automatically include it in the generated list

use std::{env, fs, path::PathBuf};

/// Collect all `.weir` files from a directory, sorted alphabetically.
/// Returns an empty vector if the directory doesn't exist (for optional language support).
fn collect_weir_files(dir: &PathBuf) -> Vec<PathBuf> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut files: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().unwrap().is_file())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "weir").unwrap_or(false))
                .collect();
            files.sort();
            files
        }
        Err(_) => vec![],
    }
}

/// Generate Rust code that creates test functions for each Weir rule.
/// The output is a file containing: `generate_boilerplate!{[Rule1, Rule2, ...]}`
/// which is then expanded by the macro in `src/linting/weir_rules/mod.rs`.
fn write_boilerplate(files: &[PathBuf], dest: &PathBuf) {
    let mut code = String::new();
    code.push_str("generate_boilerplate!{[");
    for file in files {
        code.push_str(&format!(
            "{},\n",
            file.file_stem().unwrap().to_str().unwrap()
        ));
    }
    code.push_str("]}");
    fs::write(dest, code).unwrap();
}

fn main() {
    // Locate source directories for Weir rules
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let weir_rule_dir = manifest_dir.join("./src/linting/weir_rules");
    let german_weir_rule_dir = manifest_dir.join("./src/linting/weir_rules/de");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Generate test lists for both English and German Weir rules
    let dest = out_dir.join("weir_rules_generated_list.rs");
    let de_dest = out_dir.join("german_weir_rules_generated_list.rs");

    let files = collect_weir_files(&weir_rule_dir);
    write_boilerplate(&files, &dest);

    let de_files = collect_weir_files(&german_weir_rule_dir);
    write_boilerplate(&de_files, &de_dest);

    // Tell Cargo to rerun this script if any Weir files change
    println!("cargo:rerun-if-changed={}", weir_rule_dir.display());
    println!("cargo:rerun-if-changed={}", german_weir_rule_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    // Export environment variables for use in the codebase
    // These allow the code to locate Weir rule files and generated test lists at runtime
    println!("cargo:rustc-env=WEIR_RULE_DIR={}", weir_rule_dir.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_DIR={}",
        german_weir_rule_dir.display()
    );
    println!("cargo:rustc-env=WEIR_RULE_LIST={}", dest.display());
    println!(
        "cargo:rustc-env=GERMAN_WEIR_RULE_LIST={}",
        de_dest.display()
    );
}
