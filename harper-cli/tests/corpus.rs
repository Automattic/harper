use std::io::Write;
use std::process::Command;

use tempfile::NamedTempFile;

fn harper_cli() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_harper-cli"));
    cmd.arg("--no-color");
    cmd
}

fn write_corpus(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn correct_sentence_passes() {
    let f = write_corpus("✅ This sentence is perfectly grammatical.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[PASS]"));
    assert!(!stdout.contains("[FAIL]"));
    assert!(output.status.success());
}

#[test]
fn incorrect_sentence_passes() {
    let f = write_corpus("❌ This is an test.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[PASS]"));
    assert!(!stdout.contains("[FAIL]"));
    assert!(output.status.success());
}

#[test]
fn false_positive_detected() {
    // Marking a correct sentence as expected-to-fail → false negative detection.
    let f = write_corpus("❌ This is correct.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[FAIL]"));
    assert!(!output.status.success());
}

#[test]
fn false_negative_detected() {
    // Marking a bad sentence as expected-to-pass → false positive detection.
    let f = write_corpus("✅ This is an test.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[FAIL]"));
    assert!(!output.status.success());
}

#[test]
fn inline_rule_filter() {
    let f = write_corpus("❌ `AnA` This is an test.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[PASS]"));
    assert!(output.status.success());
}

#[test]
fn global_rule_flag() {
    let f = write_corpus("❌ This is an test.\n");
    let output = harper_cli()
        .args(["corpus", "--rule", "AnA", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[PASS]"));
    assert!(output.status.success());
}

#[test]
fn comments_and_blanks_skipped() {
    let f = write_corpus("# This is a comment\n\n✅ This is fine.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[PASS]"));
    assert!(stdout.contains("1 total"));
    assert!(output.status.success());
}

#[test]
fn summary_line_printed() {
    let f = write_corpus("✅ Good sentence.\n❌ This is an test.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 total, 2 passed, 0 failed"));
    assert!(output.status.success());
}

#[test]
fn exit_code_nonzero_on_failure() {
    let f = write_corpus("✅ This is an test.\n");
    let output = harper_cli()
        .args(["corpus", f.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
}
