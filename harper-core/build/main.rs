// Harper Core Build Script Main Logic
//
// This module contains the main orchestration logic for Harper's compile-time
// code generation system. It is invoked by `build.rs`.
//
// ## What This Script Does
//
// 1. **Weir Rule Processing**:
//    - Processes main English weir rules from `src/linting/weir_rules/`
//    - Discovers and processes language-specific weir rules from `src/language/<lang>/linting/weir_rules/`
//
// 2. **Language Integration**:
//    - Generates `src/language/mod.rs`, `src/language/languages.rs`
//    - Generates `src/language/registry.rs`, `src/language/dialects/dialect_flags.rs`
//
// 3. **Feature-based Compilation**:
//    - Automatically adds `#[cfg(feature)]` attributes for optional languages
//    - English is always included (no feature flag)

use std::{env, path::Path};

mod build_lib;

use build_lib::language_config;
use build_lib::weir_rules::{write_grouped_weir_boilerplate, process_language_weir_rules};
use build_lib::language_modules::generate_language_modules;

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