use std::io::Write;
use std::process::Command;

fn harper_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harper-cli"))
}

/// Input that triggers at least one lint ("an test" → AnA rule).
const BAD_INPUT: &str = "This is an test.";

#[test]
fn json_format_is_valid_json() {
    let output = harper_cli()
        .args(["--no-color", "lint", "--format", "json", BAD_INPUT])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON output: {e}\nstdout: {stdout}"));

    let arr = parsed.as_array().expect("top-level should be an array");
    assert_eq!(arr.len(), 1, "single input should produce one result");

    let first = &arr[0];
    assert!(
        first.get("file").is_some(),
        "result should have a 'file' field"
    );
    assert!(
        first.get("lint_count").is_some(),
        "result should have a 'lint_count' field"
    );
    assert!(
        first.get("lints").is_some(),
        "result should have a 'lints' field"
    );

    let lints = first["lints"].as_array().unwrap();
    assert!(!lints.is_empty(), "should have at least one lint");

    let lint = &lints[0];
    assert!(lint.get("rule").is_some());
    assert!(lint.get("kind").is_some());
    assert!(lint.get("line").is_some());
    assert!(lint.get("column").is_some());
    assert!(lint.get("message").is_some());
    assert!(lint.get("suggestions").is_some());
}

#[test]
fn json_format_no_ansi() {
    let output = harper_cli()
        .args(["lint", "--format", "json", BAD_INPUT])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check raw bytes for ESC
    assert!(
        !output.stdout.contains(&0x1b),
        "JSON output should never contain raw ANSI escape bytes"
    );

    // Also check for escaped ANSI in JSON string values (e.g. \\u001b)
    assert!(
        !stdout.contains("\\u001b") && !stdout.contains("\\x1b"),
        "JSON output should not contain escaped ANSI sequences in values"
    );
}

/// Regression test: directory inputs with color enabled must not leak
/// ANSI escapes into JSON `file` fields or compact output paths.
#[test]
fn json_directory_paths_no_ansi() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("bad.md");
    {
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(f, "{BAD_INPUT}").unwrap();
    }

    // Run with color explicitly enabled (no --no-color, NO_COLOR removed)
    let output = harper_cli()
        .args(["lint", "--format", "json", dir.path().to_str().unwrap()])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON from directory lint: {e}\nstdout: {stdout}"));

    let arr = parsed.as_array().expect("top-level should be an array");
    assert!(!arr.is_empty(), "should have at least one file result");

    for entry in arr {
        let file_val = entry["file"].as_str().expect("file should be a string");
        assert!(
            !file_val.contains('\x1b'),
            "JSON file field must not contain ANSI escapes, got: {file_val:?}"
        );
    }
}

#[test]
fn compact_directory_paths_no_ansi() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("bad.md");
    {
        let mut f = std::fs::File::create(&file_path).unwrap();
        write!(f, "{BAD_INPUT}").unwrap();
    }

    // Run with color explicitly enabled
    let output = harper_cli()
        .args(["lint", "--format", "compact", dir.path().to_str().unwrap()])
        .env_remove("NO_COLOR")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains('\x1b'),
        "compact output must not contain ANSI escapes in paths"
    );
}

#[test]
fn compact_format_one_line_per_lint() {
    let output = harper_cli()
        .args(["--no-color", "lint", "--format", "compact", BAD_INPUT])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        !lines.is_empty(),
        "compact output should have at least one line"
    );

    for line in &lines {
        // Each line should match the pattern: source:line:col: Kind::Rule: message
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        assert!(
            parts.len() >= 4,
            "compact line should have at least 4 colon-separated parts: {line}"
        );
    }
}

#[test]
fn default_format_unchanged() {
    let output = harper_cli()
        .args(["--no-color", "lint", "--format", "default", BAD_INPUT])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Default format should include the Ariadne report header and lint count
    assert!(
        stdout.contains("lints") || stdout.contains("No lints found"),
        "default format should include lint count summary"
    );
}
