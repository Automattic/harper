// Source-tree checks for Weir-rule build inputs.
// These validate the expected rule layout and file structure; they do not
// exercise Cargo's generated output directly.

use std::fs;
use std::path::Path;

/// Verify that German Weir rule directory is configured
#[test]
fn verify_german_weir_rule_dir_exists() {
    // Check that the German Weir rule directory exists and contains .weer files
    let weir_dir = Path::new("src/linting/weir_rules/de");

    // Verify directory exists
    assert!(
        weir_dir.exists(),
        "German Weir rules directory should exist at {:?}",
        weir_dir
    );

    // Verify it contains .weir files
    let entries = fs::read_dir(weir_dir).unwrap();
    let weir_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "weir")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !weir_files.is_empty(),
        "German Weir rules directory should contain at least one .weir file, found {}",
        weir_files.len()
    );

    // Expected German Weir rules
    let expected_rules = vec![
        "BeimBei",
        "EtwasDass",
        "HerzlichenDank",
        "VielenDank",
        "WirHaben",
        "ZumAnbeissen",
        "ZurDer",
    ];

    for rule_name in expected_rules {
        let rule_file = weir_dir.join(format!("{}.weir", rule_name));
        assert!(
            rule_file.exists(),
            "Expected German Weir rule file {:?} should exist",
            rule_file
        );
    }
}

/// Verify that English Weir rule directory exists
#[test]
fn verify_english_weir_rule_dir_exists() {
    let weir_dir = Path::new("src/linting/weir_rules");

    assert!(
        weir_dir.exists(),
        "English Weir rules directory should exist at {:?}",
        weir_dir
    );

    // Verify it contains .weir files
    let entries = fs::read_dir(weir_dir).unwrap();
    let weir_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "weir")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        weir_files.len() > 50,
        "English Weir rules directory should contain many .weir files, found {}",
        weir_files.len()
    );
}

/// Verify each German Weir rule file has proper structure
#[test]
fn verify_german_weir_rules_structure() {
    use std::ffi::OsStr;

    let weir_dir = Path::new("src/linting/weir_rules/de");
    let entries = fs::read_dir(weir_dir).unwrap();

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension() != Some(OsStr::new("weir")) {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        // Each Weir rule should have these components
        assert!(
            content.contains("expr main"),
            "{:?} should define 'expr main' pattern",
            path
        );
        assert!(
            content.contains("let message"),
            "{:?} should define error message",
            path
        );
        assert!(
            content.contains("let becomes") || content.contains("let strategy"),
            "{:?} should define replacement or strategy",
            path
        );

        // Should have at least one test or allows statement
        assert!(
            content.contains("test ") || content.contains("allows "),
            "{:?} should have at least one test or allows statement",
            path
        );
    }
}

/// Verify build.rs environment variables are documented
#[test]
fn verify_build_script_exists() {
    let build_rs = Path::new("build.rs");

    assert!(
        build_rs.exists(),
        "build.rs should exist in harper-core directory"
    );

    let content = fs::read_to_string(build_rs).unwrap();

    // Verify build.rs handles German Weir rules
    assert!(
        content.contains("german_weir_rule_dir"),
        "build.rs should reference german_weir_rule_dir"
    );
    assert!(
        content.contains("german_weir_rules_generated_list"),
        "build.rs should generate german_weir_rules_generated_list.rs"
    );
    assert!(
        content.contains("GERMAN_WEIR_RULE_DIR"),
        "build.rs should set GERMAN_WEIR_RULE_DIR environment variable"
    );
}
